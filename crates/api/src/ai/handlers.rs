// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Row};
use tracing::{error, info};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

use super::{
    client::{AiError, Message, ToolCall},
    execute_tool, tool_definitions,
};

const SYSTEM_PROMPT: &str = r#"You are a helpful, read-only assistant for Power OS, a Malaysian business operating system.

You can answer questions about the workspace's contacts, companies, deals, invoices, and employees. You must use the provided tools to fetch data; do not make up information.

Rules:
- You are read-only. You cannot create, update, or delete anything.
- Use plain, concise language.
- If a tool returns an error, explain it to the user.
- If no relevant data is found, say so clearly.
- Malaysian context: amounts are normally in MYR unless stated otherwise.
"#;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub session_id: Uuid,
    pub message: String,
    pub tool_calls: Vec<ToolCallLog>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallLog {
    pub tool_name: String,
    pub arguments: Value,
    pub result: Option<Value>,
    pub error_message: Option<String>,
}

pub async fn chat(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ChatRequest>,
) -> Result<(StatusCode, Json<ChatResponse>), ApiError> {
    if !state.ai.is_configured() {
        return Err(ApiError::BadRequest(
            "AI provider is not configured. Set OPENROUTER_API_KEY.".to_string(),
        ));
    }

    let session_id = match payload.session_id {
        Some(id) => {
            ensure_session_belongs_to_user(
                &state.db,
                id,
                auth_user.user_id.0,
                auth_user.workspace_id.0,
            )
            .await?
        }
        None => {
            create_session(
                &state.db,
                auth_user.workspace_id.0,
                auth_user.user_id.0,
                &payload.message,
            )
            .await?
        }
    };

    let mut messages = load_messages(&state.db, session_id).await?;
    messages.push(Message::user(payload.message));

    let tools = tool_definitions();
    let assistant_message = state
        .ai
        .chat(messages.clone(), tools)
        .await
        .map_err(map_ai_error)?;

    let mut tool_calls_log: Vec<ToolCallLog> = Vec::new();
    let mut current_messages = messages.clone();

    if let Some(tool_calls) = assistant_message.tool_calls.clone() {
        save_assistant_message(&state.db, session_id, &assistant_message).await?;
        current_messages.push(assistant_message);

        for tool_call in tool_calls {
            let arguments: Value =
                serde_json::from_str(&tool_call.function.arguments).unwrap_or(Value::Null);

            let (result, error_message) = match execute_tool(
                &state.db,
                auth_user.workspace_id.0,
                &tool_call.function.name,
                &arguments,
            )
            .await
            {
                Ok(value) => (Some(value.clone()), None),
                Err(err) => {
                    error!(
                        tool_name = %tool_call.function.name,
                        error = %err,
                        "AI tool execution failed"
                    );
                    (None, Some(err.to_string()))
                }
            };

            tool_calls_log.push(ToolCallLog {
                tool_name: tool_call.function.name.clone(),
                arguments: arguments.clone(),
                result: result.clone(),
                error_message: error_message.clone(),
            });

            let tool_result_json = if let Some(err) = &error_message {
                serde_json::json!({ "error": err }).to_string()
            } else {
                serde_json::to_string(&result).unwrap_or_default()
            };

            let tool_message_record = query(
                r#"
                INSERT INTO ai_message (session_id, role, content, tool_call_id)
                VALUES ($1, 'tool', $2, $3)
                RETURNING id
                "#,
            )
            .bind(session_id)
            .bind(&tool_result_json)
            .bind(&tool_call.id)
            .fetch_one(&state.db)
            .await?;

            let tool_message_id: Uuid = tool_message_record.try_get("id")?;

            query(
                r#"
                INSERT INTO ai_tool_call (
                    session_id, message_id, tool_name, arguments, result, error_message
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(session_id)
            .bind(tool_message_id)
            .bind(&tool_call.function.name)
            .bind(&arguments)
            .bind(&result)
            .bind(error_message.as_deref())
            .execute(&state.db)
            .await?;

            current_messages.push(Message::tool(tool_call.id.clone(), tool_result_json));
        }

        let final_message = state
            .ai
            .chat(current_messages, tool_definitions())
            .await
            .map_err(map_ai_error)?;
        save_assistant_message(&state.db, session_id, &final_message).await?;

        Ok((
            StatusCode::OK,
            Json(ChatResponse {
                session_id,
                message: final_message.content.unwrap_or_default(),
                tool_calls: tool_calls_log,
            }),
        ))
    } else {
        save_assistant_message(&state.db, session_id, &assistant_message).await?;

        Ok((
            StatusCode::OK,
            Json(ChatResponse {
                session_id,
                message: assistant_message.content.unwrap_or_default(),
                tool_calls: tool_calls_log,
            }),
        ))
    }
}

async fn ensure_session_belongs_to_user(
    db: &sqlx::PgPool,
    session_id: Uuid,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<Uuid, ApiError> {
    let exists =
        query("SELECT 1 FROM ai_session WHERE id = $1 AND user_id = $2 AND workspace_id = $3")
            .bind(session_id)
            .bind(user_id)
            .bind(workspace_id)
            .fetch_optional(db)
            .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    Ok(session_id)
}

async fn create_session(
    db: &sqlx::PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    first_message: &str,
) -> Result<Uuid, ApiError> {
    let title = first_message.chars().take(60).collect::<String>();

    let row = query(
        r#"
        INSERT INTO ai_session (workspace_id, user_id, title)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind(title)
    .fetch_one(db)
    .await?;

    Ok(row.try_get("id")?)
}

async fn load_messages(db: &sqlx::PgPool, session_id: Uuid) -> Result<Vec<Message>, ApiError> {
    let system = Message::system(SYSTEM_PROMPT);

    let rows = query(
        r#"
        SELECT role, content, tool_calls, tool_call_id
        FROM ai_message
        WHERE session_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(db)
    .await?;

    let mut messages = vec![system];

    for row in rows {
        let role: String = row.try_get("role")?;
        let content: Option<String> = row.try_get("content")?;
        let tool_calls: Option<Value> = row.try_get("tool_calls")?;
        let tool_call_id: Option<String> = row.try_get("tool_call_id")?;

        let message = match role.as_str() {
            "user" => Message::user(content.unwrap_or_default()),
            "assistant" => {
                if let Some(tool_calls_value) = tool_calls {
                    let tool_calls: Vec<ToolCall> =
                        serde_json::from_value(tool_calls_value).unwrap_or_default();
                    Message::assistant_with_tool_calls(tool_calls)
                } else {
                    Message::assistant(content.unwrap_or_default())
                }
            }
            "tool" => Message::tool(
                tool_call_id.unwrap_or_default(),
                content.unwrap_or_default(),
            ),
            _ => Message::assistant(content.unwrap_or_default()),
        };

        messages.push(message);
    }

    Ok(messages)
}

async fn save_assistant_message(
    db: &sqlx::PgPool,
    session_id: Uuid,
    message: &Message,
) -> Result<(), ApiError> {
    query(
        r#"
        INSERT INTO ai_message (session_id, role, content, tool_calls)
        VALUES ($1, 'assistant', $2, $3)
        "#,
    )
    .bind(session_id)
    .bind(&message.content)
    .bind(
        &message
            .tool_calls
            .as_ref()
            .map(|tc| serde_json::to_value(tc).unwrap_or(Value::Null)),
    )
    .execute(db)
    .await?;

    query(
        r#"
        UPDATE ai_session
        SET updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .execute(db)
    .await?;

    Ok(())
}

fn map_ai_error(err: AiError) -> ApiError {
    match err {
        AiError::NotConfigured => ApiError::BadRequest(err.to_string()),
        _ => ApiError::Internal,
    }
}

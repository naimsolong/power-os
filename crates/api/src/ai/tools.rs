// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use serde_json::{json, Value};
use sqlx::{query, PgPool, Row};
use tracing::info;
use uuid::Uuid;

use crate::ai::client::{FunctionDefinition, ToolDefinition};
use crate::routes::error::ApiError;

pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "list_parties".to_string(),
                description: "List contacts and companies in the workspace.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "search": {
                            "type": "string",
                            "description": "Optional name search term"
                        },
                        "party_type": {
                            "type": "string",
                            "enum": ["customer", "vendor", "other"],
                            "description": "Optional filter by party type"
                        }
                    }
                }),
            },
        },
        ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "list_deals".to_string(),
                description: "List CRM deals in the workspace.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "search": {
                            "type": "string",
                            "description": "Optional search term"
                        }
                    }
                }),
            },
        },
        ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "list_invoices".to_string(),
                description: "List invoices in the workspace.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["draft", "sent", "paid", "overdue", "posted"],
                            "description": "Optional filter by invoice status"
                        }
                    }
                }),
            },
        },
        ToolDefinition {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "list_employees".to_string(),
                description: "List employees in the workspace.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["active", "inactive"],
                            "description": "Optional filter by employee status"
                        }
                    }
                }),
            },
        },
    ]
}

pub async fn execute_tool(
    db: &PgPool,
    workspace_id: Uuid,
    name: &str,
    arguments: &Value,
) -> Result<Value, ApiError> {
    info!(tool_name = name, workspace_id = %workspace_id, "executing AI tool");

    match name {
        "list_parties" => list_parties(db, workspace_id, arguments).await,
        "list_deals" => list_deals(db, workspace_id, arguments).await,
        "list_invoices" => list_invoices(db, workspace_id, arguments).await,
        "list_employees" => list_employees(db, workspace_id, arguments).await,
        _ => Err(ApiError::BadRequest(format!("Unknown tool: {name}"))),
    }
}

async fn list_parties(
    db: &PgPool,
    workspace_id: Uuid,
    arguments: &Value,
) -> Result<Value, ApiError> {
    let search = arguments["search"].as_str();
    let party_type = arguments["party_type"].as_str();

    let rows = query(
        r#"
        SELECT id, name, email, phone, address, tin, party_type
        FROM party
        WHERE workspace_id = $1
          AND ($2::text IS NULL OR party_type = $2)
          AND ($3::text IS NULL OR name ILIKE '%' || $3 || '%')
        ORDER BY name
        LIMIT 50
        "#,
    )
    .bind(workspace_id)
    .bind(party_type)
    .bind(search)
    .fetch_all(db)
    .await?;

    let parties: Vec<Value> = rows
        .iter()
        .map(|row| -> Result<Value, sqlx::Error> {
            Ok(json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "name": row.try_get::<String, _>("name")?,
                "email": row.try_get::<Option<String>, _>("email")?,
                "phone": row.try_get::<Option<String>, _>("phone")?,
                "party_type": row.try_get::<String, _>("party_type")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    Ok(json!({ "parties": parties }))
}

async fn list_deals(
    db: &PgPool,
    workspace_id: Uuid,
    arguments: &Value,
) -> Result<Value, ApiError> {
    let search = arguments["search"].as_str();

    let rows = query(
        r#"
        SELECT d.id, d.title, d.value, d.currency, d.stage, d.status,
               p.name AS party_name
        FROM deal d
        JOIN party p ON p.id = d.party_id
        WHERE d.workspace_id = $1
          AND ($2::text IS NULL OR d.title ILIKE '%' || $2 || '%' OR p.name ILIKE '%' || $2 || '%')
        ORDER BY d.updated_at DESC
        LIMIT 50
        "#,
    )
    .bind(workspace_id)
    .bind(search)
    .fetch_all(db)
    .await?;

    let deals: Vec<Value> = rows
        .iter()
        .map(|row| -> Result<Value, sqlx::Error> {
            Ok(json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "title": row.try_get::<String, _>("title")?,
                "value": row.try_get::<Option<String>, _>("value")?,
                "currency": row.try_get::<String, _>("currency")?,
                "stage": row.try_get::<String, _>("stage")?,
                "status": row.try_get::<String, _>("status")?,
                "party_name": row.try_get::<String, _>("party_name")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    Ok(json!({ "deals": deals }))
}

async fn list_invoices(
    db: &PgPool,
    workspace_id: Uuid,
    arguments: &Value,
) -> Result<Value, ApiError> {
    let status = arguments["status"].as_str();

    let rows = query(
        r#"
        SELECT i.id, i.invoice_number, i.issue_date, i.due_date,
               i.status, i.total_amount, i.currency,
               p.name AS party_name
        FROM invoice i
        JOIN party p ON p.id = i.party_id
        WHERE i.workspace_id = $1
          AND ($2::text IS NULL OR i.status = $2)
        ORDER BY i.issue_date DESC
        LIMIT 50
        "#,
    )
    .bind(workspace_id)
    .bind(status)
    .fetch_all(db)
    .await?;

    let invoices: Vec<Value> = rows
        .iter()
        .map(|row| -> Result<Value, sqlx::Error> {
            Ok(json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "invoice_number": row.try_get::<String, _>("invoice_number")?,
                "issue_date": row.try_get::<Option<String>, _>("issue_date")?,
                "due_date": row.try_get::<Option<String>, _>("due_date")?,
                "status": row.try_get::<String, _>("status")?,
                "total_amount": row.try_get::<String, _>("total_amount")?,
                "currency": row.try_get::<String, _>("currency")?,
                "party_name": row.try_get::<String, _>("party_name")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    Ok(json!({ "invoices": invoices }))
}

async fn list_employees(
    db: &PgPool,
    workspace_id: Uuid,
    arguments: &Value,
) -> Result<Value, ApiError> {
    let status = arguments["status"].as_str();

    let rows = query(
        r#"
        SELECT e.id, e.employee_code, e.job_title, e.department,
               e.hire_date, e.status, p.name
        FROM employee e
        JOIN party p ON p.id = e.party_id
        WHERE e.workspace_id = $1
          AND ($2::text IS NULL OR e.status = $2)
        ORDER BY p.name
        LIMIT 50
        "#,
    )
    .bind(workspace_id)
    .bind(status)
    .fetch_all(db)
    .await?;

    let employees: Vec<Value> = rows
        .iter()
        .map(|row| -> Result<Value, sqlx::Error> {
            Ok(json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "employee_code": row.try_get::<String, _>("employee_code")?,
                "name": row.try_get::<String, _>("name")?,
                "job_title": row.try_get::<Option<String>, _>("job_title")?,
                "department": row.try_get::<Option<String>, _>("department")?,
                "hire_date": row.try_get::<Option<String>, _>("hire_date")?,
                "status": row.try_get::<String, _>("status")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    Ok(json!({ "employees": employees }))
}

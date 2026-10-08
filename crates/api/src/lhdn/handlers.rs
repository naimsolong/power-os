// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use base64::Engine;
use power_os_domain::InvoiceId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Row};
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::lhdn::client::{DocumentStatusResponse, LhdnClient, LhdnError, LhdnSettings, SubmitPayload};
use crate::lhdn::ubl::{build_ubl_json, BuyerInfo, SupplierInfo};
use crate::routes::error::ApiError;
use crate::routes::invoices::fetch_invoice_detail;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateLhdnSettingsRequest {
    pub lhdn_client_id: Option<String>,
    pub lhdn_client_secret: Option<String>,
    pub lhdn_tin: Option<String>,
    pub lhdn_sandbox: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct LhdnSettingsResponse {
    pub lhdn_client_id: Option<String>,
    pub lhdn_tin: Option<String>,
    pub lhdn_sandbox: bool,
    pub lhdn_base_url: String,
}

#[derive(Debug, Serialize)]
pub struct LhdnSubmissionResponse {
    pub id: Uuid,
    pub invoice_id: InvoiceId,
    pub status: Option<String>,
    pub lhdn_uuid: Option<String>,
    pub lhdn_submission_uid: Option<String>,
    pub error_message: Option<String>,
    pub response_json: Option<Value>,
    pub submitted_at: Option<OffsetDateTime>,
    pub polled_at: Option<OffsetDateTime>,
}

pub async fn get_lhdn_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<LhdnSettingsResponse>), ApiError> {
    let row = query(
        r#"
        SELECT lhdn_client_id, lhdn_tin, lhdn_sandbox, lhdn_base_url
        FROM workspace
        WHERE id = $1
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(LhdnSettingsResponse {
            lhdn_client_id: row.try_get("lhdn_client_id")?,
            lhdn_tin: row.try_get("lhdn_tin")?,
            lhdn_sandbox: row.try_get("lhdn_sandbox")?,
            lhdn_base_url: row.try_get("lhdn_base_url")?,
        }),
    ))
}

pub async fn update_lhdn_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateLhdnSettingsRequest>,
) -> Result<(StatusCode, Json<LhdnSettingsResponse>), ApiError> {
    let sandbox = payload.lhdn_sandbox.unwrap_or(true);
    let base_url = if sandbox {
        "https://preprod-sdk.myinvois.hasil.gov.my"
    } else {
        "https://sdk.myinvois.hasil.gov.my"
    }
    .to_string();

    let row = query(
        r#"
        UPDATE workspace
        SET
            lhdn_client_id = COALESCE($2, lhdn_client_id),
            lhdn_client_secret = COALESCE($3, lhdn_client_secret),
            lhdn_tin = COALESCE($4, lhdn_tin),
            lhdn_sandbox = $5,
            lhdn_base_url = $6,
            updated_at = now()
        WHERE id = $1
        RETURNING lhdn_client_id, lhdn_tin, lhdn_sandbox, lhdn_base_url
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(&payload.lhdn_client_id)
    .bind(&payload.lhdn_client_secret)
    .bind(&payload.lhdn_tin)
    .bind(sandbox)
    .bind(&base_url)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(LhdnSettingsResponse {
            lhdn_client_id: row.try_get("lhdn_client_id")?,
            lhdn_tin: row.try_get("lhdn_tin")?,
            lhdn_sandbox: row.try_get("lhdn_sandbox")?,
            lhdn_base_url: row.try_get("lhdn_base_url")?,
        }),
    ))
}

pub async fn submit_lhdn_invoice(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<LhdnSubmissionResponse>), ApiError> {
    let invoice = fetch_invoice_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let workspace_row = query(
        r#"
        SELECT name, lhdn_client_id, lhdn_client_secret, lhdn_tin, lhdn_sandbox, lhdn_base_url
        FROM workspace
        WHERE id = $1
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_one(&state.db)
    .await?;

    let base_url: String = workspace_row
        .try_get("lhdn_base_url")
        .ok()
        .unwrap_or_else(|| {
            if workspace_row.try_get::<bool, _>("lhdn_sandbox").unwrap_or(true) {
                "https://preprod-sdk.myinvois.hasil.gov.my".to_string()
            } else {
                "https://sdk.myinvois.hasil.gov.my".to_string()
            }
        });

    let client_id: Option<String> = workspace_row.try_get("lhdn_client_id")?;
    let client_secret: Option<String> = workspace_row.try_get("lhdn_client_secret")?;
    let supplier_tin: Option<String> = workspace_row.try_get("lhdn_tin")?;
    let supplier_name: String = workspace_row.try_get("name")?;

    if client_id.is_none() || client_secret.is_none() {
        return Err(ApiError::BadRequest(
            "LHDN client credentials are not configured".to_string(),
        ));
    }
    let supplier_tin = supplier_tin.ok_or_else(|| {
        ApiError::BadRequest("Workspace LHDN TIN is not configured".to_string())
    })?;

    let party_row = query(
        "SELECT name, tin FROM party WHERE id = $1 AND workspace_id = $2",
    )
    .bind(invoice.party_id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_one(&state.db)
    .await
    .map_err(|_| ApiError::NotFound)?;

    let buyer_name: String = party_row.try_get("name")?;
    let buyer_tin: Option<String> = party_row.try_get("tin")?;

    let ubl = build_ubl_json(
        &invoice,
        SupplierInfo {
            name: &supplier_name,
            tin: &supplier_tin,
        },
        BuyerInfo {
            name: &buyer_name,
            tin: buyer_tin.as_deref(),
        },
    );

    let ubl_json_string = ubl.to_string();
    let document_base64 = base64::engine::general_purpose::STANDARD.encode(ubl_json_string.as_bytes());

    let request_json = serde_json::to_value(SubmitPayload {
        documents: vec![crate::lhdn::client::DocumentWrapper {
            format: "JSON".to_string(),
            document: document_base64.clone(),
        }],
    })
    .unwrap_or(Value::Null);

    info!(
        invoice_id = %id.0,
        workspace_id = %auth_user.workspace_id.0,
        "submitting invoice to LHDN"
    );

    let settings = LhdnSettings {
        base_url,
        client_id: client_id.unwrap(),
        client_secret: client_secret.unwrap(),
    };

    let submission_result = state
        .lhdn
        .submit_document(
            &settings,
            SubmitPayload {
                documents: vec![crate::lhdn::client::DocumentWrapper {
                    format: "JSON".to_string(),
                    document: document_base64,
                }],
            },
        )
        .await;

    match submission_result {
        Ok(response) => {
            let response_json = serde_json::to_value(&response).unwrap_or(Value::Null);
            let row = query(
                r#"
                INSERT INTO e_invoice_submission (
                    id, workspace_id, invoice_id, lhdn_uuid, lhdn_submission_uid,
                    status, request_json, response_json, submitted_at
                )
                VALUES ($1, $2, $3, $4, $5, 'submitted', $6, $7, now())
                RETURNING id, invoice_id, status, lhdn_uuid, lhdn_submission_uid,
                          error_message, response_json, submitted_at, polled_at
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(auth_user.workspace_id.0)
            .bind(id.0)
            .bind(response.uuid.as_deref())
            .bind(response.submission_uid.as_deref())
            .bind(&request_json)
            .bind(&response_json)
            .fetch_one(&state.db)
            .await?;

            Ok((StatusCode::OK, Json(map_submission_row(&row)?)))
        }
        Err(err) => {
            error!("LHDN submission failed for invoice {}: {}", id.0, err);
            let (error_message, response_json) = match err {
                LhdnError::Api { status, body } => {
                    let value = serde_json::from_str(&body).unwrap_or_else(|_| {
                        serde_json::json!({ "status": status.as_u16(), "raw": body })
                    });
                    (Some(format!("LHDN API error {status}")), Some(value))
                }
                LhdnError::Token(msg) => (Some(format!("LHDN authentication error: {msg}")), None),
                _ => (Some(err.to_string()), None),
            };

            let row = query(
                r#"
                INSERT INTO e_invoice_submission (
                    id, workspace_id, invoice_id, status, request_json,
                    response_json, error_message, submitted_at
                )
                VALUES ($1, $2, $3, 'error', $4, $5, $6, now())
                RETURNING id, invoice_id, status, lhdn_uuid, lhdn_submission_uid,
                          error_message, response_json, submitted_at, polled_at
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(auth_user.workspace_id.0)
            .bind(id.0)
            .bind(&request_json)
            .bind(&response_json)
            .bind(error_message.as_deref())
            .fetch_one(&state.db)
            .await?;

            Ok((StatusCode::OK, Json(map_submission_row(&row)?)))
        }
    }
}

pub async fn get_lhdn_status(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<LhdnSubmissionResponse>), ApiError> {
    let row = query(
        r#"
        SELECT id, invoice_id, status, lhdn_uuid, lhdn_submission_uid,
               error_message, response_json, submitted_at, polled_at
        FROM e_invoice_submission
        WHERE workspace_id = $1 AND invoice_id = $2
        ORDER BY submitted_at DESC NULLS LAST, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(id.0)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(ApiError::NotFound);
    };

    let mut submission = map_submission_row(&row)?;

    if let Some(uuid) = submission.lhdn_uuid.as_deref() {
        if let Ok(settings) = load_lhdn_settings(&state.db, auth_user.workspace_id.0).await {
            match state.lhdn.get_document_status(&settings, uuid).await {
                Ok(DocumentStatusResponse {
                    uuid: _,
                    status: Some(status),
                    extra,
                }) => {
                    let updated = query(
                        r#"
                        UPDATE e_invoice_submission
                        SET status = $3, response_json = $4, polled_at = now(), updated_at = now()
                        WHERE id = $1 AND workspace_id = $2
                        RETURNING id, invoice_id, status, lhdn_uuid, lhdn_submission_uid,
                                  error_message, response_json, submitted_at, polled_at
                        "#,
                    )
                    .bind(submission.id)
                    .bind(auth_user.workspace_id.0)
                    .bind(&status)
                    .bind(serde_json::to_value(&extra).unwrap_or(Value::Null))
                    .fetch_one(&state.db)
                    .await?;
                    submission = map_submission_row(&updated)?;
                }
                Ok(DocumentStatusResponse { status: None, .. }) => {
                    // Status endpoint returned without a status field; keep cached record.
                }
                Err(err) => {
                    error!("LHDN status poll failed for {}: {}", uuid, err);
                }
            }
        }
    }

    Ok((StatusCode::OK, Json(submission)))
}

async fn load_lhdn_settings(
    db: &sqlx::PgPool,
    workspace_id: Uuid,
) -> Result<LhdnSettings, ApiError> {
    let row = query(
        "SELECT lhdn_client_id, lhdn_client_secret, lhdn_base_url FROM workspace WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_one(db)
    .await?;

    let client_id: Option<String> = row.try_get("lhdn_client_id")?;
    let client_secret: Option<String> = row.try_get("lhdn_client_secret")?;
    let base_url: Option<String> = row.try_get("lhdn_base_url")?;

    match (client_id, client_secret, base_url) {
        (Some(client_id), Some(client_secret), Some(base_url)) => Ok(LhdnSettings {
            base_url,
            client_id,
            client_secret,
        }),
        _ => Err(ApiError::BadRequest(
            "LHDN credentials are not configured".to_string(),
        )),
    }
}

fn map_submission_row(row: &sqlx::postgres::PgRow) -> Result<LhdnSubmissionResponse, sqlx::Error> {
    Ok(LhdnSubmissionResponse {
        id: row.try_get("id")?,
        invoice_id: InvoiceId(row.try_get("invoice_id")?),
        status: row.try_get("status")?,
        lhdn_uuid: row.try_get("lhdn_uuid")?,
        lhdn_submission_uid: row.try_get("lhdn_submission_uid")?,
        error_message: row.try_get("error_message")?,
        response_json: row.try_get("response_json")?,
        submitted_at: row.try_get("submitted_at")?,
        polled_at: row.try_get("polled_at")?,
    })
}

// This import is used by the Axum router module; keep it visible.
#[allow(dead_code)]
pub fn _lhdn_client_ref(client: &LhdnClient) -> &LhdnClient {
    client
}

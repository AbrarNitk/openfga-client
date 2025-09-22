use axum::{Json, extract::State, http::StatusCode};
use openfga_http_client::apis::relationship_tuples_api;
use openfga_http_client::models::{ReadRequest, WriteRequest};
use serde_json::Value;

use crate::context::Ctx;

#[derive(Debug, serde::Deserialize)]
pub struct WriteTupleRequest {
    pub store_id: String,
    pub write_request: WriteRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct ReadTupleRequest {
    pub store_id: String,
    pub read_request: ReadRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteTupleRequest {
    pub store_id: String,
    pub write_request: WriteRequest, // Uses WriteRequest with deletes field
}

#[derive(Debug, serde::Deserialize)]
pub struct TupleChangesRequest {
    pub store_id: String,
    pub r#type: Option<String>,
    pub page_size: Option<i32>,
    pub continuation_token: Option<String>,
    pub start_time: Option<String>,
}

/// Write tuples using HTTP client
pub async fn write_tuple(
    State(ctx): State<Ctx>,
    Json(req): Json<WriteTupleRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_tuples_api::write(&ctx.fga_http_config, &req.store_id, req.write_request)
        .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to write tuple via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Read tuples using HTTP client
pub async fn read_tuple(
    State(ctx): State<Ctx>,
    Json(req): Json<ReadTupleRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_tuples_api::read(&ctx.fga_http_config, &req.store_id, req.read_request).await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to read tuple via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Delete tuples using HTTP client
pub async fn delete_tuple(
    State(ctx): State<Ctx>,
    Json(req): Json<DeleteTupleRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_tuples_api::write(&ctx.fga_http_config, &req.store_id, req.write_request)
        .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to delete tuple via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Get tuple changes using HTTP client
pub async fn tuple_changes(
    State(ctx): State<Ctx>,
    Json(req): Json<TupleChangesRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_tuples_api::read_changes(
        &ctx.fga_http_config,
        &req.store_id,
        req.r#type.clone().as_deref(),
        req.page_size,
        req.continuation_token.as_deref(),
        req.start_time,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to get tuple changes via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

use axum::{Json, extract::State, http::StatusCode};
use openfga_http_client::apis::relationship_queries_api;
use openfga_http_client::models::{
    BatchCheckRequest, CheckRequest, ExpandRequest, ListObjectsRequest, ListUsersRequest,
};
use serde_json::Value;

use crate::context::Ctx;

#[derive(Debug, serde::Deserialize)]
pub struct CheckReq {
    pub store_id: String,
    pub check_request: CheckRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchCheckReq {
    pub store_id: String,
    pub batch_check_request: BatchCheckRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct ExpandReq {
    pub store_id: String,
    pub expand_request: ExpandRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct ListObjectsReq {
    pub store_id: String,
    pub list_objects_request: ListObjectsRequest,
}

#[derive(Debug, serde::Deserialize)]
pub struct ListUsersReq {
    pub store_id: String,
    pub list_users_request: ListUsersRequest,
}

/// Check authorization using HTTP client
pub async fn check(
    State(ctx): State<Ctx>,
    Json(req): Json<CheckReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_queries_api::check(&ctx.fga_http_config, &req.store_id, req.check_request)
        .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to check authorization via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Batch check authorization using HTTP client
pub async fn batch_check(
    State(ctx): State<Ctx>,
    Json(req): Json<BatchCheckReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_queries_api::batch_check(
        &ctx.fga_http_config,
        &req.store_id,
        req.batch_check_request,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to batch check authorization via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Expand relationships using HTTP client
pub async fn expand(
    State(ctx): State<Ctx>,
    Json(req): Json<ExpandReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_queries_api::expand(&ctx.fga_http_config, &req.store_id, req.expand_request)
        .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to expand relationships via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// List objects using HTTP client
pub async fn list_objects(
    State(ctx): State<Ctx>,
    Json(req): Json<ListObjectsReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_queries_api::list_objects(
        &ctx.fga_http_config,
        &req.store_id,
        req.list_objects_request,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to list objects via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// List users using HTTP client
pub async fn list_users(
    State(ctx): State<Ctx>,
    Json(req): Json<ListUsersReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match relationship_queries_api::list_users(
        &ctx.fga_http_config,
        &req.store_id,
        req.list_users_request,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to list users via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

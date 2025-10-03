use axum::{Json, extract::Path, extract::State, http::StatusCode};
use openfga_http_client::apis::stores_api;
use openfga_http_client::models::CreateStoreRequest;
use serde_json::Value;

use crate::context::Ctx;

/// Create a new store using HTTP client
pub async fn create_store(
    State(ctx): State<Ctx>,
    Json(req): Json<CreateStoreRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match stores_api::create_store(&ctx.fga_http_config, req).await {
        Ok(response) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to create store via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Get a store by ID using HTTP client
pub async fn get_store(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match stores_api::get_store(&ctx.fga_http_config, &store_id).await {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to get store via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// List all stores using HTTP client
pub async fn list_stores(
    State(ctx): State<Ctx>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match stores_api::list_stores(&ctx.fga_http_config, None, None, None).await {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to list stores via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Delete a store by ID using HTTP client
pub async fn delete_store(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match stores_api::delete_store(&ctx.fga_http_config, &store_id).await {
        Ok(_) => Ok((
            StatusCode::NO_CONTENT,
            Json(serde_json::json!({ "message": "Store deleted successfully" })),
        )),
        Err(e) => {
            tracing::error!("Failed to delete store via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

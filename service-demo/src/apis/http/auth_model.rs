use axum::{Json, extract::Path, extract::State, http::StatusCode};
use openfga_http_client::apis::authorization_models_api;
use openfga_http_client::models::{AuthorizationModel, WriteAuthorizationModelRequest};
use serde_json::Value;

use crate::context::Ctx;

/// Create a new authorization model using HTTP client
pub async fn create_auth_model(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
    Json(req): Json<WriteAuthorizationModelRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match authorization_models_api::write_authorization_model(&ctx.fga_http_config, &store_id, req)
        .await
    {
        Ok(response) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to create authorization model via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// Create authorization model from JSON (convenience endpoint)
pub async fn create_auth_model_from_json(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
    Json(model): Json<AuthorizationModel>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Convert AuthorizationModel to WriteAuthorizationModelRequest
    let req = WriteAuthorizationModelRequest {
        type_definitions: model.type_definitions,
        schema_version: model.schema_version,
        conditions: model.conditions,
    };

    create_auth_model(State(ctx), Path(store_id), Json(req)).await
}

/// Get an authorization model by ID using HTTP client
pub async fn get_auth_model(
    State(ctx): State<Ctx>,
    Path((store_id, auth_model_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match authorization_models_api::read_authorization_model(
        &ctx.fga_http_config,
        &store_id,
        &auth_model_id,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to get authorization model via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

/// List authorization models using HTTP client
pub async fn list_auth_models(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match authorization_models_api::read_authorization_models(
        &ctx.fga_http_config,
        &store_id,
        None,
        None,
    )
    .await
    {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        )),
        Err(e) => {
            tracing::error!("Failed to list authorization models via HTTP: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ))
        }
    }
}

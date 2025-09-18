use std::collections::HashMap;

use crate::context::Ctx;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use openfga_client::{Condition, TypeDefinition, WriteAuthorizationModelRequest};
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
pub struct CreateAuthModelReq {
    pub type_definitions: Vec<TypeDefinition>,
    pub schema_version: Option<String>,
    pub conditions: Option<HashMap<String, Condition>>,
}

pub async fn create_auth_model(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
    Json(req): Json<CreateAuthModelReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!("Creating auth model for store: {}", store_id);
    let create_request = WriteAuthorizationModelRequest {
        store_id: store_id.clone(),
        type_definitions: req.type_definitions,
        schema_version: req.schema_version.unwrap_or_else(|| "1.1".to_string()),
        conditions: req.conditions.unwrap_or_else(|| HashMap::new()),
    };

    let create_response = match ctx
        .fga_client
        .clone()
        .write_authorization_model(create_request)
        .await
    {
        Ok(create_response) => create_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": e.to_string() })),
            ));
        }
    };

    tracing::info!("Auth model created for store: {}", store_id);

    Ok((
        StatusCode::OK,
        Json(
            serde_json::json!({ "message": "Auth model created", "create_response": create_response.into_inner() }),
        ),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateAuthModelReq {
    pub type_definitions: Vec<TypeDefinition>,
    pub schema_version: String,
}

pub async fn update_auth_model(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Auth model updated" })),
    ))
}

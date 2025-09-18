use std::collections::HashMap;

use crate::{context::Ctx, json_types::JsonAuthModel};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use openfga_client::{
    Condition, ReadAuthorizationModelRequest, ReadAuthorizationModelsRequest, TypeDefinition,
    WriteAuthorizationModelRequest,
};
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

// New endpoint that accepts JSON format from OpenFGA playground
pub async fn create_auth_model_from_json(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
    Json(json_model): Json<JsonAuthModel>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!("Creating auth model from JSON for store: {}", store_id);

    // Convert our JSON types to OpenFGA types
    let (type_definitions, schema_version, conditions) = match json_model.to_openfga_types() {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to convert JSON to OpenFGA types: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Conversion failed: {}", e) })),
            ));
        }
    };

    // Debug log the converted type definitions
    for type_def in &type_definitions {
        tracing::info!("Type: {}", type_def.r#type);
        if let Some(metadata) = &type_def.metadata {
            for (relation_name, relation_metadata) in &metadata.relations {
                tracing::info!("  Relation: {}", relation_name);
                for user_type in &relation_metadata.directly_related_user_types {
                    tracing::info!(
                        "    User type: {}, relation_or_wildcard: {:?}",
                        user_type.r#type,
                        user_type.relation_or_wildcard
                    );
                }
            }
        }
    }

    let create_request = WriteAuthorizationModelRequest {
        store_id: store_id.clone(),
        type_definitions,
        schema_version,
        conditions,
    };

    let create_response = match ctx
        .fga_client
        .clone()
        .write_authorization_model(create_request)
        .await
    {
        Ok(create_response) => create_response,
        Err(e) => {
            tracing::error!("Failed to create auth model: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    tracing::info!("Auth model created from JSON for store: {}", store_id);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Auth model created from JSON",
            "authorization_model_id": create_response.into_inner().authorization_model_id
        })),
    ))
}

pub async fn get_auth_model(
    State(ctx): State<Ctx>,
    Path((store_id, auth_model_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!("Getting auth model for store: {}", store_id);
    let get_request = ReadAuthorizationModelRequest {
        store_id: store_id.clone(),
        id: auth_model_id.clone(),
    };

    let get_response = match ctx
        .fga_client
        .clone()
        .read_authorization_model(get_request)
        .await
    {
        Ok(get_response) => get_response,
        Err(e) => {
            tracing::error!("Failed to get auth model: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(
            serde_json::json!({ "message": "Auth model fetched", "get_response": get_response.into_inner() }),
        ),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct ReadAuthorizationModelsQuery {
    pub page_size: Option<i32>,
    pub continuation_token: Option<String>,
}

pub async fn list_auth_models(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
    Query(query): Query<ReadAuthorizationModelsQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!("Listing auth models for store: {}", store_id);
    let list_request = ReadAuthorizationModelsRequest {
        store_id: store_id.clone(),
        page_size: query.page_size,
        continuation_token: query.continuation_token.unwrap_or_else(|| String::new()),
    };

    let list_response = match ctx
        .fga_client
        .clone()
        .read_authorization_models(list_request)
        .await
    {
        Ok(list_response) => list_response,
        Err(e) => {
            tracing::error!("Failed to list auth models: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(
            serde_json::json!({ "message": "Auth models listed", "list_response": list_response.into_inner() }),
        ),
    ))
}

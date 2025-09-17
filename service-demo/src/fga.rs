use axum::{Json, extract::State, http::StatusCode};
use openfga_client::{ListObjectsRequest, TupleKey, WriteRequest, WriteRequestWrites};
use serde_json::{Value, json};

use crate::context::Ctx;

pub async fn create_tuple(
    State(ctx): State<Ctx>,
    Json(tuple): Json<TupleKey>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let write_request = WriteRequest {
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        store_id: ctx.fga_config.store_id.clone(),
        deletes: None,
        writes: Some(WriteRequestWrites {
            tuple_keys: vec![tuple],
            on_duplicate: "ignore".to_string(),
        }),
    };

    let write_response = match ctx.fga_client.clone().write(write_request).await {
        Ok(write_response) => write_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Tuple created", "write_response": write_response.into_inner() })),
    ))
}

pub async fn list_tuples(
    State(ctx): State<Ctx>,
    Json(tuple): Json<TupleKey>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let list_request = ListObjectsRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        r#type: "resource".to_string(),
        relation: "viewer".to_string(),
        user: tuple.user.clone(),
        contextual_tuples: None,
        context: None,
        consistency: 8,
    };

    let list_response = match ctx.fga_client.clone().list_objects(list_request).await {
        Ok(list_response) => list_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Tuples listed", "list_response": list_response.into_inner() })),
    ))
}

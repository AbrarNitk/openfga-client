use axum::{Json, extract::State, http::StatusCode};
use openfga_client::{
    ConsistencyPreference, ReadRequest, ReadRequestTupleKey, TupleKey, WriteRequest,
    WriteRequestWrites,
};
use serde_json::{Value, json};

use crate::context::Ctx;

pub async fn write_tuple(
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

pub async fn read_tuple(
    State(ctx): State<Ctx>,
    Json(tuple): Json<ReadRequestTupleKey>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let read_request = ReadRequest {
        store_id: ctx.fga_config.store_id.clone(),
        tuple_key: Some(tuple),
        page_size: Some(100),
        continuation_token: String::new(),
        consistency: ConsistencyPreference::HigherConsistency as i32,
    };

    let read_response = match ctx.fga_client.clone().read(read_request).await {
        Ok(read_response) => read_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Tuple read", "read_response": read_response.into_inner() })),
    ))
}

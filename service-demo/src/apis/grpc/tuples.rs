use axum::{Json, extract::State, http::StatusCode};
use openfga_grpc_client::{
    ConsistencyPreference, ReadChangesRequest, ReadRequest, ReadRequestTupleKey, TupleKey,
    TupleKeyWithoutCondition, WriteRequest, WriteRequestDeletes, WriteRequestWrites,
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

pub async fn delete_tuple(
    State(ctx): State<Ctx>,
    Json(tuple): Json<TupleKeyWithoutCondition>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let delete_request = WriteRequest {
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        store_id: ctx.fga_config.store_id.clone(),
        deletes: Some(WriteRequestDeletes {
            tuple_keys: vec![tuple],
            on_missing: "error".to_string(),
        }),
        writes: None,
    };

    let delete_response = match ctx.fga_client.clone().write(delete_request).await {
        Ok(delete_response) => delete_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(
            json!({ "message": "Tuple deleted", "delete_response": delete_response.into_inner() }),
        ),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct TupleChangesRequest {
    pub r#type: String,
    pub page_size: Option<i32>,
    pub continuation_token: Option<String>,
    pub start_time: Option<Timestamp>,
}

pub async fn tuple_changes(
    State(ctx): State<Ctx>,
    Json(tuple): Json<TupleChangesRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let tuple_changes_request = ReadChangesRequest {
        store_id: ctx.fga_config.store_id.clone(),
        r#type: tuple.r#type,
        page_size: Some(100),
        continuation_token: String::new(),
        start_time: tuple
            .start_time
            .map(|timestamp| prost_wkt_types::Timestamp {
                seconds: timestamp.seconds,
                nanos: timestamp.nanos,
            }),
    };

    let tuple_changes_response = match ctx
        .fga_client
        .clone()
        .read_changes(tuple_changes_request)
        .await
    {
        Ok(tuple_changes_response) => tuple_changes_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(
            json!({ "message": "Tuple changes", "tuple_changes_response": tuple_changes_response.into_inner() }),
        ),
    ))
}

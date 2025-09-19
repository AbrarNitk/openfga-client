use axum::{Json, extract::State, http::StatusCode};
use openfga_client::{
    ConsistencyPreference, ListObjectsRequest, ListUsersRequest, Object, ReadRequest,
    ReadRequestTupleKey, UserTypeFilter,
};
use serde_json::{Value, json};

use crate::context::Ctx;

#[derive(Debug, serde::Deserialize)]
pub struct ListTuplesReq {
    pub relation: Option<String>,
    pub user: Option<String>,
    pub object: Option<String>,
}

pub async fn list_tuples(
    State(ctx): State<Ctx>,
    Json(tuple): Json<ListTuplesReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let list_request = ReadRequest {
        store_id: ctx.fga_config.store_id.clone(),
        tuple_key: Some(ReadRequestTupleKey {
            object: tuple.object.clone().unwrap_or_else(|| String::new()),
            relation: tuple.relation.clone().unwrap_or_else(|| String::new()),
            user: tuple.user.clone().unwrap_or_else(|| String::new()),
        }),
        page_size: None,
        continuation_token: String::new(),
        consistency: ConsistencyPreference::MinimizeLatency as i32,
    };

    let list_response = match ctx.fga_client.clone().read(list_request).await {
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

#[derive(Debug, serde::Deserialize)]
pub struct ListObjsRequest {
    pub r#type: String,
    pub relation: String,
    pub user: String,
}

pub async fn list_objects(
    State(ctx): State<Ctx>,
    Json(tuple): Json<ListObjsRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let list_request = ListObjectsRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        r#type: tuple.r#type.clone(),
        relation: tuple.relation.clone(),
        user: tuple.user.clone(),
        contextual_tuples: None,
        context: None,
        consistency: ConsistencyPreference::MinimizeLatency as i32,
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

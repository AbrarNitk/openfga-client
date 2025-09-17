use axum::{Json, extract::State, http::StatusCode};
use openfga_client::{
    ConsistencyPreference, ListObjectsRequest, ListUsersRequest, Object, ReadRequest,
    ReadRequestTupleKey, TupleKey, UserTypeFilter, WriteRequest, WriteRequestWrites,
};
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

#[derive(Debug, serde::Deserialize)]
pub struct UserTypeFilterReq {
    pub r#type: String,
    pub relation: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ObjectReq {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ListUsersReq {
    pub relation: String,
    pub user_filters: Vec<UserTypeFilterReq>,
    pub object: ObjectReq,
}

pub async fn list_users(
    State(ctx): State<Ctx>,
    Json(tuple): Json<ListUsersReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let list_request = ListUsersRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        relation: tuple.relation.clone(),
        object: Some(Object {
            r#type: tuple.object.r#type.clone(),
            id: tuple.object.id.clone(),
        }),
        user_filters: tuple
            .user_filters
            .into_iter()
            .map(|f| UserTypeFilter {
                r#type: f.r#type.clone(),
                relation: f.relation.clone(),
            })
            .collect(),
        contextual_tuples: vec![],
        context: None,
        consistency: ConsistencyPreference::MinimizeLatency as i32,
    };

    let list_response = match ctx.fga_client.clone().list_users(list_request).await {
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
        Json(json!({ "message": "Users listed", "list_response": list_response.into_inner() })),
    ))
}

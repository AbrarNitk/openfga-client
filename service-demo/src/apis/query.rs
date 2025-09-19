use axum::{Json, extract::State, http::StatusCode};
use openfga_client::{
    BatchCheckItem, BatchCheckRequest, CheckRequest, CheckRequestTupleKey, ConsistencyPreference,
    ExpandRequest, ExpandRequestTupleKey, ListUsersRequest,
};
use serde_json::Value;

use crate::context::Ctx;

#[derive(Debug, serde::Deserialize)]
pub struct CheckReq {
    pub user: String,
    pub object: String,
    pub relation: String,
}

pub async fn check(
    State(ctx): State<Ctx>,
    Json(req): Json<CheckReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let check_request = CheckRequest {
        store_id: ctx.fga_config.store_id.clone(),
        tuple_key: Some(CheckRequestTupleKey {
            user: req.user,
            object: req.object,
            relation: req.relation,
        }),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        consistency: ConsistencyPreference::HigherConsistency as i32,
        context: None,
        trace: false,
        contextual_tuples: None,
    };

    tracing::info!(
        "Checking if user has relation to object: {:?}",
        check_request
    );
    let check_response = match ctx.fga_client.clone().check(check_request).await {
        Ok(check_response) => check_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "check_response": check_response.into_inner() })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchCheckItemReq {
    tuple: CheckReq,
    id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchCheckReq {
    pub checks: Vec<BatchCheckItemReq>,
}

pub async fn batch_check(
    State(ctx): State<Ctx>,
    Json(req): Json<BatchCheckReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let batch_check_request = BatchCheckRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        consistency: ConsistencyPreference::HigherConsistency as i32,
        checks: req
            .checks
            .into_iter()
            .map(|check| BatchCheckItem {
                tuple_key: Some(CheckRequestTupleKey {
                    user: check.tuple.user,
                    object: check.tuple.object,
                    relation: check.tuple.relation,
                }),
                contextual_tuples: None,
                context: None,
                correlation_id: check.id,
            })
            .collect(),
    };

    let batch_check_response = match ctx
        .fga_client
        .clone()
        .batch_check(batch_check_request)
        .await
    {
        Ok(batch_check_response) => batch_check_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "batch_check_response": batch_check_response.into_inner() })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct ExpandReq {
    pub object: String,
    pub relation: String,
}

pub async fn expand(
    State(ctx): State<Ctx>,
    Json(req): Json<ExpandReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let expand_request = ExpandRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
        consistency: ConsistencyPreference::HigherConsistency as i32,
        contextual_tuples: None,
        tuple_key: Some(ExpandRequestTupleKey {
            object: req.object,
            relation: req.relation,
        }),
    };

    let expand_response = match ctx.fga_client.clone().expand(expand_request).await {
        Ok(expand_response) => expand_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "expand_response": expand_response.into_inner() })),
    ))
}

// List Users associated with an object for a type

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
        object: Some(openfga_client::Object {
            r#type: tuple.object.r#type.clone(),
            id: tuple.object.id.clone(),
        }),
        user_filters: tuple
            .user_filters
            .into_iter()
            .map(|f| openfga_client::UserTypeFilter {
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
                Json(serde_json::json!({ "message": e.to_string() })),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(
            serde_json::json!({ "message": "Users listed", "list_response": list_response.into_inner() }),
        ),
    ))
}

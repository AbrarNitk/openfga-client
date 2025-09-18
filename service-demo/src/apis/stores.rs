use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use openfga_client::{CreateStoreRequest, DeleteStoreRequest, GetStoreRequest, ListStoresRequest};
use serde_json::Value;

use crate::context::Ctx;

#[derive(Debug, serde::Deserialize)]
pub struct CreateStoreReq {
    pub name: String,
}

pub async fn create_store(
    State(ctx): State<Ctx>,
    Json(tuple): Json<CreateStoreReq>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let create_request = CreateStoreRequest {
        name: tuple.name.clone(),
    };

    let create_response = match ctx.fga_client.clone().create_store(create_request).await {
        Ok(create_response) => create_response,
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
            serde_json::json!({ "message": "Store created", "create_response": create_response.into_inner() }),
        ),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct GetStoreReq {
    pub store_id: String,
}

pub async fn get_store(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let get_request = GetStoreRequest { store_id: store_id };

    let get_response = match ctx.fga_client.clone().get_store(get_request).await {
        Ok(get_response) => get_response,
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
            serde_json::json!({ "message": "Store fetched", "get_response": get_response.into_inner() }),
        ),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListStoresQuery {
    pub page_size: Option<i32>,
    pub continuation_token: Option<String>,
    pub name: Option<String>,
}

pub async fn list_stores(
    State(ctx): State<Ctx>,
    Query(tuple): Query<ListStoresQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let list_request = ListStoresRequest {
        page_size: tuple.page_size,
        continuation_token: tuple.continuation_token.unwrap_or_else(|| String::new()),
        name: tuple.name.unwrap_or_else(|| String::new()),
    };

    let list_response = match ctx.fga_client.clone().list_stores(list_request).await {
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
            serde_json::json!({ "message": "Stores listed", "list_response": list_response.into_inner() }),
        ),
    ))
}

pub async fn delete_store(
    State(ctx): State<Ctx>,
    Path(store_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!("Deleting store: {}", store_id);
    let delete_request = DeleteStoreRequest {
        store_id: store_id.clone(),
    };

    let delete_response = match ctx.fga_client.clone().delete_store(delete_request).await {
        Ok(delete_response) => delete_response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": e.to_string() })),
            ));
        }
    };

    tracing::info!("Delete response: {:?}", delete_response);
    tracing::info!("Store deleted: {}", store_id);

    Ok((
        StatusCode::OK,
        Json(
            serde_json::json!({ "message": "Store deleted", "delete_response": delete_response.into_inner() }),
        ),
    ))
}

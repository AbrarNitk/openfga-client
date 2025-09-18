use axum::{Json, extract::State, http::StatusCode};
use openfga_client::CreateStoreRequest;
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

// pub async fn get_store(

//     State(ctx): State<Ctx>,
//     Json(tuple): Json<GetStoreReq>,
// ) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
//     let get_request = GetStoreRequest {
//         store_id: ctx.fga_config.store_id.clone(),
//     };
// }

// pub async fn list_stores(
//     State(ctx): State<Ctx>,
//     Json(tuple): Json<ListStoresReq>,
// ) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
//     let list_request = ListStoresRequest {
//         store_id: ctx.fga_config.store_id.clone(),
//         authorization_model_id: ctx.fga_config.authorization_model_id.clone(),
//     };
// }

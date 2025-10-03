use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::{Value, json};

pub fn routes<S: Send + Sync>(ctx: crate::context::Ctx) -> Router<S> {
    Router::new()
        .route("/auth", get(crate::auth::login::serve_login_template))
        .route("/auth/login-with", get(crate::auth::login::login_with))
        .with_state(ctx)
}

pub async fn login() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

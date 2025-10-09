use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::{Value, json};

pub fn routes<S: Send + Sync>(ctx: crate::context::Ctx) -> Router<S> {
    Router::new()
        .route("/auth", get(crate::auth::home::serve_login_template))
        .route("/auth/login-with", get(crate::auth::openid::login_with))
        .route(
            "/auth/callback",
            get(crate::auth::openid::handle_openid_callback),
        )
        .with_state(ctx)
}

pub fn routes_auth0<S: Send + Sync>(ctx: crate::context::Ctx) -> Router<S> {
    Router::new()
        .route("/auth/auth0", get(crate::auth::home::serve_login_template))
        .route("/auth/auth0/login", get(crate::auth::auth0::login_with))
        .route(
            "/auth/auth0/callback",
            get(crate::auth::auth0::handle_auth0_callback),
        )
        .with_state(ctx)
}

pub async fn login() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

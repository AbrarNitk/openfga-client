use crate::auth;
use crate::context::Ctx;
use crate::controller;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;

/// Create all routes for the application
pub fn create_routes<S: Send + Sync>(ctx: Arc<Ctx>) -> Router<S> {
    // Create protected routes that require authentication
    let protected_routes = Router::new()
        .route(
            "/api/resource/{service_name}/{service_type}/{org_id}/{name}",
            post(controller::create_resource)
                .put(controller::update_resource)
                .get(controller::get_resource)
                .delete(controller::delete_resource),
        )
        .route("/api/list-objects", get(controller::list_objects))
        .route(
            "/api/shared-resources",
            get(controller::get_shared_resources),
        )
        .route_layer(middleware::from_fn_with_state(
            ctx.clone(),
            auth::auth_middleware,
        ));

    // Create public routes that don't require authentication
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root));

    // Merge all routes
    public_routes.merge(protected_routes).with_state(ctx)
}

/// Health check endpoint
async fn health_check() -> (StatusCode, Json<Value>) {
    tracing::info!("Health check endpoint called");
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

/// Root endpoint
async fn root() -> (StatusCode, Json<Value>) {
    tracing::info!("Root endpoint called");
    (
        StatusCode::OK,
        Json(json!({ "message": "Welcome to OpenFGA Demo API" })),
    )
}

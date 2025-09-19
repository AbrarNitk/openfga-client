use crate::auth;
use crate::context::Ctx;
use crate::{apis, controller};
use axum::routing::delete;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use serde_json::{Value, json};

/// Create all routes for the application
pub fn create_routes<S: Send + Sync>(ctx: Ctx) -> Router<S> {
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
        .route("/", get(root))
        // store APIs
        .route("/api/ofga/store", post(apis::stores::create_store))
        .route("/api/ofga/store/{store_id}", get(apis::stores::get_store))
        .route("/api/ofga/store", get(apis::stores::list_stores))
        .route(
            "/api/ofga/store/{store_id}",
            delete(apis::stores::delete_store),
        )
        // model APIs
        .route(
            "/api/ofga/model/{store_id}",
            post(apis::auth_model::create_auth_model),
        )
        .route(
            "/api/ofga/model-json/{store_id}",
            post(apis::auth_model::create_auth_model_from_json),
        )
        .route(
            "/api/ofga/model/{store_id}/{auth_model_id}",
            get(apis::auth_model::get_auth_model),
        )
        .route(
            "/api/ofga/model/{store_id}",
            get(apis::auth_model::list_auth_models),
        )
        // tuple APIs
        .route("/api/ofga/tuple-write", post(apis::tuples::write_tuple))
        .route("/api/ofga/tuple-read", post(apis::tuples::read_tuple))
        .route("/api/ofga/tuple-delete", post(apis::tuples::delete_tuple))
        .route("/api/ofga/tuple-changes", post(apis::tuples::tuple_changes))
        // tuple query APIs
        .route("/api/ofga/list-objs", get(apis::query::list_objects))
        .route("/api/ofga/list-users", get(apis::query::list_users))
        .route("/api/ofga/check", post(apis::query::check))
        .route("/api/ofga/batch-check", post(apis::query::batch_check))
        .route("/api/ofga/expand", post(apis::query::expand));

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

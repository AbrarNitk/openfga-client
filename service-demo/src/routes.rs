use crate::auth;
use crate::context::Ctx;
use crate::{controller, fga_apis};
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
        // =============================================================================
        // gRPC-based APIs (existing)
        // =============================================================================
        // store APIs (gRPC)
        .route(
            "/api/ofga/grpc/store",
            post(fga_apis::grpc::stores::create_store),
        )
        .route(
            "/api/ofga/grpc/store/{store_id}",
            get(fga_apis::grpc::stores::get_store),
        )
        .route(
            "/api/ofga/grpc/store",
            get(fga_apis::grpc::stores::list_stores),
        )
        .route(
            "/api/ofga/grpc/store/{store_id}",
            delete(fga_apis::grpc::stores::delete_store),
        )
        // model APIs (gRPC)
        .route(
            "/api/ofga/grpc/model/{store_id}",
            post(fga_apis::grpc::auth_model::create_auth_model),
        )
        .route(
            "/api/ofga/grpc/model-json/{store_id}",
            post(fga_apis::grpc::auth_model::create_auth_model_from_json),
        )
        .route(
            "/api/ofga/grpc/model/{store_id}/{auth_model_id}",
            get(fga_apis::grpc::auth_model::get_auth_model),
        )
        .route(
            "/api/ofga/grpc/model/{store_id}",
            get(fga_apis::grpc::auth_model::list_auth_models),
        )
        // tuple APIs (gRPC)
        .route(
            "/api/ofga/grpc/tuple-write",
            post(fga_apis::grpc::tuples::write_tuple),
        )
        .route(
            "/api/ofga/grpc/tuple-read",
            post(fga_apis::grpc::tuples::read_tuple),
        )
        .route(
            "/api/ofga/grpc/tuple-delete",
            post(fga_apis::grpc::tuples::delete_tuple),
        )
        .route(
            "/api/ofga/grpc/tuple-changes",
            post(fga_apis::grpc::tuples::tuple_changes),
        )
        // tuple query APIs (gRPC)
        .route(
            "/api/ofga/grpc/list-objs",
            get(fga_apis::grpc::query::list_objects),
        )
        .route(
            "/api/ofga/grpc/list-users",
            get(fga_apis::grpc::query::list_users),
        )
        .route("/api/ofga/grpc/check", post(fga_apis::grpc::query::check))
        .route(
            "/api/ofga/grpc/batch-check",
            post(fga_apis::grpc::query::batch_check),
        )
        .route("/api/ofga/grpc/expand", post(fga_apis::grpc::query::expand))
        // =============================================================================
        // HTTP-based APIs (new - following OpenFGA REST API standards)
        // =============================================================================
        // store APIs (HTTP) - following OpenFGA REST API paths
        .route(
            "/api/ofga/http/stores",
            post(fga_apis::http::stores::create_store),
        )
        .route(
            "/api/ofga/http/stores",
            get(fga_apis::http::stores::list_stores),
        )
        .route(
            "/api/ofga/http/stores/{store_id}",
            get(fga_apis::http::stores::get_store),
        )
        .route(
            "/api/ofga/http/stores/{store_id}",
            delete(fga_apis::http::stores::delete_store),
        )
        // authorization model APIs (HTTP)
        .route(
            "/api/ofga/http/stores/{store_id}/authorization-models",
            post(fga_apis::http::auth_model::create_auth_model),
        )
        .route(
            "/api/ofga/http/stores/{store_id}/authorization-models",
            get(fga_apis::http::auth_model::list_auth_models),
        )
        .route(
            "/api/ofga/http/stores/{store_id}/authorization-models/{auth_model_id}",
            get(fga_apis::http::auth_model::get_auth_model),
        )
        .route(
            "/api/ofga/http/stores/{store_id}/authorization-models/json",
            post(fga_apis::http::auth_model::create_auth_model_from_json),
        )
        // tuple APIs (HTTP)
        .route(
            "/api/ofga/http/write",
            post(fga_apis::http::tuples::write_tuple),
        )
        .route(
            "/api/ofga/http/read",
            post(fga_apis::http::tuples::read_tuple),
        )
        .route(
            "/api/ofga/http/delete",
            post(fga_apis::http::tuples::delete_tuple),
        )
        .route(
            "/api/ofga/http/changes",
            post(fga_apis::http::tuples::tuple_changes),
        )
        // relationship query APIs (HTTP)
        .route("/api/ofga/http/check", post(fga_apis::http::query::check))
        .route(
            "/api/ofga/http/batch-check",
            post(fga_apis::http::query::batch_check),
        )
        .route("/api/ofga/http/expand", post(fga_apis::http::query::expand))
        .route(
            "/api/ofga/http/list-objects",
            post(fga_apis::http::query::list_objects),
        )
        .route(
            "/api/ofga/http/list-users",
            post(fga_apis::http::query::list_users),
        );

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

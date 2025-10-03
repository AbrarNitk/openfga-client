use crate::context::Ctx;
use crate::fga_apis;
use axum::routing::delete;
use axum::{
    Router,
    routing::{get, post},
};

pub fn create_fga_routes<S: Send + Sync>(ctx: Ctx) -> Router<S> {
    Router::new() // =============================================================================
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
        )
        .with_state(ctx)
}

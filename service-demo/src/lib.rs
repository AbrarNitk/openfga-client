pub mod auth;
pub mod context;
pub mod controller;
pub mod fga_apis;
pub mod listener;
pub mod routes;

// Re-export json types from openfga-client for convenience
pub use openfga_grpc_client::{
    JsonAuthModel, JsonDirectlyRelatedUserType, JsonMetadata, JsonRelationMetadata,
    JsonTypeDefinition, JsonUserset,
};

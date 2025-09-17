use crate::auth::AuthUser;
use crate::context::Ctx;
use axum::{
    Extension,
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use openfga_client::{
    CheckRequest, CheckRequestTupleKey, ListObjectsRequest, TupleKeyWithoutCondition,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use tonic::Request;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub name: String,
    pub service_name: String,
    pub service_type: String,
    pub org_id: String,
    pub properties: Value,
}

#[derive(Debug, Deserialize)]
pub struct ResourceParams {
    pub service_name: String,
    pub service_type: String,
    pub org_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    pub relation: Option<String>,
    pub object_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub objects: Vec<String>,
    pub total_count: usize,
    pub object_type: String,
    pub relation: String,
}

#[derive(Debug, Serialize)]
pub struct SharedResourcesResponse {
    pub services: Vec<SharedService>,
    pub service_types: Vec<SharedServiceType>,
    pub resources: Vec<SharedResource>,
}

#[derive(Debug, Serialize)]
pub struct SharedService {
    pub id: String,
    pub name: String,
    pub shared_via: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SharedServiceType {
    pub id: String,
    pub service_name: String,
    pub service_type: String,
    pub shared_via: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SharedResource {
    pub id: String,
    pub service_name: String,
    pub service_type: String,
    pub resource_name: String,
    pub shared_via: String,
    pub permissions: Vec<String>,
}

/// Check if a user has the required permission for a resource
async fn check_permission(
    ctx: &Ctx,
    user_id: &str,
    relation: &str,
    object_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    tracing::info!(
        "Checking if user {} has {} permission on resource {}",
        user_id,
        relation,
        object_id
    );

    // Get store ID from context
    let store_id = &ctx.fga_config.store_id;
    if store_id.is_empty() {
        return Err("OpenFGA store ID not configured".into());
    }

    // Get authorization model ID from context
    let authorization_model_id = match &ctx.fga_config.authorization_model_id {
        Some(id) => id,
        None => return Err("OpenFGA authorization model ID not configured".into()),
    };

    // Get the OpenFGA client
    let mut service_client = ctx.fga_client.clone();

    // Create the tuple key for checking
    let tuple_key = TupleKeyWithoutCondition {
        user: format!("user:{}", user_id),
        relation: relation.to_string(),
        object: object_id.to_string(),
    };

    // Create a check request using tonic::Request
    let check_request = Request::new(CheckRequest {
        store_id: store_id.clone(),
        tuple_key: Some(CheckRequestTupleKey {
            user: tuple_key.user,
            relation: tuple_key.relation,
            object: tuple_key.object,
        }),
        authorization_model_id: authorization_model_id.clone(),
        ..Default::default()
    });

    // Perform the check
    match service_client.check(check_request).await {
        Ok(response) => {
            let allowed = response.into_inner().allowed;
            tracing::info!(
                "Permission check result for user {} on resource {}: {}",
                user_id,
                object_id,
                allowed
            );
            Ok(allowed)
        }
        Err(e) => {
            tracing::error!("Error checking permission with OpenFGA: {}", e);

            // Check if it's a transport error (connection issue)
            let error_msg = e.to_string();
            if error_msg.contains("transport error") || error_msg.contains("Connection refused") {
                tracing::error!("OpenFGA server appears to be unavailable. Please check:");
                tracing::error!("1. OpenFGA server is running");
                tracing::error!(
                    "2. OPENFGA_CLIENT_URL is correct (default: http://localhost:8081)"
                );
                tracing::error!("3. Network connectivity to OpenFGA server");
                return Err("OpenFGA server is not available. Please check server status and configuration.".into());
            }

            Err(format!("OpenFGA permission check failed: {}", e).into())
        }
    }
}

// Create a new resource
pub async fn create_resource(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
    Path(params): Path<ResourceParams>,
    Json(_payload): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!(
        "Creating resource: {}/{}/{}/{}",
        params.service_name,
        params.service_type,
        params.org_id,
        params.name
    );

    // let resource_key = format!(
    //     "{}/{}/{}/{}",
    //     params.service_name, params.service_type, params.org_id, params.name
    // );

    let org_key = format!("organisation:{}", params.org_id);

    // Get user ID from authentication middleware
    let user_id = &auth_user.user_id;

    // To create a resource, user needs to be an admin of the organization
    // In a real app, we would check if the user is an admin of the organization
    // For this example, we'll check if the user has admin permission on the resource
    match check_permission(&ctx, user_id, "admin", &org_key).await {
        Ok(allowed) => {
            if !allowed {
                tracing::warn!(
                    "User {} does not have admin permission for resource {}",
                    user_id,
                    org_key
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Permission denied",
                        "message": "You do not have permission to create this resource"
                    })),
                ));
            }

            tracing::info!(
                "User {} has admin permission for organisation {}",
                user_id,
                org_key
            );

            // In a real app, we would create the resource in the database

            Ok((
                StatusCode::CREATED,
                Json(json!({
                    "message": "Resource created successfully",
                    "organisation": params.org_id
                })),
            ))
        }
        Err(e) => {
            tracing::error!("Error checking permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check permission",
                    "message": e.to_string()
                })),
            ))
        }
    }
}

// Update an existing resource
pub async fn update_resource(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
    Path(params): Path<ResourceParams>,
    Json(_payload): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!(
        "Updating resource: {}/{}/{}/{}",
        params.service_name,
        params.service_type,
        params.org_id,
        params.name
    );

    let resource_key = format!(
        "{}/{}/{}/{}",
        params.service_name, params.service_type, params.org_id, params.name
    );

    // Get user ID from authentication middleware
    let user_id = &auth_user.user_id;

    // To update a resource, user needs to be an editor of the resource
    match check_permission(&ctx, user_id, "editor", &resource_key).await {
        Ok(allowed) => {
            if !allowed {
                tracing::warn!(
                    "User {} does not have editor permission for resource {}",
                    user_id,
                    resource_key
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Permission denied",
                        "message": "You do not have permission to update this resource"
                    })),
                ));
            }

            tracing::info!(
                "User {} has editor permission for resource {}",
                user_id,
                resource_key
            );

            // In a real app, we would update the resource in the database

            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Resource updated successfully",
                    "resource_id": resource_key
                })),
            ))
        }
        Err(e) => {
            tracing::error!("Error checking permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check permission",
                    "message": e.to_string()
                })),
            ))
        }
    }
}

// Get a resource
pub async fn get_resource(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
    Path(params): Path<ResourceParams>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!(
        "Getting resource: {}/{}/{}/{}",
        params.service_name,
        params.service_type,
        params.org_id,
        params.name
    );

    let resource_key = format!(
        "{}/{}/{}/{}",
        params.service_name, params.service_type, params.org_id, params.name
    );

    // Get user ID from authentication middleware
    let user_id = &auth_user.user_id;

    // Check if user has viewer permission on the resource
    match check_permission(&ctx, user_id, "viewer", &resource_key).await {
        Ok(allowed) => {
            if !allowed {
                tracing::warn!(
                    "User {} does not have viewer permission for resource {}",
                    user_id,
                    resource_key
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Permission denied",
                        "message": "You do not have permission to view this resource"
                    })),
                ));
            }

            tracing::info!(
                "User {} has viewer permission for resource {}",
                user_id,
                resource_key
            );

            Ok((
                StatusCode::OK,
                Json(json!({
                    "resource_id": resource_key,
                    "name": params.name,
                    "service_name": params.service_name,
                    "service_type": params.service_type,
                    "org_id": params.org_id
                })),
            ))
        }
        Err(e) => {
            tracing::error!("Error checking permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check permission",
                    "message": e.to_string()
                })),
            ))
        }
    }
}

/// List objects that a user has access to using OpenFGA ListObjects API
pub async fn list_objects(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ListQueryParams>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let user_id = &auth_user.user_id;
    let relation = params.relation.unwrap_or_else(|| "viewer".to_string());
    let object_type = params.object_type.unwrap_or_else(|| "resource".to_string());

    tracing::info!(
        "Listing {} objects for user {} with relation {}",
        object_type,
        user_id,
        relation
    );

    // Create ListObjects request
    let request = Request::new(ListObjectsRequest {
        store_id: ctx.fga_config.store_id.clone(),
        authorization_model_id: ctx
            .fga_config
            .authorization_model_id
            .clone()
            .unwrap_or_default(),
        r#type: object_type.clone(),
        consistency: 8,
        relation: relation.clone(),
        user: user_id.to_string(),
        contextual_tuples: None,
        context: None,
    });

    match ctx.fga_client.clone().list_objects(request).await {
        Ok(response) => {
            let objects = response.into_inner().objects;
            tracing::info!(
                "Found {} {} objects for user {}",
                objects.len(),
                object_type,
                user_id
            );

            Ok((
                StatusCode::OK,
                Json(json!(ListResponse {
                    total_count: objects.len(),
                    objects: objects,
                    object_type: object_type,
                    relation: relation,
                })),
            ))
        }
        Err(e) => {
            tracing::error!("Error listing objects: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to list objects",
                    "message": e.to_string()
                })),
            ))
        }
    }
}

/// Get shared resources from parent organizations (comprehensive approach)
pub async fn get_shared_resources(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let user_id = &auth_user.user_id;

    tracing::info!("Getting shared resources for user {}", user_id);

    let mut shared_services = Vec::new();
    let mut shared_service_types = Vec::new();
    let mut shared_resources = Vec::new();

    // List all object types that the user can view
    let object_types = vec!["service", "service_type", "resource"];
    let relations = vec!["viewer", "editor", "admin"];

    for object_type in object_types {
        for relation in &relations {
            let request = Request::new(ListObjectsRequest {
                store_id: ctx.fga_config.store_id.clone(),
                authorization_model_id: ctx
                    .fga_config
                    .authorization_model_id
                    .clone()
                    .unwrap_or_default(),
                r#type: object_type.to_string(),
                consistency: 8,
                relation: relation.to_string(),
                user: user_id.to_string(),
                contextual_tuples: None,
                context: None,
            });

            match ctx.fga_client.clone().list_objects(request).await {
                Ok(response) => {
                    let objects = response.into_inner().objects;

                    for object_id in objects {
                        match object_type {
                            "service" => {
                                if let Some(service_name) =
                                    object_id.clone().strip_prefix("service:")
                                {
                                    shared_services.push(SharedService {
                                        id: object_id,
                                        name: service_name.to_string(),
                                        shared_via: "parent_organization".to_string(),
                                        permissions: vec![relation.to_string()],
                                    });
                                }
                            }
                            "service_type" => {
                                if let Some(service_type_path) =
                                    object_id.clone().strip_prefix("service_type:")
                                {
                                    let parts: Vec<&str> = service_type_path.split('/').collect();
                                    if parts.len() == 2 {
                                        shared_service_types.push(SharedServiceType {
                                            id: object_id,
                                            service_name: parts[0].to_string(),
                                            service_type: parts[1].to_string(),
                                            shared_via: "parent_organization".to_string(),
                                            permissions: vec![relation.to_string()],
                                        });
                                    }
                                }
                            }
                            "resource" => {
                                if let Some(resource_path) =
                                    object_id.clone().strip_prefix("resource:")
                                {
                                    let parts: Vec<&str> = resource_path.split('/').collect();
                                    if parts.len() == 3 {
                                        shared_resources.push(SharedResource {
                                            id: object_id,
                                            service_name: parts[0].to_string(),
                                            service_type: parts[1].to_string(),
                                            resource_name: parts[2].to_string(),
                                            shared_via: "parent_organization".to_string(),
                                            permissions: vec![relation.to_string()],
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Error listing {} objects with relation {}: {}",
                        object_type,
                        relation,
                        e
                    );
                }
            }
        }
    }

    // Deduplicate and merge permissions
    let mut service_map: HashMap<String, SharedService> = HashMap::new();
    let mut service_type_map: HashMap<String, SharedServiceType> = HashMap::new();
    let mut resource_map: HashMap<String, SharedResource> = HashMap::new();

    for service in shared_services {
        service_map
            .entry(service.id.clone())
            .and_modify(|existing| {
                existing.permissions.extend(service.permissions.clone());
                existing.permissions.sort();
                existing.permissions.dedup();
            })
            .or_insert(service);
    }

    for service_type in shared_service_types {
        service_type_map
            .entry(service_type.id.clone())
            .and_modify(|existing| {
                existing
                    .permissions
                    .extend(service_type.permissions.clone());
                existing.permissions.sort();
                existing.permissions.dedup();
            })
            .or_insert(service_type);
    }

    for resource in shared_resources {
        resource_map
            .entry(resource.id.clone())
            .and_modify(|existing| {
                existing.permissions.extend(resource.permissions.clone());
                existing.permissions.sort();
                existing.permissions.dedup();
            })
            .or_insert(resource);
    }

    let response = SharedResourcesResponse {
        services: service_map.into_values().collect(),
        service_types: service_type_map.into_values().collect(),
        resources: resource_map.into_values().collect(),
    };

    Ok((StatusCode::OK, Json(json!(response))))
}

// Delete a resource
pub async fn delete_resource(
    State(ctx): State<Ctx>,
    Extension(auth_user): Extension<AuthUser>,
    Path(params): Path<ResourceParams>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    tracing::info!(
        "Deleting resource: {}/{}/{}/{}",
        params.service_name,
        params.service_type,
        params.org_id,
        params.name
    );

    let resource_key = format!(
        "{}/{}/{}/{}",
        params.service_name, params.service_type, params.org_id, params.name
    );

    // Get user ID from authentication middleware
    let user_id = &auth_user.user_id;

    // To delete a resource, user needs to be an owner of the resource
    match check_permission(&ctx, user_id, "owner", &resource_key).await {
        Ok(allowed) => {
            if !allowed {
                tracing::warn!(
                    "User {} does not have owner permission for resource {}",
                    user_id,
                    resource_key
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Permission denied",
                        "message": "You do not have permission to delete this resource"
                    })),
                ));
            }

            tracing::info!(
                "User {} has owner permission for resource {}",
                user_id,
                resource_key
            );

            // In a real app, we would delete the resource from the database

            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Resource deleted successfully",
                    "resource_id": resource_key
                })),
            ))
        }
        Err(e) => {
            tracing::error!("Error checking permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check permission",
                    "message": e.to_string()
                })),
            ))
        }
    }
}

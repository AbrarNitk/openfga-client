# OpenFGA Client Usage Examples

This document provides examples of how to use the OpenFGA client in the application.

## Basic Usage

```rust
// Get the OpenFGA client from the context
let fga_client = ctx.fga_client.clone();

// Create a store
let create_store_response = fga_client
    .create_store(openfga_client::api::CreateStoreRequest {
        name: "my-store".to_string(),
    })
    .await?;

let store_id = create_store_response.into_inner().id;
println!("Created store with ID: {}", store_id);

// Write an authorization model
let authorization_model = r#"{
  "type_definitions": [
    {
      "type": "user",
      "relations": {}
    },
    {
      "type": "document",
      "relations": {
        "reader": {
          "this": {}
        },
        "writer": {
          "this": {}
        },
        "owner": {
          "this": {}
        }
      }
    }
  ]
}"#;

let write_model_response = fga_client
    .write_authorization_model(openfga_client::api::WriteAuthorizationModelRequest {
        store_id: store_id.clone(),
        type_definitions: serde_json::from_str(authorization_model).unwrap(),
        schema_version: "1.1".to_string(),
    })
    .await?;

let model_id = write_model_response.into_inner().authorization_model_id;
println!("Created authorization model with ID: {}", model_id);

// Write tuples (relationships)
let write_response = fga_client
    .write(openfga_client::api::WriteRequest {
        store_id: store_id.clone(),
        writes: vec![openfga_client::api::TupleKey {
            user: "user:anne".to_string(),
            relation: "reader".to_string(),
            object: "document:doc1".to_string(),
        }],
        authorization_model_id: Some(model_id.clone()),
        ..Default::default()
    })
    .await?;

println!("Write response: {:?}", write_response);

// Check if a user has permission
let check_response = fga_client
    .check(openfga_client::api::CheckRequest {
        store_id: store_id.clone(),
        tuple_key: Some(openfga_client::api::TupleKey {
            user: "user:anne".to_string(),
            relation: "reader".to_string(),
            object: "document:doc1".to_string(),
        }),
        authorization_model_id: Some(model_id.clone()),
        ..Default::default()
    })
    .await?;

let allowed = check_response.into_inner().allowed;
println!("Is Anne allowed to read doc1? {}", allowed);
```

## Using the Client in Controllers

```rust
pub async fn check_permission(
    State(ctx): State<Arc<Ctx>>,
    Path(params): Path<ResourceParams>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let fga_client = ctx.fga_client.clone();
    
    // Get store ID and model ID from context
    let store_id = &ctx.fga_config.store_id;
    let model_id = ctx.fga_config.authorization_model_id.as_ref()
        .ok_or_else(|| {
            tracing::error!("Authorization model ID not set");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Authorization model not configured" })),
            )
        })?;
    
    // Check if user has permission
    let check_result = fga_client
        .check(openfga_client::api::CheckRequest {
            store_id: store_id.clone(),
            tuple_key: Some(openfga_client::api::TupleKey {
                user: format!("user:{}", params.name),
                relation: "reader".to_string(),
                object: format!("document:{}", params.service_name),
            }),
            authorization_model_id: Some(model_id.clone()),
            ..Default::default()
        })
        .await;
    
    match check_result {
        Ok(response) => {
            let allowed = response.into_inner().allowed;
            Ok((
                StatusCode::OK,
                Json(json!({
                    "allowed": allowed,
                })),
            ))
        },
        Err(e) => {
            tracing::error!("Error checking permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check permission",
                })),
            ))
        }
    }
}
```

## Environment Setup

Make sure to set the following environment variables:

```bash
# Application profile
export PROFILE=dev

# Database URL
export DATABASE_URL=postgres://user:password@localhost:5432/openfga_demo

# OpenFGA configuration
export OPENFGA_CLIENT_URL=http://localhost:8080
export OPENFGA_STORE_ID=01HBPC7QTJQPQGCM9MSCG1JM1P
export OPENFGA_AUTH_MODEL_ID=01HBPC7QTJQPQGCM9MSCG1JM1Q
```

You can also create a `.env` file in the project root with these variables.

## Helper Functions for OpenFGA

```rust
// Helper function to create a store if it doesn't exist
pub async fn ensure_store_exists(ctx: &Arc<Ctx>) -> Result<String, Box<dyn std::error::Error>> {
    let fga_client = ctx.fga_client.clone();
    
    // Check if store ID is already set
    if !ctx.fga_config.store_id.is_empty() {
        return Ok(ctx.fga_config.store_id.clone());
    }
    
    // Create a new store
    let create_store_response = fga_client
        .create_store(openfga_client::api::CreateStoreRequest {
            name: format!("openfga-demo-{}", ctx.profile),
        })
        .await?;
    
    let store_id = create_store_response.into_inner().id;
    tracing::info!("Created new OpenFGA store with ID: {}", store_id);
    
    Ok(store_id)
}

// Helper function to create an authorization model if it doesn't exist
pub async fn ensure_auth_model_exists(
    ctx: &Arc<Ctx>,
    store_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let fga_client = ctx.fga_client.clone();
    
    // Check if model ID is already set
    if let Some(model_id) = &ctx.fga_config.authorization_model_id {
        return Ok(model_id.clone());
    }
    
    // Create a new authorization model
    let authorization_model = r#"{
      "type_definitions": [
        {
          "type": "user",
          "relations": {}
        },
        {
          "type": "resource",
          "relations": {
            "reader": {
              "this": {}
            },
            "writer": {
              "this": {}
            },
            "owner": {
              "this": {}
            }
          }
        }
      ]
    }"#;
    
    let write_model_response = fga_client
        .write_authorization_model(openfga_client::api::WriteAuthorizationModelRequest {
            store_id: store_id.to_string(),
            type_definitions: serde_json::from_str(authorization_model).unwrap(),
            schema_version: "1.1".to_string(),
        })
        .await?;
    
    let model_id = write_model_response.into_inner().authorization_model_id;
    tracing::info!("Created new OpenFGA authorization model with ID: {}", model_id);
    
    Ok(model_id)
}
```

# OpenFGA Rust Client

A Rust client library for OpenFGA that provides both low-level gRPC access and high-level JSON-friendly APIs.

## Features

- **gRPC Client**: Direct access to all OpenFGA APIs using generated protobuf types
- **JSON Support**: Work with authorization models using familiar JSON structures from OpenFGA playground
- **Type Conversion**: Seamless conversion between JSON and protobuf types
- **High-level Wrappers**: Convenient methods for common operations

## Usage

### Basic gRPC Client

```rust
use openfga_client::OpenFGAClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OpenFGAClient::new("http://localhost:8080".to_string()).await?;
    
    // Use low-level gRPC methods
    let stores = client.list_stores(ListStoresRequest::default()).await?;
    println!("Stores: {:?}", stores);
    
    Ok(())
}
```

### JSON Authorization Models

```rust
use openfga_client::OpenFGAClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OpenFGAClient::new("http://localhost:8080".to_string()).await?;
    
    // Parse JSON authorization model (from OpenFGA playground)
    let json_content = r#"{
        "schema_version": "1.1",
        "type_definitions": [
            {
                "type": "user"
            },
            {
                "type": "document",
                "relations": {
                    "owner": {"this": {}},
                    "viewer": {
                        "union": {
                            "child": [
                                {"this": {}},
                                {"computedUserset": {"object": "", "relation": "owner"}}
                            ]
                        }
                    }
                }
            }
        ]
    }"#;
    
    // Write authorization model from JSON
    let response = client
        .write_authorization_model_from_json_string("store_id".to_string(), json_content)
        .await?;
    
    println!("Authorization model created: {:?}", response);
    
    Ok(())
}
```

### Working with JSON Types

```rust
use openfga_client::{JsonAuthModel, JsonTypeDefinition, JsonUserset};

// Parse JSON model
let json_model = OpenFGAClient::parse_authorization_model_from_json(json_content)?;

// Access type definitions
for type_def in &json_model.type_definitions {
    println!("Type: {}", type_def.type_name);
    
    for (relation_name, userset) in &type_def.relations {
        println!("  Relation: {}", relation_name);
        
        // Check userset type
        if userset.this.is_some() {
            println!("    -> Direct assignment");
        } else if let Some(union) = &userset.union {
            println!("    -> Union with {} children", union.child.len());
        }
    }
}

// Convert to protobuf types when needed
let (type_definitions, schema_version, conditions) = json_model.to_openfga_types()?;
```

## Available JSON Types

- `JsonAuthModel`: Complete authorization model
- `JsonTypeDefinition`: Type definition with relations and metadata
- `JsonUserset`: Relation definition (this, computedUserset, union, etc.)
- `JsonMetadata`: Type and relation metadata
- `JsonRelationMetadata`: Relation-specific metadata
- `JsonDirectlyRelatedUserType`: User type constraints

## API Methods

### JSON-Friendly Methods

- `write_authorization_model_from_json()`: Write model from JsonAuthModel
- `write_authorization_model_from_json_string()`: Write model from JSON string
- `parse_authorization_model_from_json()`: Parse JSON string to JsonAuthModel
- `authorization_model_to_json()`: Convert protobuf model to JSON

### Standard gRPC Methods

All standard OpenFGA gRPC methods are available:

- `read()`, `write()`, `check()`, `expand()`
- `read_authorization_model()`, `write_authorization_model()`, `read_authorization_models()`
- `get_store()`, `list_stores()`, `create_store()`, `delete_store()`
- `list_objects()`, `read_changes()`

## Examples

See the `examples/` directory for complete examples:

- `json_client_example.rs`: Working with JSON authorization models

## Development

This crate is part of a workspace. To run tests:

```bash
cargo test --package openfga-client
```

To run examples:

```bash
cargo run --package openfga-client --example json_client_example
```

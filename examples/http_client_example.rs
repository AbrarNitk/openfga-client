use serde_json::json;

/// Example demonstrating HTTP client usage
/// 
/// This example shows how to interact with the OpenFGA HTTP API
/// through the service-demo endpoints.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3000";

    println!("🚀 OpenFGA HTTP Client Example");
    println!("================================");

    // Example 1: Create a store
    println!("\n📦 1. Creating a store...");
    let create_store_response = client
        .post(&format!("{}/api/ofga/http/stores", base_url))
        .json(&json!({
            "name": "example-store"
        }))
        .send()
        .await?;

    if create_store_response.status().is_success() {
        let store_data: serde_json::Value = create_store_response.json().await?;
        println!("✅ Store created: {}", store_data);
        
        if let Some(store_id) = store_data.get("id").and_then(|id| id.as_str()) {
            
            // Example 2: Create an authorization model
            println!("\n🔐 2. Creating authorization model...");
            let auth_model = json!({
                "schema_version": "1.1",
                "type_definitions": [
                    {
                        "type": "user"
                    },
                    {
                        "type": "document",
                        "relations": {
                            "reader": {
                                "this": {}
                            },
                            "writer": {
                                "this": {}
                            }
                        },
                        "metadata": {
                            "relations": {
                                "reader": {
                                    "directly_related_user_types": [
                                        {"type": "user"}
                                    ]
                                },
                                "writer": {
                                    "directly_related_user_types": [
                                        {"type": "user"}
                                    ]
                                }
                            }
                        }
                    }
                ]
            });

            let create_model_response = client
                .post(&format!("{}/api/ofga/http/stores/{}/authorization-models", base_url, store_id))
                .json(&auth_model)
                .send()
                .await?;

            if create_model_response.status().is_success() {
                let model_data: serde_json::Value = create_model_response.json().await?;
                println!("✅ Authorization model created: {}", model_data);

                // Example 3: Write a tuple
                println!("\n📝 3. Writing a tuple...");
                let write_request = json!({
                    "store_id": store_id,
                    "write_request": {
                        "writes": {
                            "tuple_keys": [
                                {
                                    "user": "user:alice",
                                    "relation": "reader",
                                    "object": "document:readme"
                                }
                            ]
                        }
                    }
                });

                let write_response = client
                    .post(&format!("{}/api/ofga/http/write", base_url))
                    .json(&write_request)
                    .send()
                    .await?;

                if write_response.status().is_success() {
                    println!("✅ Tuple written successfully");

                    // Example 4: Check authorization
                    println!("\n🔍 4. Checking authorization...");
                    let check_request = json!({
                        "store_id": store_id,
                        "check_request": {
                            "tuple_key": {
                                "user": "user:alice",
                                "relation": "reader",
                                "object": "document:readme"
                            }
                        }
                    });

                    let check_response = client
                        .post(&format!("{}/api/ofga/http/check", base_url))
                        .json(&check_request)
                        .send()
                        .await?;

                    if check_response.status().is_success() {
                        let check_data: serde_json::Value = check_response.json().await?;
                        let allowed = check_data.get("allowed").and_then(|a| a.as_bool()).unwrap_or(false);
                        
                        if allowed {
                            println!("✅ Authorization check passed: user:alice can read document:readme");
                        } else {
                            println!("❌ Authorization check failed: user:alice cannot read document:readme");
                        }
                    } else {
                        println!("❌ Check request failed: {}", check_response.status());
                    }

                    // Example 5: Read tuples
                    println!("\n📖 5. Reading tuples...");
                    let read_request = json!({
                        "store_id": store_id,
                        "read_request": {
                            "tuple_key": {
                                "object": "document:readme"
                            }
                        }
                    });

                    let read_response = client
                        .post(&format!("{}/api/ofga/http/read", base_url))
                        .json(&read_request)
                        .send()
                        .await?;

                    if read_response.status().is_success() {
                        let read_data: serde_json::Value = read_response.json().await?;
                        println!("✅ Tuples read successfully: {}", read_data);
                    } else {
                        println!("❌ Read request failed: {}", read_response.status());
                    }

                } else {
                    println!("❌ Write request failed: {}", write_response.status());
                }
            } else {
                println!("❌ Model creation failed: {}", create_model_response.status());
            }
        } else {
            println!("❌ Could not extract store ID from response");
        }
    } else {
        println!("❌ Store creation failed: {}", create_store_response.status());
    }

    println!("\n🎉 HTTP Client Example Complete!");
    Ok(())
}

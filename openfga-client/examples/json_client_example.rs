use openfga_client::OpenFGAClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Example JSON authorization model (from OpenFGA playground format)
    let json_model_str = r#"{
        "schema_version": "1.1",
        "type_definitions": [
            {
                "type": "user"
            },
            {
                "type": "document",
                "relations": {
                    "owner": {
                        "this": {}
                    },
                    "viewer": {
                        "union": {
                            "child": [
                                {
                                    "this": {}
                                },
                                {
                                    "computedUserset": {
                                        "object": "",
                                        "relation": "owner"
                                    }
                                }
                            ]
                        }
                    }
                },
                "metadata": {
                    "relations": {
                        "owner": {
                            "directly_related_user_types": [
                                {
                                    "type": "user"
                                }
                            ]
                        },
                        "viewer": {
                            "directly_related_user_types": [
                                {
                                    "type": "user"
                                }
                            ]
                        }
                    }
                }
            }
        ]
    }"#;

    // Parse the JSON model
    let json_model = OpenFGAClient::parse_authorization_model_from_json(json_model_str)?;
    println!("✅ Parsed JSON authorization model");
    println!("   Schema version: {}", json_model.schema_version);
    println!("   Type definitions: {}", json_model.type_definitions.len());

    // Create a client (this would connect to a real OpenFGA server)
    // let mut client = OpenFGAClient::new("http://localhost:8080".to_string()).await?;

    // Write the authorization model from JSON
    // let response = client.write_authorization_model_from_json("store_id".to_string(), json_model).await?;
    // println!("✅ Authorization model written successfully");

    // You can also work with the JSON types directly
    for type_def in &json_model.type_definitions {
        println!("\n📋 Type: {}", type_def.type_name);
        for (relation_name, userset) in &type_def.relations {
            println!("   Relation: {}", relation_name);

            // Check what kind of userset this is
            if userset.this.is_some() {
                println!("     -> Direct assignment (this)");
            } else if let Some(computed) = &userset.computed_userset {
                println!("     -> Computed userset: {}", computed.relation);
            } else if let Some(union) = &userset.union {
                println!("     -> Union with {} children", union.child.len());
            }
        }
    }

    // Convert to OpenFGA protobuf types if needed
    let (type_definitions, schema_version, conditions) = json_model.to_openfga_types()?;
    println!("\n✅ Successfully converted to OpenFGA protobuf types");
    println!("   Converted {} type definitions", type_definitions.len());
    println!("   Schema version: {}", schema_version);
    println!("   Conditions: {}", conditions.len());

    Ok(())
}

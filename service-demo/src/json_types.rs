use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON representation of an authorization model from OpenFGA playground
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAuthModel {
    pub schema_version: String,
    pub type_definitions: Vec<JsonTypeDefinition>,
    #[serde(default)]
    pub conditions: HashMap<String, serde_json::Value>,
}

/// JSON representation of a type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTypeDefinition {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub relations: HashMap<String, JsonUserset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonMetadata>,
}

/// JSON representation of metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<HashMap<String, JsonRelationMetadata>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_info: Option<serde_json::Value>,
}

/// JSON representation of relation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRelationMetadata {
    #[serde(default)]
    pub directly_related_user_types: Vec<JsonDirectlyRelatedUserType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_info: Option<serde_json::Value>,
}

/// JSON representation of directly related user type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDirectlyRelatedUserType {
    #[serde(rename = "type")]
    pub type_name: String,
    pub relation: Option<String>,
    pub condition: Option<String>,
}

/// JSON representation of a userset - matches exactly what comes from OpenFGA playground
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonUserset {
    // Direct userset
    #[serde(rename = "this")]
    pub this: Option<JsonDirectUserset>,

    // Computed userset
    #[serde(rename = "computedUserset")]
    pub computed_userset: Option<JsonComputedUserset>,

    // Tuple to userset
    #[serde(rename = "tupleToUserset")]
    pub tuple_to_userset: Option<JsonTupleToUserset>,

    // Union
    pub union: Option<JsonUnion>,

    // Intersection
    pub intersection: Option<JsonIntersection>,

    // Difference
    pub difference: Option<JsonDifference>,
}

/// Direct userset - just an empty object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDirectUserset {}

/// Computed userset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonComputedUserset {
    #[serde(default)]
    pub object: String,
    pub relation: String,
}

/// Tuple to userset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTupleToUserset {
    pub tupleset: JsonObjectRelation,
    #[serde(rename = "computedUserset")]
    pub computed_userset: JsonObjectRelation,
}

/// Object relation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonObjectRelation {
    #[serde(default)]
    pub object: String,
    pub relation: String,
}

/// Union of usersets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonUnion {
    pub child: Vec<JsonUserset>,
}

/// Intersection of usersets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonIntersection {
    pub child: Vec<JsonUserset>,
}

/// Difference of usersets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDifference {
    pub base: Box<JsonUserset>,
    pub subtract: Box<JsonUserset>,
}

// Conversion functions to OpenFGA generated types
impl JsonAuthModel {
    /// Convert to OpenFGA generated types
    pub fn to_openfga_types(
        self,
    ) -> Result<
        (
            Vec<openfga_client::TypeDefinition>,
            String,
            HashMap<String, openfga_client::Condition>,
        ),
        String,
    > {
        let mut type_definitions = Vec::new();

        for json_type_def in self.type_definitions {
            type_definitions.push(json_type_def.to_openfga_type()?);
        }

        // For now, return empty conditions - can be enhanced later
        let conditions = HashMap::new();

        Ok((type_definitions, self.schema_version, conditions))
    }
}

impl JsonTypeDefinition {
    /// Convert to OpenFGA TypeDefinition
    pub fn to_openfga_type(self) -> Result<openfga_client::TypeDefinition, String> {
        let mut relations = HashMap::new();

        for (relation_name, json_userset) in self.relations {
            relations.insert(relation_name, json_userset.to_openfga_userset()?);
        }

        // Convert metadata if present
        let metadata = if let Some(json_metadata) = self.metadata {
            Some(json_metadata.to_openfga_metadata()?)
        } else {
            None
        };

        Ok(openfga_client::TypeDefinition {
            r#type: self.type_name,
            relations,
            metadata,
        })
    }
}

impl JsonUserset {
    /// Convert to OpenFGA Userset
    pub fn to_openfga_userset(self) -> Result<openfga_client::Userset, String> {
        use openfga_client::{
            Difference, DirectUserset, ObjectRelation, TupleToUserset, Userset, Usersets, userset,
        };

        if self.this.is_some() {
            Ok(Userset {
                userset: Some(userset::Userset::This(DirectUserset {})),
            })
        } else if let Some(computed) = self.computed_userset {
            Ok(Userset {
                userset: Some(userset::Userset::ComputedUserset(ObjectRelation {
                    object: computed.object,
                    relation: computed.relation,
                })),
            })
        } else if let Some(tuple_to_userset) = self.tuple_to_userset {
            Ok(Userset {
                userset: Some(userset::Userset::TupleToUserset(TupleToUserset {
                    tupleset: Some(ObjectRelation {
                        object: tuple_to_userset.tupleset.object,
                        relation: tuple_to_userset.tupleset.relation,
                    }),
                    computed_userset: Some(ObjectRelation {
                        object: tuple_to_userset.computed_userset.object,
                        relation: tuple_to_userset.computed_userset.relation,
                    }),
                })),
            })
        } else if let Some(union) = self.union {
            let mut child_usersets = Vec::new();
            for child in union.child {
                child_usersets.push(child.to_openfga_userset()?);
            }
            Ok(Userset {
                userset: Some(userset::Userset::Union(Usersets {
                    child: child_usersets,
                })),
            })
        } else if let Some(intersection) = self.intersection {
            let mut child_usersets = Vec::new();
            for child in intersection.child {
                child_usersets.push(child.to_openfga_userset()?);
            }
            Ok(Userset {
                userset: Some(userset::Userset::Intersection(Usersets {
                    child: child_usersets,
                })),
            })
        } else if let Some(difference) = self.difference {
            Ok(Userset {
                userset: Some(userset::Userset::Difference(Box::new(Difference {
                    base: Some(Box::new(difference.base.to_openfga_userset()?)),
                    subtract: Some(Box::new(difference.subtract.to_openfga_userset()?)),
                }))),
            })
        } else {
            Err("Unknown userset type - no recognized fields found".to_string())
        }
    }
}

impl JsonMetadata {
    /// Convert to OpenFGA Metadata
    pub fn to_openfga_metadata(self) -> Result<openfga_client::Metadata, String> {
        let mut relations = HashMap::new();

        if let Some(json_relations) = self.relations {
            for (relation_name, json_relation_metadata) in json_relations {
                relations.insert(
                    relation_name,
                    json_relation_metadata.to_openfga_relation_metadata()?,
                );
            }
        }

        Ok(openfga_client::Metadata {
            relations,
            module: self.module.unwrap_or_default(),
            source_info: None, // We can implement source_info conversion later if needed
        })
    }
}

impl JsonRelationMetadata {
    /// Convert to OpenFGA RelationMetadata
    pub fn to_openfga_relation_metadata(self) -> Result<openfga_client::RelationMetadata, String> {
        let mut directly_related_user_types = Vec::new();

        for json_user_type in self.directly_related_user_types {
            directly_related_user_types.push(json_user_type.to_openfga_relation_reference()?);
        }

        Ok(openfga_client::RelationMetadata {
            directly_related_user_types,
            module: self.module.unwrap_or_default(),
            source_info: None, // We can implement source_info conversion later if needed
        })
    }
}

impl JsonDirectlyRelatedUserType {
    /// Convert to OpenFGA RelationReference
    pub fn to_openfga_relation_reference(
        self,
    ) -> Result<openfga_client::RelationReference, String> {
        use openfga_client::{RelationReference, Wildcard, relation_reference};

        // Debug log the input
        tracing::debug!(
            "Converting relation reference: type={}, relation={:?}, condition={:?}",
            self.type_name,
            self.relation,
            self.condition
        );

        let relation_or_wildcard = match self.relation {
            Some(relation) if !relation.is_empty() => {
                // Specific relation like "group#member"
                tracing::debug!("Using specific relation: {}", relation);
                Some(relation_reference::RelationOrWildcard::Relation(relation))
            }
            Some(_) | None => {
                // Empty string or None means wildcard (any instance of the type)
                // Try setting to None instead of Wildcard to see if that fixes the issue
                tracing::debug!(
                    "Using None (no relation specified) for type: {}",
                    self.type_name
                );
                None
            }
        };

        let condition = self.condition.unwrap_or_default();

        // Don't set empty conditions - they might cause validation issues
        let final_condition = if condition.is_empty() {
            String::new()
        } else {
            condition
        };

        Ok(RelationReference {
            r#type: self.type_name,
            condition: final_condition,
            relation_or_wildcard,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_this_relation() {
        let json = r#"{"this": {}}"#;
        let userset: JsonUserset = serde_json::from_str(json).unwrap();
        assert!(userset.this.is_some());

        let openfga_userset = userset.to_openfga_userset().unwrap();
        match openfga_userset.userset {
            Some(openfga_client::userset::Userset::This(_)) => {}
            _ => panic!("Expected This variant"),
        }
    }

    #[test]
    fn test_parse_computed_userset() {
        let json = r#"{"computedUserset": {"object": "", "relation": "member"}}"#;
        let userset: JsonUserset = serde_json::from_str(json).unwrap();
        assert!(userset.computed_userset.is_some());

        let openfga_userset = userset.to_openfga_userset().unwrap();
        match openfga_userset.userset {
            Some(openfga_client::userset::Userset::ComputedUserset(obj_rel)) => {
                assert_eq!(obj_rel.relation, "member");
            }
            _ => panic!("Expected ComputedUserset variant"),
        }
    }

    #[test]
    fn test_parse_union() {
        let json = r#"{"union": {"child": [{"this": {}}, {"computedUserset": {"object": "", "relation": "owner"}}]}}"#;
        let userset: JsonUserset = serde_json::from_str(json).unwrap();
        assert!(userset.union.is_some());

        let openfga_userset = userset.to_openfga_userset().unwrap();
        match openfga_userset.userset {
            Some(openfga_client::userset::Userset::Union(usersets)) => {
                assert_eq!(usersets.child.len(), 2);
            }
            _ => panic!("Expected Union variant"),
        }
    }

    #[test]
    fn test_auth_model_example_conversion() {
        // Test with the actual auth-model-example.json file
        let json_content = std::fs::read_to_string("../etc/auth-model-example.json")
            .expect("Failed to read auth-model-example.json");

        println!("🔄 Testing conversion of auth-model-example.json");

        // Parse the JSON
        let json_model: JsonAuthModel =
            serde_json::from_str(&json_content).expect("Failed to parse JSON");

        println!("✅ Successfully parsed JSON auth model");
        println!("   Schema version: {}", json_model.schema_version);
        println!("   Type definitions: {}", json_model.type_definitions.len());

        // Convert to OpenFGA types
        let (type_definitions, schema_version, conditions) = json_model
            .to_openfga_types()
            .expect("Failed to convert to OpenFGA types");

        println!("✅ Successfully converted to OpenFGA types");
        println!("   Converted type definitions: {}", type_definitions.len());
        println!("   Schema version: {}", schema_version);
        println!("   Conditions: {}", conditions.len());

        // Detailed analysis of each type
        for type_def in &type_definitions {
            println!("\n📋 Type: {}", type_def.r#type);
            println!("   Relations: {}", type_def.relations.len());

            // Show relation details
            for (relation_name, userset) in &type_def.relations {
                let variant = match &userset.userset {
                    Some(openfga_client::userset::Userset::This(_)) => "This",
                    Some(openfga_client::userset::Userset::ComputedUserset(_)) => "ComputedUserset",
                    Some(openfga_client::userset::Userset::TupleToUserset(_)) => "TupleToUserset",
                    Some(openfga_client::userset::Userset::Union(_)) => "Union",
                    Some(openfga_client::userset::Userset::Intersection(_)) => "Intersection",
                    Some(openfga_client::userset::Userset::Difference(_)) => "Difference",
                    None => "None",
                };
                println!("     - {} -> {}", relation_name, variant);
            }

            // Show metadata details
            if let Some(metadata) = &type_def.metadata {
                println!("   Metadata relations: {}", metadata.relations.len());
                for (relation_name, relation_metadata) in &metadata.relations {
                    println!(
                        "     - {}: {} user types",
                        relation_name,
                        relation_metadata.directly_related_user_types.len()
                    );
                    for user_type in &relation_metadata.directly_related_user_types {
                        let relation_info = match &user_type.relation_or_wildcard {
                            Some(
                                openfga_client::relation_reference::RelationOrWildcard::Relation(
                                    rel,
                                ),
                            ) => {
                                format!("#{}", rel)
                            }
                            Some(
                                openfga_client::relation_reference::RelationOrWildcard::Wildcard(_),
                            ) => "*".to_string(),
                            None => "None".to_string(),
                        };
                        println!(
                            "       * {}{} (condition: '{}')",
                            user_type.r#type, relation_info, user_type.condition
                        );
                    }
                }
            } else {
                println!("   No metadata");
            }
        }

        // Test specific cases that are causing issues
        let organisation_type = type_definitions
            .iter()
            .find(|t| t.r#type == "organisation")
            .expect("Organisation type not found");

        println!("\n🔍 Detailed analysis of organisation type:");

        // Check the child relation specifically
        if let Some(metadata) = &organisation_type.metadata {
            if let Some(child_metadata) = metadata.relations.get("child") {
                println!("   Child relation metadata:");
                for (i, user_type) in child_metadata
                    .directly_related_user_types
                    .iter()
                    .enumerate()
                {
                    println!(
                        "     [{}] Type: '{}', Condition: '{}'",
                        i, user_type.r#type, user_type.condition
                    );
                    match &user_type.relation_or_wildcard {
                        Some(openfga_client::relation_reference::RelationOrWildcard::Relation(
                            rel,
                        )) => {
                            println!("         Relation: '{}'", rel);
                        }
                        Some(openfga_client::relation_reference::RelationOrWildcard::Wildcard(
                            _,
                        )) => {
                            println!("         Wildcard (any instance of type)");
                        }
                        None => {
                            println!("         No relation_or_wildcard set");
                        }
                    }
                }
            }
        }

        // Verify the conversion matches expectations from the .fga file
        // From auth-model-example.fga: "define child: [organisation]"
        // This should mean: child relation can be assigned any organisation (wildcard)
        assert_eq!(type_definitions.len(), 4);
        assert_eq!(schema_version, "1.1");
    }
}

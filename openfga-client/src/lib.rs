pub mod grpc;

pub use grpc::generated;
pub use grpc::json_types;

// Re-export the generated types and client for convenience
pub use generated::open_fga_service_client::OpenFgaServiceClient;
pub use generated::*;

// Re-export JSON types for public API
pub use json_types::*;

// High-level client wrapper for easier usage
use tonic::transport::Channel;

pub struct OpenFGAClient {
    client: OpenFgaServiceClient<Channel>,
}

impl OpenFGAClient {
    /// Create a new OpenFGA client
    pub async fn new(endpoint: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(endpoint)?.connect().await?;

        let client = OpenFgaServiceClient::new(channel);

        Ok(Self { client })
    }

    /// Get the underlying gRPC client
    pub fn inner(&mut self) -> &mut OpenFgaServiceClient<Channel> {
        &mut self.client
    }

    /// Read tuples from the store
    pub async fn read(
        &mut self,
        request: ReadRequest,
    ) -> Result<tonic::Response<ReadResponse>, tonic::Status> {
        self.client.read(request).await
    }

    /// Write tuples to the store
    pub async fn write(
        &mut self,
        request: WriteRequest,
    ) -> Result<tonic::Response<WriteResponse>, tonic::Status> {
        self.client.write(request).await
    }

    /// Check if a user has a relation to an object
    pub async fn check(
        &mut self,
        request: CheckRequest,
    ) -> Result<tonic::Response<CheckResponse>, tonic::Status> {
        self.client.check(request).await
    }

    /// Expand a userset
    pub async fn expand(
        &mut self,
        request: ExpandRequest,
    ) -> Result<tonic::Response<ExpandResponse>, tonic::Status> {
        self.client.expand(request).await
    }

    /// Get authorization model
    pub async fn read_authorization_model(
        &mut self,
        request: ReadAuthorizationModelRequest,
    ) -> Result<tonic::Response<ReadAuthorizationModelResponse>, tonic::Status> {
        self.client.read_authorization_model(request).await
    }

    /// Write authorization model
    pub async fn write_authorization_model(
        &mut self,
        request: WriteAuthorizationModelRequest,
    ) -> Result<tonic::Response<WriteAuthorizationModelResponse>, tonic::Status> {
        self.client.write_authorization_model(request).await
    }

    /// List authorization models
    pub async fn read_authorization_models(
        &mut self,
        request: ReadAuthorizationModelsRequest,
    ) -> Result<tonic::Response<ReadAuthorizationModelsResponse>, tonic::Status> {
        self.client.read_authorization_models(request).await
    }

    /// Get store
    pub async fn get_store(
        &mut self,
        request: GetStoreRequest,
    ) -> Result<tonic::Response<GetStoreResponse>, tonic::Status> {
        self.client.get_store(request).await
    }

    /// List stores
    pub async fn list_stores(
        &mut self,
        request: ListStoresRequest,
    ) -> Result<tonic::Response<ListStoresResponse>, tonic::Status> {
        self.client.list_stores(request).await
    }

    /// Create store
    pub async fn create_store(
        &mut self,
        request: CreateStoreRequest,
    ) -> Result<tonic::Response<CreateStoreResponse>, tonic::Status> {
        self.client.create_store(request).await
    }

    /// Delete store
    pub async fn delete_store(
        &mut self,
        request: DeleteStoreRequest,
    ) -> Result<tonic::Response<DeleteStoreResponse>, tonic::Status> {
        self.client.delete_store(request).await
    }

    /// List objects
    pub async fn list_objects(
        &mut self,
        request: ListObjectsRequest,
    ) -> Result<tonic::Response<ListObjectsResponse>, tonic::Status> {
        self.client.list_objects(request).await
    }

    /// Stream changes
    pub async fn read_changes(
        &mut self,
        request: ReadChangesRequest,
    ) -> Result<tonic::Response<ReadChangesResponse>, tonic::Status> {
        self.client.read_changes(request).await
    }
}

// Helper functions for creating common request types
impl OpenFGAClient {
    /// Create a simple check request
    pub fn create_check_request(
        store_id: String,
        object: String,
        relation: String,
        user: String,
    ) -> CheckRequest {
        CheckRequest {
            store_id,
            tuple_key: Some(CheckRequestTupleKey {
                object,
                relation,
                user,
            }),
            contextual_tuples: None,
            authorization_model_id: String::new(),
            trace: false,
            consistency: ConsistencyPreference::Unspecified as i32,
            context: None,
        }
    }

    /// Create a simple write request
    pub fn create_write_request(
        store_id: String,
        object_type: String,
        object_id: String,
        relation: String,
        user_type: String,
        user_id: String,
    ) -> WriteRequest {
        WriteRequest {
            store_id,
            writes: Some(WriteRequestWrites {
                tuple_keys: vec![TupleKey {
                    object: format!("{}:{}", object_type, object_id),
                    relation,
                    user: format!("{}:{}", user_type, user_id),
                    condition: None,
                }],
                on_duplicate: String::new(),
            }),
            deletes: None,
            authorization_model_id: String::new(),
        }
    }
}

// JSON-friendly wrapper methods
impl OpenFGAClient {
    /// Write authorization model from JSON
    pub async fn write_authorization_model_from_json(
        &mut self,
        store_id: String,
        json_model: JsonAuthModel,
    ) -> Result<tonic::Response<WriteAuthorizationModelResponse>, Box<dyn std::error::Error>> {
        let (type_definitions, _schema_version, _conditions) = json_model
            .to_openfga_types()
            .map_err(|e| format!("Failed to convert JSON model: {}", e))?;

        let request = WriteAuthorizationModelRequest {
            store_id,
            type_definitions,
            schema_version: "1.1".to_string(),
            conditions: std::collections::HashMap::new(),
        };

        Ok(self.write_authorization_model(request).await?)
    }

    /// Parse authorization model from JSON string
    pub fn parse_authorization_model_from_json(
        json_content: &str,
    ) -> Result<JsonAuthModel, Box<dyn std::error::Error>> {
        let model: JsonAuthModel = serde_json::from_str(json_content)?;
        Ok(model)
    }

    /// Write authorization model from JSON string
    pub async fn write_authorization_model_from_json_string(
        &mut self,
        store_id: String,
        json_content: &str,
    ) -> Result<tonic::Response<WriteAuthorizationModelResponse>, Box<dyn std::error::Error>> {
        let json_model = Self::parse_authorization_model_from_json(json_content)?;
        self.write_authorization_model_from_json(store_id, json_model)
            .await
    }

    /// Convert protobuf authorization model to JSON
    pub fn authorization_model_to_json(
        model: &AuthorizationModel,
    ) -> Result<JsonAuthModel, Box<dyn std::error::Error>> {
        let mut json_type_definitions = Vec::new();

        for type_def in &model.type_definitions {
            let mut json_relations = std::collections::HashMap::new();

            for (relation_name, userset) in &type_def.relations {
                json_relations.insert(relation_name.clone(), Self::userset_to_json(userset)?);
            }

            let json_metadata = if let Some(metadata) = &type_def.metadata {
                Some(Self::metadata_to_json(metadata)?)
            } else {
                None
            };

            json_type_definitions.push(JsonTypeDefinition {
                type_name: type_def.r#type.clone(),
                relations: json_relations,
                metadata: json_metadata,
            });
        }

        Ok(JsonAuthModel {
            schema_version: model.schema_version.clone(),
            type_definitions: json_type_definitions,
            conditions: std::collections::HashMap::new(),
        })
    }

    /// Helper to convert Userset to JsonUserset
    fn userset_to_json(userset: &Userset) -> Result<JsonUserset, Box<dyn std::error::Error>> {
        use crate::userset::Userset as UsersetVariant;

        let mut json_userset = JsonUserset {
            this: None,
            computed_userset: None,
            tuple_to_userset: None,
            union: None,
            intersection: None,
            difference: None,
        };

        if let Some(variant) = &userset.userset {
            match variant {
                UsersetVariant::This(_) => {
                    json_userset.this = Some(JsonDirectUserset {});
                }
                UsersetVariant::ComputedUserset(obj_rel) => {
                    json_userset.computed_userset = Some(JsonComputedUserset {
                        object: obj_rel.object.clone(),
                        relation: obj_rel.relation.clone(),
                    });
                }
                UsersetVariant::TupleToUserset(ttu) => {
                    let tupleset = if let Some(ts) = &ttu.tupleset {
                        JsonObjectRelation {
                            object: ts.object.clone(),
                            relation: ts.relation.clone(),
                        }
                    } else {
                        return Err("TupleToUserset missing tupleset".into());
                    };

                    let computed_userset = if let Some(cu) = &ttu.computed_userset {
                        JsonObjectRelation {
                            object: cu.object.clone(),
                            relation: cu.relation.clone(),
                        }
                    } else {
                        return Err("TupleToUserset missing computed_userset".into());
                    };

                    json_userset.tuple_to_userset = Some(JsonTupleToUserset {
                        tupleset,
                        computed_userset,
                    });
                }
                UsersetVariant::Union(usersets) => {
                    let mut children = Vec::new();
                    for child in &usersets.child {
                        children.push(Self::userset_to_json(child)?);
                    }
                    json_userset.union = Some(JsonUnion { child: children });
                }
                UsersetVariant::Intersection(usersets) => {
                    let mut children = Vec::new();
                    for child in &usersets.child {
                        children.push(Self::userset_to_json(child)?);
                    }
                    json_userset.intersection = Some(JsonIntersection { child: children });
                }
                UsersetVariant::Difference(diff) => {
                    let base = if let Some(b) = &diff.base {
                        Box::new(Self::userset_to_json(b)?)
                    } else {
                        return Err("Difference missing base".into());
                    };

                    let subtract = if let Some(s) = &diff.subtract {
                        Box::new(Self::userset_to_json(s)?)
                    } else {
                        return Err("Difference missing subtract".into());
                    };

                    json_userset.difference = Some(JsonDifference { base, subtract });
                }
            }
        }

        Ok(json_userset)
    }

    /// Helper to convert Metadata to JsonMetadata
    fn metadata_to_json(metadata: &Metadata) -> Result<JsonMetadata, Box<dyn std::error::Error>> {
        let mut json_relations = std::collections::HashMap::new();

        for (relation_name, relation_metadata) in &metadata.relations {
            let mut json_user_types = Vec::new();

            for relation_ref in &relation_metadata.directly_related_user_types {
                let relation = match &relation_ref.relation_or_wildcard {
                    Some(crate::relation_reference::RelationOrWildcard::Relation(rel)) => {
                        Some(rel.clone())
                    }
                    Some(crate::relation_reference::RelationOrWildcard::Wildcard(_)) => None,
                    None => None,
                };

                json_user_types.push(JsonDirectlyRelatedUserType {
                    type_name: relation_ref.r#type.clone(),
                    relation,
                    condition: if relation_ref.condition.is_empty() {
                        None
                    } else {
                        Some(relation_ref.condition.clone())
                    },
                });
            }

            json_relations.insert(
                relation_name.clone(),
                JsonRelationMetadata {
                    directly_related_user_types: json_user_types,
                    module: if relation_metadata.module.is_empty() {
                        None
                    } else {
                        Some(relation_metadata.module.clone())
                    },
                    source_info: None,
                },
            );
        }

        Ok(JsonMetadata {
            relations: if json_relations.is_empty() {
                None
            } else {
                Some(json_relations)
            },
            module: if metadata.module.is_empty() {
                None
            } else {
                Some(metadata.module.clone())
            },
            source_info: None,
        })
    }
}

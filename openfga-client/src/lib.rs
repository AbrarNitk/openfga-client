// Include the generated OpenFGA client code
pub mod generated;

// Re-export the generated types and client for convenience
pub use generated::open_fga_service_client::OpenFgaServiceClient;
pub use generated::*;

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

use openfga_grpc_client::OpenFgaServiceClient;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tonic::transport::Channel;

/// OpenFGA configuration parameters
#[derive(Clone, Debug)]
pub struct OpenFgaConfig {
    /// OpenFGA store ID
    pub store_id: String,
    /// OpenFGA authorization model ID
    pub authorization_model_id: String,
}

/// Application context that holds shared resources
#[derive(Clone)]
pub struct Ctx {
    /// PostgreSQL connection pool
    pub db: PgPool,
    /// Application profile name (e.g., "dev", "prod")
    pub profile: String,
    /// OpenFGA client
    pub fga_client: OpenFgaServiceClient<Channel>,
    /// OpenFGA configuration
    pub fga_config: OpenFgaConfig,
}

impl Ctx {
    /// Create a new application context
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables from .env file if it exists
        dotenv::dotenv().ok();

        // Get profile name from environment, default to "dev"
        let profile = env::var("PROFILE").unwrap_or_else(|_| "dev".to_string());
        tracing::info!("Starting application with profile: {}", profile);

        // Create database connection pool
        let db = pg_pool().await?;

        // Initialize OpenFGA client
        let fga_client = init_fga_client().await?;

        // Get OpenFGA configuration
        let fga_config = get_fga_config();

        // Log OpenFGA configuration
        if !fga_config.store_id.is_empty() {
            tracing::info!("Using OpenFGA store ID: {}", fga_config.store_id);
        }

        Ok(Self {
            db,
            profile,
            fga_client,
            fga_config,
        })
    }
}

async fn pg_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing::info!("Connecting to database");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Test database connection
    sqlx::query("SELECT 1").execute(&db).await?;
    tracing::info!("Database connection established successfully");

    Ok(db)
}

/// Initialize the OpenFGA client
async fn init_fga_client() -> Result<OpenFgaServiceClient<Channel>, Box<dyn std::error::Error>> {
    // Get OpenFGA client URL from environment, default to localhost
    let fga_url =
        env::var("OPENFGA_CLIENT_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    tracing::info!("Connecting to OpenFGA at {}", fga_url);

    // Create OpenFGA client without authentication
    let client = OpenFgaServiceClient::connect(fga_url).await?;
    tracing::info!("OpenFGA client initialized successfully");

    Ok(client)
}

/// Get OpenFGA configuration from environment variables
fn get_fga_config() -> OpenFgaConfig {
    // Get OpenFGA store ID from environment, default to empty string which will need to be set later
    let store_id = env::var("OPENFGA_STORE_ID").unwrap_or_else(|_| {
        tracing::warn!("OPENFGA_STORE_ID not set, using empty string");
        String::new()
    });

    // Get OpenFGA authorization model ID from environment, optional
    let authorization_model_id = match env::var("OPENFGA_AUTH_MODEL_ID") {
        Ok(id) => {
            tracing::info!("Using OpenFGA authorization model ID: {}", id);
            id
        }
        Err(_) => {
            tracing::info!("OPENFGA_AUTH_MODEL_ID not set, will need to be set later");
            std::process::exit(1);
        }
    };

    OpenFgaConfig {
        store_id,
        authorization_model_id,
    }
}

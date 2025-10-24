/// Example: Initializing AppState with Redis Connection Pool
/// 
/// This example shows how to properly initialize the application state
/// with a Redis connection pool for multi-tenant authentication.

use crate::auth::{
    authn::{DexAppConfig},
    authn_controller::AppState,
    redis_pool::create_redis_pool,
};
use sqlx::PgPool;

/// Initialize application state with Redis connection pool
pub async fn initialize_app_state(
    db_pool: PgPool,
    redis_url: &str,
    dex_config: DexAppConfig,
) -> anyhow::Result<AppState> {
    // Create Redis connection pool
    let redis_pool = create_redis_pool(redis_url).await
        .context("Failed to create Redis connection pool")?;
    
    // Create application state
    let app_state = AppState {
        db: db_pool,
        dex_config,
        redis_pool,
    };
    
    // Verify Redis connectivity
    let mut conn = app_state.redis_pool.get().await
        .context("Failed to get Redis connection for verification")?;
    
    let _: String = conn.ping().await
        .context("Redis connectivity check failed")?;
    
    tracing::info!("Application state initialized successfully with Redis pool");
    
    Ok(app_state)
}

/// Example usage in main.rs or application startup
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db_pool = sqlx::PgPool::connect(&database_url).await
        .context("Failed to connect to database")?;
    
    // Redis connection
    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be set");
    
    // Dex configuration
    let dex_config = DexAppConfig {
        issuer_url: std::env::var("DEX_ISSUER_URL")
            .expect("DEX_ISSUER_URL must be set"),
        client_id: std::env::var("DEX_CLIENT_ID")
            .expect("DEX_CLIENT_ID must be set"),
        client_secret: std::env::var("DEX_CLIENT_SECRET")
            .expect("DEX_CLIENT_SECRET must be set"),
        redirect_url: std::env::var("DEX_REDIRECT_URL")
            .expect("DEX_REDIRECT_URL must be set"),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
            "offline_access".to_string(),
        ],
    };
    
    // Initialize application state
    let app_state = initialize_app_state(db_pool, &redis_url, dex_config).await?;
    
    // Your application setup continues here...
    // e.g., setting up routes, starting the server, etc.
    
    tracing::info!("Application started successfully");
    Ok(())
}

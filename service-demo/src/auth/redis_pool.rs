/// Redis Connection Pool Helper
///
/// This module provides utilities for creating and managing Redis connection pools
/// using bb8 for efficient connection management in multi-tenant applications.
use anyhow::{Context, Result};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use bb8_redis::redis::AsyncCommands;

/// Create a Redis connection pool with optimized settings
pub async fn create_redis_pool(redis_url: &str) -> Result<Pool<RedisConnectionManager>> {
    let manager = RedisConnectionManager::new(redis_url)
        .context("Failed to create Redis connection manager")?;

    let pool = Pool::builder()
        .max_size(20) // Maximum number of connections in the pool
        .min_idle(Some(5)) // Minimum number of idle connections
        .connection_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(Some(std::time::Duration::from_secs(300))) // 5 minutes
        .build(manager)
        .await
        .context("Failed to create Redis connection pool")?;

    // Test the connection
    let mut conn = pool
        .get()
        .await
        .context("Failed to get initial Redis connection")?;

    let _: String = conn
        .ping()
        .await
        .context("Redis ping failed during pool initialization")?;

    Ok(pool.clone())
}

/// Health check for Redis pool
pub async fn check_redis_health(pool: &Pool<RedisConnectionManager>) -> Result<bool> {
    let mut conn = pool
        .get()
        .await
        .context("Failed to get Redis connection for health check")?;

    let _: String = conn.ping().await.context("Redis ping failed")?;

    Ok(true)
}

/// Get pool statistics
pub fn get_pool_stats(pool: &Pool<RedisConnectionManager>) -> PoolStats {
    PoolStats {
        size: pool.state().connections,
        idle: pool.state().idle_connections,
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
}

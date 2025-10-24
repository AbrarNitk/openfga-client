/// Redis Connection Timeout Examples
/// 
/// This demonstrates how connection_timeout affects your authentication flow

use bb8::Pool;
use bb8_redis::{RedisConnectionManager, redis::AsyncCommands};
use std::time::Duration;

/// Example 1: Pool Exhaustion Scenario
/// 
/// If you have a high-traffic authentication system:
/// - Pool size: 10 connections
/// - 15 concurrent login requests come in
/// - First 10 get connections immediately
/// - Next 5 will wait for connection_timeout
async fn pool_exhaustion_example() -> anyhow::Result<()> {
    let pool = Pool::builder()
        .max_size(10) // Only 10 connections
        .connection_timeout(Duration::from_secs(5)) // Wait max 5 seconds
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    // Simulate 15 concurrent requests
    let mut handles = vec![];
    for i in 0..15 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            match pool.get().await {
                Ok(mut conn) => {
                    let duration = start.elapsed();
                    println!("Request {}: Got connection after {:?}", i, duration);
                    
                    // Do some Redis work
                    let _: String = conn.ping().await?;
                    Ok(())
                }
                Err(e) => {
                    let duration = start.elapsed();
                    println!("Request {}: Failed after {:?} - {}", i, duration, e);
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all requests
    for handle in handles {
        let _ = handle.await?;
    }
    
    Ok(())
}

/// Example 2: Slow Redis Server
/// 
/// If Redis is under heavy load or has slow queries:
async fn slow_redis_example() -> anyhow::Result<()> {
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(3)) // Short timeout
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    let start = std::time::Instant::now();
    
    match pool.get().await {
        Ok(mut conn) => {
            let duration = start.elapsed();
            println!("Got connection after {:?}", duration);
            
            // This might be slow if Redis is busy
            let _: String = conn.ping().await?;
            Ok(())
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("Failed after {:?} - {}", duration, e);
            Err(e)
        }
    }
}

/// Example 3: Your Authentication Flow
/// 
/// In your auth system, connection_timeout affects:
/// 1. Storing auth state
/// 2. Retrieving auth state  
/// 3. Invalidating auth state
async fn auth_flow_example() -> anyhow::Result<()> {
    let pool = Pool::builder()
        .max_size(20)
        .connection_timeout(Duration::from_secs(10)) // 10 second timeout
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    // 1. Store auth state (during login)
    let start = std::time::Instant::now();
    match pool.get().await {
        Ok(mut conn) => {
            let duration = start.elapsed();
            println!("Store auth state: Got connection after {:?}", duration);
            
            // Store the auth state
            let _: () = conn.set_ex("auth:state:123", "state_data", 300).await?;
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("Store auth state: Failed after {:?} - {}", duration, e);
            return Err(e);
        }
    }
    
    // 2. Retrieve auth state (during callback)
    let start = std::time::Instant::now();
    match pool.get().await {
        Ok(mut conn) => {
            let duration = start.elapsed();
            println!("Retrieve auth state: Got connection after {:?}", duration);
            
            // Retrieve the auth state
            let _: Option<String> = conn.get("auth:state:123").await?;
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("Retrieve auth state: Failed after {:?} - {}", duration, e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Example 4: Different Timeout Strategies
/// 
/// Different applications need different timeout strategies:
async fn timeout_strategies_example() -> anyhow::Result<()> {
    // Strategy 1: Fast fail (good for real-time apps)
    let fast_pool = Pool::builder()
        .connection_timeout(Duration::from_millis(500)) // 500ms timeout
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    // Strategy 2: Patient wait (good for batch processing)
    let patient_pool = Pool::builder()
        .connection_timeout(Duration::from_secs(30)) // 30 second timeout
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    // Strategy 3: Balanced (good for web apps)
    let balanced_pool = Pool::builder()
        .connection_timeout(Duration::from_secs(5)) // 5 second timeout
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    println!("Different timeout strategies configured");
    Ok(())
}

/// Example 5: Error Handling with Timeouts
/// 
/// How to handle connection timeout errors gracefully:
async fn error_handling_example() -> anyhow::Result<()> {
    let pool = Pool::builder()
        .max_size(5)
        .connection_timeout(Duration::from_secs(2))
        .build(RedisConnectionManager::new("redis://localhost")?)
        .await?;
    
    // Simulate high load
    for i in 0..10 {
        match pool.get().await {
            Ok(mut conn) => {
                println!("Request {}: Success", i);
                
                // Do work
                let _: String = conn.ping().await?;
            }
            Err(e) => {
                // Handle timeout gracefully
                if e.to_string().contains("timeout") {
                    println!("Request {}: Timeout - Redis pool exhausted", i);
                    // Could implement retry logic here
                } else {
                    println!("Request {}: Other error - {}", i, e);
                }
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Redis Connection Timeout Examples");
    println!("================================");
    
    // Run examples (commented out to avoid requiring Redis)
    // pool_exhaustion_example().await?;
    // slow_redis_example().await?;
    // auth_flow_example().await?;
    // timeout_strategies_example().await?;
    // error_handling_example().await?;
    
    println!("Examples completed!");
    Ok(())
}
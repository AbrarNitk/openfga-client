/// Authentication Routes
/// 
/// This module contains route definitions for the multi-tenant authentication flow

use crate::auth::authn_controller::{
    extract_subdomain_from_host, get_authorize_url_handler, login_handler, AppState,
    LoginRequest,
};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
    Json, Router,
};

// ============================================================================
// Route Handlers
// ============================================================================

/// OAuth callback handler
/// 
/// # Example Request
/// GET /auth/callback?code=AUTH_CODE&state=SIGNED_STATE
/// Host: acme.example.com
/// Cookie: ...
/// 
/// # Response
/// 302 Redirect to return_url with session cookie set
async fn callback_handler(
    State(state): State<AppState>,
    Query(query): Query<crate::auth::callback::CallbackQuery>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
) -> Result<axum::response::Redirect, axum::http::StatusCode> {
    use crate::auth::authn_controller::extract_subdomain_from_host;
    
    // Extract Host header
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("Missing or invalid Host header");
            axum::http::StatusCode::BAD_REQUEST
        })?;
    
    // Extract subdomain from host
    let subdomain = extract_subdomain_from_host(host).ok_or_else(|| {
        tracing::error!("Failed to extract subdomain from host: {}", host);
        axum::http::StatusCode::BAD_REQUEST
    })?;
    
    tracing::info!("Callback request for organization: {}", subdomain);
    
    // Get organization configuration
    let org_config = crate::auth::authn_controller::get_org_config_by_subdomain(&state.db, &subdomain)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get org config: {:?}", e);
            axum::http::StatusCode::NOT_FOUND
        })?;
    
    // Extract client information
    let client_ip = crate::auth::authn_controller::extract_client_ip(&headers);
    let client_user_agent = crate::auth::authn_controller::extract_user_agent(&headers);
    
    // Create auth builder
    let auth_builder = state.create_auth_builder().map_err(|e| {
        tracing::error!("Failed to create auth builder: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Handle callback
    let result = crate::auth::callback::handle_callback(
        &state.db,
        &state.dex_config,
        &org_config,
        &auth_builder,
        &query,
        &cookies,
        &client_ip,
        &client_user_agent,
    )
    .await
    .map_err(|e| {
        tracing::error!("Callback handling failed: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tracing::info!(
        "User {} logged in successfully with session {}",
        result.user_id,
        result.session_id
    );
    
    // Redirect to return URL
    Ok(axum::response::Redirect::to(&result.return_url))
}

/// Web login handler that extracts subdomain from Host header
/// 
/// # Example Request
/// GET https://acme.example.com/auth/login?return_url=/dashboard
/// Host: acme.example.com
/// 
/// # Response
/// 302 Redirect to Dex authorization URL
async fn login_with_subdomain_handler(
    State(state): State<AppState>,
    Query(query): Query<LoginRequest>,
    headers: HeaderMap,
) -> Result<Response, axum::http::StatusCode> {
    // Extract Host header
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("Missing or invalid Host header");
            axum::http::StatusCode::BAD_REQUEST
        })?;
    
    // Extract subdomain from host
    let subdomain = extract_subdomain_from_host(host).ok_or_else(|| {
        tracing::error!("Failed to extract subdomain from host: {}", host);
        axum::http::StatusCode::BAD_REQUEST
    })?;
    
    tracing::info!(
        "Login request for organization subdomain: {}, return_url: {:?}",
        subdomain,
        query.return_url
    );
    
    // Call the main login handler
    login_handler(State(state), Query(query), headers, subdomain)
        .await
        .map_err(|e| {
            tracing::error!("Login handler error: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// API login handler for SPAs and mobile apps
/// 
/// # Example Request
/// POST https://acme.example.com/api/v2/login-with
/// Host: acme.example.com
/// Content-Type: application/json
/// 
/// {
///   "return_url": "/dashboard"
/// }
/// 
/// # Example Response
/// {
///   "authorize_url": "https://dex.example.com/authorize?client_id=..."
/// }
async fn api_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Extract Host header
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("Missing or invalid Host header");
            axum::http::StatusCode::BAD_REQUEST
        })?;
    
    // Extract subdomain from host
    let subdomain = extract_subdomain_from_host(host).ok_or_else(|| {
        tracing::error!("Failed to extract subdomain from host: {}", host);
        axum::http::StatusCode::BAD_REQUEST
    })?;
    
    tracing::info!(
        "API login request for organization subdomain: {}, return_url: {:?}",
        subdomain,
        request.return_url
    );
    
    // Call the authorization URL handler
    let response = get_authorize_url_handler(State(state), headers, subdomain, Json(request))
        .await
        .map_err(|e| {
            tracing::error!("Get authorize URL error: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(serde_json::to_value(response.0).unwrap()))
}

// ============================================================================
// Route Definitions
// ============================================================================

/// Create authentication router with all auth-related routes
/// Subdomain is extracted from Host header (e.g., acme.example.com)
pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        // Web-based login flow (subdomain from Host header)
        .route("/auth/login", get(login_with_subdomain_handler))
        // API-based login flow (subdomain from Host header)
        .route("/api/v2/login-with", post(api_login_handler))
        // OAuth callback (handles token exchange and session creation)
        .route("/auth/callback", get(callback_handler))
        .layer(tower_cookies::CookieManagerLayer::new()) // Add cookie middleware
        .with_state(state)
}

// ============================================================================
// Usage Example in main.rs
// ============================================================================

/*
```rust
// In main.rs

use service_demo::auth::authn::{AuthorizationUrlBuilder, DexAppConfig};
use service_demo::auth::authn_controller::AppState;
use service_demo::routes::authn_routes::auth_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")?;
    let db = sqlx::PgPool::connect(&database_url).await?;

    // Load Dex configuration
    let dex_config = DexAppConfig {
        client_id: std::env::var("DEX_CLIENT_ID")?,
        client_secret: std::env::var("DEX_CLIENT_SECRET")?,
        issuer_url: std::env::var("DEX_ISSUER_URL")?,
        auth_url: std::env::var("DEX_AUTH_URL")?,
        token_url: std::env::var("DEX_TOKEN_URL")?,
        redirect_url: std::env::var("DEX_REDIRECT_URL")?,
        scopes: std::env::var("DEX_SCOPES")?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
    };

    // Get Redis URL for auth builder creation
    let redis_url = std::env::var("REDIS_URL")?;

    // Create application state
    let app_state = AppState {
        db,
        dex_config,
        redis_url,
    };

    // Build application router
    let app = Router::new()
        .merge(auth_routes(app_state))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // Start server
    let addr = "0.0.0.0:5001";
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```
*/


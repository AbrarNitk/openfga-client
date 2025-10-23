/// Authentication Controller
/// 
/// Handles the OAuth2/OIDC authentication flow with Dex for multi-tenant organizations

use super::authn::{AuthorizationUrlBuilder, AuthorizeRequest, DexAppConfig, OrgAuthConfig};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Optional return URL after successful login
    pub return_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// Authorization URL to redirect the user to
    pub authorize_url: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================================================
// Application State
// ============================================================================

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: sqlx::PgPool,
    
    /// Dex application configuration
    pub dex_config: DexAppConfig,
    
    /// Redis URL for creating auth builders
    pub redis_url: String,
}

impl AppState {
    /// Create a new authorization URL builder
    /// We create instances as needed since AuthorizationUrlBuilder is not Clone
    pub fn create_auth_builder(&self) -> anyhow::Result<AuthorizationUrlBuilder> {
        AuthorizationUrlBuilder::new(&self.redis_url)
    }
}

// ============================================================================
// Controller Functions
// ============================================================================

/// Handler for initiating login flow
/// 
/// # Endpoint
/// GET /auth/login?return_url=/dashboard
/// 
/// # Flow
/// 1. Extract organization from subdomain or request
/// 2. Load organization config from database
/// 3. Generate secure authorization URL
/// 4. Return URL or redirect user
pub async fn login_handler(
    State(app_state): State<AppState>,
    Query(query): Query<LoginRequest>,
    headers: HeaderMap,
    org_subdomain: String, // This should be extracted from host header or path
) -> Result<Response, AppError> {
    // 1. Extract client information
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    // 2. Lookup organization configuration by subdomain
    let org_config = get_org_config_by_subdomain(&app_state.db, &org_subdomain)
        .await
        .map_err(|e| AppError::NotFound(format!("Organization not found: {}", e)))?;
    
    // 3. Build authorization request
    let authorize_request = AuthorizeRequest {
        dex_config: app_state.dex_config.clone(),
        org_config,
        return_url: query.return_url.unwrap_or_else(|| "/dashboard".to_string()),
        client_ip,
        client_user_agent: user_agent,
    };
    
    // 4. Create auth builder and generate authorization URL
    let auth_builder = app_state
        .create_auth_builder()
        .map_err(|e| AppError::InternalError(format!("Failed to create auth builder: {}", e)))?;
    
    let auth_url = auth_builder
        .build_authorize_url(authorize_request)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to build auth URL: {}", e)))?;
    
    // 5. Return redirect response
    Ok(Redirect::to(&auth_url).into_response())
}

/// Handler for getting authorization URL as JSON (for SPA/mobile apps)
/// 
/// # Endpoint
/// POST /api/v2/login-with
/// Body: { "return_url": "/dashboard" }
/// 
/// # Response
/// { "authorize_url": "https://dex.example.com/authorize?..." }
pub async fn get_authorize_url_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    org_subdomain: String,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // 1. Extract client information
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    // 2. Lookup organization configuration
    let org_config = get_org_config_by_subdomain(&app_state.db, &org_subdomain)
        .await
        .map_err(|e| AppError::NotFound(format!("Organization not found: {}", e)))?;
    
    // 3. Build authorization request
    let authorize_request = AuthorizeRequest {
        dex_config: app_state.dex_config.clone(),
        org_config,
        return_url: request.return_url.unwrap_or_else(|| "/dashboard".to_string()),
        client_ip,
        client_user_agent: user_agent,
    };
    
    // 4. Create auth builder and generate authorization URL
    let auth_builder = app_state
        .create_auth_builder()
        .map_err(|e| AppError::InternalError(format!("Failed to create auth builder: {}", e)))?;
    
    let authorize_url = auth_builder
        .build_authorize_url(authorize_request)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to build auth URL: {}", e)))?;
    
    Ok(Json(LoginResponse { authorize_url }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract client IP address from request headers
fn extract_client_ip(headers: &HeaderMap) -> String {
    // Check for X-Forwarded-For header (if behind proxy)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // Check for X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Fallback to unknown
    "unknown".to_string()
}

/// Extract user agent from request headers
fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// Get organization configuration from database by subdomain
/// 
/// # Database Query Example
/// ```sql
/// SELECT 
///     org_id,
///     subdomain,
///     dex_connector_id,
///     auth0_organization_id,
///     session_secret,
///     pkce_required,
///     max_age_seconds,
///     prompt,
///     additional_params
/// FROM organizations
/// WHERE subdomain = $1 AND active = true
/// ```
async fn get_org_config_by_subdomain(
    db: &sqlx::PgPool,
    subdomain: &str,
) -> anyhow::Result<OrgAuthConfig> {
    let row = sqlx::query_as::<_, OrgAuthConfigRow>(
        r#"
        SELECT 
            org_id,
            subdomain,
            dex_connector_id,
            auth0_organization_id,
            session_secret,
            pkce_required,
            max_age_seconds,
            prompt,
            additional_params
        FROM organizations
        WHERE subdomain = $1 AND active = true
        "#,
    )
    .bind(subdomain)
    .fetch_one(db)
    .await?;
    
    Ok(row.into())
}

/// Database row structure for organization configuration
#[derive(sqlx::FromRow)]
struct OrgAuthConfigRow {
    org_id: String,
    subdomain: String,
    dex_connector_id: String,
    auth0_organization_id: Option<String>,
    session_secret: String,
    pkce_required: bool,
    max_age_seconds: i32,
    prompt: Option<String>,
    additional_params: Option<sqlx::types::JsonValue>,
}

impl From<OrgAuthConfigRow> for OrgAuthConfig {
    fn from(row: OrgAuthConfigRow) -> Self {
        Self {
            org_id: row.org_id,
            subdomain: row.subdomain,
            dex_connector_id: row.dex_connector_id,
            auth0_organization_id: row.auth0_organization_id,
            session_secret: row.session_secret,
            pkce_required: row.pkce_required,
            max_age_seconds: row.max_age_seconds as u64,
            prompt: row.prompt,
            additional_params: row
                .additional_params
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
        }
    }
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
        };
        
        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });
        
        (status, body).into_response()
    }
}

// ============================================================================
// Helper Middleware/Extractors
// ============================================================================

/// Extract organization subdomain from Host header
/// 
/// # Example
/// Host: acme.example.com -> "acme"
/// Host: globex.example.com -> "globex"
pub fn extract_subdomain_from_host(host: &str) -> Option<String> {
    // Expected format: <subdomain>.example.com
    // For development: <subdomain>.localhost:5001
    
    let parts: Vec<&str> = host.split('.').collect();
    
    if parts.len() >= 2 {
        // Return the first part as subdomain
        Some(parts[0].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_subdomain() {
        assert_eq!(
            extract_subdomain_from_host("acme.example.com"),
            Some("acme".to_string())
        );
        assert_eq!(
            extract_subdomain_from_host("globex.example.com"),
            Some("globex".to_string())
        );
        assert_eq!(
            extract_subdomain_from_host("localhost:5001"),
            Some("localhost:5001".to_string())
        );
        assert_eq!(extract_subdomain_from_host("localhost"), None);
    }
}


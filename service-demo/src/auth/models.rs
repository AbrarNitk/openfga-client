/// Database models for authentication
/// 
/// This module contains the database models for users, sessions, and related entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// User Model
// ============================================================================

/// User model stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// Unique user ID
    pub user_id: String,
    
    /// User email address
    pub email: String,
    
    /// User's full name
    pub name: Option<String>,
    
    /// User's display name
    pub display_name: Option<String>,
    
    /// Profile picture URL
    pub picture: Option<String>,
    
    /// Authentication provider (e.g., "auth0", "google")
    pub auth_provider: String,
    
    /// Provider-specific user ID (the 'sub' claim from ID token)
    pub provider_user_id: String,
    
    /// Organization ID
    pub org_id: String,
    
    /// Access token (encrypted at rest in production)
    pub access_token: Option<String>,
    
    /// Refresh token (encrypted at rest in production)
    pub refresh_token: Option<String>,
    
    /// ID token (encrypted at rest in production)
    pub id_token: Option<String>,
    
    /// Token expiration time
    pub token_expires_at: Option<DateTime<Utc>>,
    
    /// Whether user is active
    pub is_active: bool,
    
    /// When user was created
    pub created_at: DateTime<Utc>,
    
    /// Last login timestamp
    pub last_login_at: DateTime<Utc>,
    
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// User creation data
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub picture: Option<String>,
    pub auth_provider: String,
    pub provider_user_id: String,
    pub org_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
}

/// User token update data
#[derive(Debug, Clone)]
pub struct UpdateUserTokens {
    pub user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// User Session Model
// ============================================================================

/// User session stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    /// Unique session ID
    pub session_id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Organization ID
    pub org_id: String,
    
    /// Client IP address
    pub ip_address: String,
    
    /// User agent string
    pub user_agent: String,
    
    /// Whether session is active
    pub is_active: bool,
    
    /// When session was created
    pub created_at: DateTime<Utc>,
    
    /// When session expires
    pub expires_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,
}

/// Session creation data
#[derive(Debug, Clone)]
pub struct CreateSession {
    pub session_id: String,
    pub user_id: String,
    pub org_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub expires_at: DateTime<Utc>,
}

// ============================================================================
// Session Configuration (part of Organization)
// ============================================================================

/// Session configuration stored at organization level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Cookie name (e.g., "session_id", "auth_token")
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    
    /// Cookie domain (e.g., ".example.com" for subdomain sharing)
    pub cookie_domain: Option<String>,
    
    /// Whether cookie should only be sent over HTTPS
    #[serde(default = "default_secure")]
    pub secure: bool,
    
    /// Whether cookie should be HTTP-only (not accessible via JavaScript)
    #[serde(default = "default_http_only")]
    pub http_only: bool,
    
    /// SameSite cookie attribute
    #[serde(default = "default_same_site")]
    pub same_site: SameSitePolicy,
    
    /// Session duration in seconds
    #[serde(default = "default_max_age")]
    pub max_age_seconds: i64,
    
    /// Secret for signing cookies (should be encrypted at rest, rotatable)
    pub cookie_signing_secret: String,
    
    /// Whether to extend session on activity (sliding expiration)
    #[serde(default = "default_session_extension")]
    pub session_extension_enabled: bool,
    
    /// Threshold for session extension (e.g., 0.5 = extend when 50% expired)
    #[serde(default = "default_extension_threshold")]
    pub session_extension_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

impl std::fmt::Display for SameSitePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSitePolicy::Strict => write!(f, "Strict"),
            SameSitePolicy::Lax => write!(f, "Lax"),
            SameSitePolicy::None => write!(f, "None"),
        }
    }
}

fn default_cookie_name() -> String {
    "session_id".to_string()
}

fn default_secure() -> bool {
    true // Always secure in production
}

fn default_http_only() -> bool {
    true // Always HTTP-only for security
}

fn default_same_site() -> SameSitePolicy {
    SameSitePolicy::Lax // Good default for most cases
}

fn default_max_age() -> i64 {
    86400 // 24 hours
}

fn default_session_extension() -> bool {
    true // Enable sliding expiration
}

fn default_extension_threshold() -> f64 {
    0.5 // Extend when 50% of session time has elapsed
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: default_cookie_name(),
            cookie_domain: None,
            secure: default_secure(),
            http_only: default_http_only(),
            same_site: default_same_site(),
            max_age_seconds: default_max_age(),
            cookie_signing_secret: String::new(), // Must be set
            session_extension_enabled: default_session_extension(),
            session_extension_threshold: default_extension_threshold(),
        }
    }
}

// ============================================================================
// Token Response from Dex
// ============================================================================

/// Token response from Dex token endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub id_token: String,
    pub scope: Option<String>,
}

/// ID Token claims (basic claims without signature verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    /// Subject (user ID from provider)
    pub sub: String,
    
    /// Issuer
    pub iss: String,
    
    /// Audience
    pub aud: String,
    
    /// Expiration time
    pub exp: i64,
    
    /// Issued at
    pub iat: i64,
    
    /// Nonce (must match the one we sent)
    pub nonce: String,
    
    /// Email
    pub email: Option<String>,
    
    /// Email verified
    pub email_verified: Option<bool>,
    
    /// Name
    pub name: Option<String>,
    
    /// Picture URL
    pub picture: Option<String>,
    
    /// Preferred username
    pub preferred_username: Option<String>,
}

// ============================================================================
// Database Schema SQL
// ============================================================================

/// SQL schema for users table
pub const USERS_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    name TEXT,
    display_name TEXT,
    picture TEXT,
    auth_provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    org_id TEXT NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    id_token TEXT,
    token_expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(org_id, provider_user_id, auth_provider)
);

CREATE INDEX IF NOT EXISTS idx_users_org_id ON users(org_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider_user_id, auth_provider);
"#;

/// SQL schema for user_sessions table
pub const USER_SESSIONS_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS user_sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    org_id TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_org_id ON user_sessions(org_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON user_sessions(is_active) WHERE is_active = TRUE;
"#;


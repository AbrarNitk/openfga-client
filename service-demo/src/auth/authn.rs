use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use oauth2::{CsrfToken, PkceCodeChallenge, PkceCodeVerifier};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// Configuration Models
// ============================================================================

/// Dex application-level configuration
/// This is loaded from config file and shared across all organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexAppConfig {
    /// Dex client ID (single application for all orgs)
    pub client_id: String,
    
    /// Dex client secret
    pub client_secret: String,
    
    /// Dex issuer URL
    pub issuer_url: String,
    
    /// Dex authorization endpoint
    pub auth_url: String,
    
    /// Dex token endpoint
    pub token_url: String,
    
    /// Callback URL for OAuth flow
    pub redirect_url: String,
    
    /// Default OAuth scopes
    pub scopes: Vec<String>,
}

impl DexAppConfig {
    /// Create DexAppConfig from context DexConfig
    /// This helps integrate with the existing context.rs configuration
    pub fn from_context_config(config: &crate::context::DexConfig) -> Self {
        Self {
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            issuer_url: config.issuer_url.clone(),
            // Derive auth_url from issuer_url if not explicitly provided
            auth_url: format!("{}/authorize", config.issuer_url.trim_end_matches('/')),
            token_url: config.token_url.clone(),
            redirect_url: config.redirect_url.clone(),
            scopes: config.scopes.clone(),
        }
    }
}

/// Organization-specific authentication configuration
/// This is stored in the database per organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgAuthConfig {
    /// Organization ID
    pub org_id: String,
    
    /// Organization subdomain (e.g., "acme" for acme.example.com)
    pub subdomain: String,
    
    /// Dex connector ID (e.g., "auth0", "google")
    pub dex_connector_id: String,
    
    /// Auth0 organization ID (only if using Auth0 connector)
    pub auth0_organization_id: Option<String>,
    
    /// Session secret for signing state (should be rotatable)
    /// In production, this should be encrypted at rest
    pub session_secret: String,
    
    /// Session configuration for this organization
    pub session_config: crate::auth::models::SessionConfig,
    
    /// Whether PKCE is required (should always be true for security)
    #[serde(default = "default_pkce_required")]
    pub pkce_required: bool,
    
    /// Max age in seconds for the auth request (default: 300 = 5 minutes)
    #[serde(default = "default_max_age")]
    pub max_age_seconds: u64,
    
    /// Prompt parameter (e.g., "login", "consent", "none")
    pub prompt: Option<String>,
    
    /// Additional custom parameters for the authorization request
    #[serde(default)]
    pub additional_params: std::collections::HashMap<String, String>,
}

fn default_pkce_required() -> bool {
    true
}

fn default_max_age() -> u64 {
    300 // 5 minutes
}

// ============================================================================
// Authentication State Management
// ============================================================================

/// Authentication state stored in Redis for the OAuth flow
/// This is stored temporarily (5-10 minutes TTL) and retrieved during callback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    /// Organization ID
    pub org_id: String,
    
    /// User session ID (generated at the start of auth flow)
    pub user_session_id: String,
    
    /// Nonce for ID token validation (prevents replay attacks)
    pub nonce: String,
    
    /// PKCE code verifier (must match code_challenge sent to auth server)
    pub code_verifier: String,
    
    /// Return URL (where to redirect after successful auth)
    pub return_url: String,
    
    /// Timestamp when state was created (Unix epoch seconds)
    pub created_at: u64,
    
    /// Timestamp when state expires (Unix epoch seconds)
    pub expires_at: u64,
    
    /// Optional CSRF token for additional security
    pub csrf_token: Option<String>,
    
    /// Client IP address (for validation during callback)
    pub ip_address: String,
    
    /// User agent hash (for additional security validation)
    pub user_agent_hash: String,
}

impl AuthState {
    /// Create a new auth state with security parameters
    pub fn new(
        org_id: String,
        return_url: String,
        ip_address: String,
        user_agent: String,
        ttl_seconds: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate security tokens using oauth2 crate's secure random generators
        let (_, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let nonce = CsrfToken::new_random(); // Using CsrfToken for nonce generation
        let csrf_token = CsrfToken::new_random();
        
        Self {
            org_id,
            user_session_id: generate_session_id(),
            nonce: nonce.secret().clone(),
            code_verifier: pkce_verifier.secret().clone(),
            return_url,
            created_at: now,
            expires_at: now + ttl_seconds,
            csrf_token: Some(csrf_token.secret().clone()),
            ip_address,
            user_agent_hash: hash_user_agent(&user_agent),
        }
    }
    
    /// Check if the state has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
    
    /// Validate the state against request context
    pub fn validate(&self, ip_address: &str, user_agent: &str) -> Result<()> {
        if self.is_expired() {
            anyhow::bail!("Auth state has expired");
        }
        
        if self.ip_address != ip_address {
            anyhow::bail!("IP address mismatch");
        }
        
        let ua_hash = hash_user_agent(user_agent);
        if self.user_agent_hash != ua_hash {
            anyhow::bail!("User agent mismatch");
        }
        
        Ok(())
    }
}

// ============================================================================
// Signed State Management
// ============================================================================

/// Signed state that is sent to the authorization server
/// This prevents tampering and includes integrity verification
#[derive(Debug, Serialize, Deserialize)]
struct SignedState {
    /// Unique state identifier (used as Redis key)
    state_id: String,
    
    /// Timestamp when signed
    timestamp: u64,
    
    /// HMAC signature of state_id + timestamp
    signature: String,
}

impl SignedState {
    /// Create a new signed state
    fn new(state_id: String, secret: &str) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let signature = Self::compute_signature(&state_id, timestamp, secret)?;
        
        Ok(Self {
            state_id,
            timestamp,
            signature,
        })
    }
    
    /// Compute HMAC signature
    fn compute_signature(state_id: &str, timestamp: u64, secret: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .context("Failed to create HMAC")?;
        
        mac.update(state_id.as_bytes());
        mac.update(&timestamp.to_le_bytes());
        
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }
    
    /// Verify the signature
    fn verify(&self, secret: &str) -> Result<()> {
        let expected_sig = Self::compute_signature(&self.state_id, self.timestamp, secret)?;
        
        if self.signature != expected_sig {
            anyhow::bail!("Invalid state signature");
        }
        
        Ok(())
    }
    
    /// Encode to base64url string
    fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }
    
    /// Decode from base64url string
    fn decode(encoded: &str, secret: &str) -> Result<Self> {
        let decoded = URL_SAFE_NO_PAD.decode(encoded)
            .context("Failed to decode state")?;
        
        let signed_state: SignedState = serde_json::from_slice(&decoded)
            .context("Failed to parse signed state")?;
        
        signed_state.verify(secret)?;
        
        Ok(signed_state)
    }
}

// ============================================================================
// Redis State Cache
// ============================================================================

/// Redis connection pool for state management
pub struct StateCache {
    client: redis::Client,
}

impl StateCache {
    /// Create a new state cache
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client")?;
        Ok(Self { client })
    }
    
    /// Store auth state in Redis with TTL
    pub async fn store(&self, state: &AuthState) -> Result<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let state_id = generate_session_id();
        let key = format!("auth:state:{}", state_id);
        let ttl = (state.expires_at - state.created_at) as i64;
        
        let json = serde_json::to_string(state)
            .context("Failed to serialize state")?;
        
        let _: () = conn.set_ex(&key, json, ttl as u64).await
            .context("Failed to store state in Redis")?;
        
        Ok(state_id)
    }
    
    /// Retrieve and validate auth state from Redis
    pub async fn retrieve(&self, state_id: &str) -> Result<Option<AuthState>> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let key = format!("auth:state:{}", state_id);
        let json: Option<String> = conn.get(&key).await
            .context("Failed to retrieve state from Redis")?;
        
        match json {
            Some(data) => {
                let state: AuthState = serde_json::from_str(&data)
                    .context("Failed to parse state from Redis")?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }
    
    /// Invalidate (delete) auth state from Redis
    pub async fn invalidate(&self, state_id: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let key = format!("auth:state:{}", state_id);
        let _: () = conn.del(&key).await
            .context("Failed to delete state from Redis")?;
        
        Ok(())
    }
}

// ============================================================================
// Authorization URL Builder
// ============================================================================

/// Request parameters for authorization URL generation
pub struct AuthorizeRequest {
    /// Dex application configuration
    pub dex_config: DexAppConfig,
    
    /// Organization configuration
    pub org_config: OrgAuthConfig,
    
    /// Return URL (where to redirect after successful auth)
    pub return_url: String,
    
    /// Client IP address
    pub client_ip: String,
    
    /// Client user agent
    pub client_user_agent: String,
}

/// Authorization URL builder with security parameters
pub struct AuthorizationUrlBuilder {
    state_cache: StateCache,
}

impl AuthorizationUrlBuilder {
    /// Create a new authorization URL builder
    pub fn new(redis_url: &str) -> Result<Self> {
        Ok(Self {
            state_cache: StateCache::new(redis_url)?,
        })
    }
    
    /// Generate a secure authorization URL for the organization
    pub async fn build_authorize_url(&self, request: AuthorizeRequest) -> Result<String> {
        let dex_config = &request.dex_config;
        let org_config = &request.org_config;
        
        // 1. Create auth state with all security parameters
        let auth_state = AuthState::new(
            org_config.org_id.clone(),
            request.return_url,
            request.client_ip,
            request.client_user_agent,
            org_config.max_age_seconds,
        );
        
        // 2. Generate PKCE challenge from verifier
        let pkce_verifier = PkceCodeVerifier::new(auth_state.code_verifier.clone());
        let pkce_challenge = PkceCodeChallenge::from_code_verifier_sha256(&pkce_verifier);
        
        // 3. Store state in Redis and get state ID
        let state_id = self.state_cache.store(&auth_state).await?;
        
        // 4. Create signed state parameter
        let signed_state = SignedState::new(state_id, &org_config.session_secret)?;
        let state_param = signed_state.encode()?;
        
        // 5. Build authorization URL with all parameters
        let mut auth_url = Url::parse(&dex_config.auth_url)
            .context("Invalid Dex auth URL")?;
        
        {
            let mut query = auth_url.query_pairs_mut();
            
            // Standard OAuth2/OIDC parameters
            query.append_pair("client_id", &dex_config.client_id);
            query.append_pair("redirect_uri", &dex_config.redirect_url);
            query.append_pair("response_type", "code");
            query.append_pair("scope", &dex_config.scopes.join(" "));
            query.append_pair("state", &state_param);
            query.append_pair("nonce", &auth_state.nonce);
            
            // PKCE parameters (required for security)
            if org_config.pkce_required {
                query.append_pair("code_challenge", pkce_challenge.as_str());
                query.append_pair("code_challenge_method", "S256");
            }
            
            // Dex-specific parameter: connector selection
            query.append_pair("connector_id", &org_config.dex_connector_id);
            
            // Auth0-specific parameter: organization
            if let Some(auth0_org_id) = &org_config.auth0_organization_id {
                query.append_pair("organization", auth0_org_id);
            }
            
            // Optional parameters
            if let Some(prompt) = &org_config.prompt {
                query.append_pair("prompt", prompt);
            }
            
            if org_config.max_age_seconds > 0 {
                query.append_pair("max_age", &org_config.max_age_seconds.to_string());
            }
            
            // Additional custom parameters
            for (key, value) in &org_config.additional_params {
                query.append_pair(key, value);
            }
        }
        
        Ok(auth_url.to_string())
    }
    
    /// Retrieve and validate auth state from signed state parameter
    pub async fn retrieve_auth_state(
        &self,
        state_param: &str,
        org_config: &OrgAuthConfig,
        client_ip: &str,
        client_user_agent: &str,
    ) -> Result<AuthState> {
        // 1. Decode and verify signed state
        let signed_state = SignedState::decode(state_param, &org_config.session_secret)
            .context("Failed to verify state signature")?;
        
        // 2. Retrieve state from Redis
        let auth_state = self.state_cache
            .retrieve(&signed_state.state_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Auth state not found or expired"))?;
        
        // 3. Validate state against request context
        auth_state.validate(client_ip, client_user_agent)
            .context("State validation failed")?;
        
        // 4. Ensure org_id matches
        if auth_state.org_id != org_config.org_id {
            anyhow::bail!("Organization ID mismatch");
        }
        
        Ok(auth_state)
    }
    
    /// Consume and invalidate auth state (call this after successful token exchange)
    pub async fn consume_auth_state(&self, state_param: &str, org_config: &OrgAuthConfig) -> Result<()> {
        let signed_state = SignedState::decode(state_param, &org_config.session_secret)?;
        self.state_cache.invalidate(&signed_state.state_id).await
    }
}


// ============================================================================
// Security Utilities
// ============================================================================

/// Generate a unique session ID using oauth2's CsrfToken for randomness
fn generate_session_id() -> String {
    // Using CsrfToken for cryptographically secure random string generation
    CsrfToken::new_random().secret().clone()
}

/// Hash user agent for privacy and validation
fn hash_user_agent(user_agent: &str) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(user_agent.as_bytes());
    hex::encode(hasher.finalize())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oauth2_random_generators() {
        // Test that oauth2's random generators create unique values
        let token1 = CsrfToken::new_random();
        let token2 = CsrfToken::new_random();
        assert_ne!(token1.secret(), token2.secret());
        assert!(!token1.secret().is_empty());
        
        // Test PKCE verifier generation
        let (_, verifier1) = PkceCodeChallenge::new_random_sha256();
        let (_, verifier2) = PkceCodeChallenge::new_random_sha256();
        assert_ne!(verifier1.secret(), verifier2.secret());
        assert!(verifier1.secret().len() >= 43); // PKCE requirement
    }
    
    #[test]
    fn test_signed_state_roundtrip() {
        let state_id = generate_session_id();
        let secret = "test-secret-key";
        
        let signed = SignedState::new(state_id.clone(), secret).unwrap();
        let encoded = signed.encode().unwrap();
        let decoded = SignedState::decode(&encoded, secret).unwrap();
        
        assert_eq!(signed.state_id, decoded.state_id);
    }
    
    #[test]
    fn test_signed_state_invalid_signature() {
        let state_id = generate_session_id();
        let secret = "test-secret-key";
        let wrong_secret = "wrong-secret";
        
        let signed = SignedState::new(state_id, secret).unwrap();
        let encoded = signed.encode().unwrap();
        
        let result = SignedState::decode(&encoded, wrong_secret);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_auth_state_expiration() {
        let state = AuthState::new(
            "org-123".to_string(),
            "/dashboard".to_string(),
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            0, // Already expired
        );
        
        assert!(state.is_expired());
    }
    
    #[test]
    fn test_auth_state_security_tokens() {
        let state = AuthState::new(
            "org-123".to_string(),
            "/dashboard".to_string(),
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            300,
        );
        
        // Verify all security tokens are generated
        assert!(!state.nonce.is_empty());
        assert!(!state.code_verifier.is_empty());
        assert!(state.csrf_token.is_some());
        assert!(!state.user_session_id.is_empty());
        assert!(!state.user_agent_hash.is_empty());
    }
}


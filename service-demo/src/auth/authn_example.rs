/// Example usage of the secure multi-tenant authentication module
/// 
/// This example demonstrates how to:
/// 1. Configure organization authentication settings
/// 2. Generate secure authorization URLs with PKCE, nonce, and signed state
/// 3. Handle OAuth callbacks with state validation
/// 
/// # Security Features Implemented:
/// 
/// - **PKCE (Proof Key for Code Exchange)**: Prevents authorization code interception attacks
/// - **Nonce**: Prevents replay attacks in ID tokens
/// - **CSRF Token**: Protects against cross-site request forgery
/// - **Signed State**: Uses HMAC-SHA256 to prevent state tampering
/// - **Redis State Cache**: Short-lived state storage with automatic expiration
/// - **IP & User-Agent Validation**: Ensures callback matches original request
/// 
/// # Example Flow:
/// 
/// ```rust,ignore
/// use service_demo::auth::authn::{
///     AuthorizationUrlBuilder, AuthorizeRequest, OrgAuthConfig,
/// };
/// 
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // 1. Load organization configuration from database
///     let org_config = OrgAuthConfig {
///         org_id: "org-12345".to_string(),
///         subdomain: "acme".to_string(),
///         auth0_organization_id: Some("org_xyz123".to_string()),
///         dex_client_id: "auth0-app".to_string(),
///         dex_client_secret: "secret-key".to_string(),
///         dex_connector_id: "auth0".to_string(),
///         dex_issuer_url: "https://dex.example.com".to_string(),
///         dex_auth_url: "https://dex.example.com/authorize".to_string(),
///         dex_token_url: "https://dex.example.com/token".to_string(),
///         callback_url: "https://acme.example.com/auth/callback".to_string(),
///         scopes: vec![
///             "openid".to_string(),
///             "profile".to_string(),
///             "email".to_string(),
///             "offline_access".to_string(),
///         ],
///         session_secret: "your-session-secret-key-min-32-chars".to_string(),
///         pkce_required: true,
///         max_age_seconds: 300, // 5 minutes
///         prompt: Some("login".to_string()),
///         additional_params: std::collections::HashMap::new(),
///     };
/// 
///     // 2. Create authorization URL builder with Redis connection
///     let redis_url = "redis://127.0.0.1:6379";
///     let auth_builder = AuthorizationUrlBuilder::new(redis_url)?;
/// 
///     // 3. Build authorization URL
///     let authorize_request = AuthorizeRequest {
///         org_config: org_config.clone(),
///         return_url: "/dashboard".to_string(),
///         client_ip: "192.168.1.100".to_string(),
///         client_user_agent: "Mozilla/5.0...".to_string(),
///     };
/// 
///     let auth_url = auth_builder.build_authorize_url(authorize_request).await?;
///     println!("Redirect user to: {}", auth_url);
/// 
///     // The generated URL will look like:
///     // https://dex.example.com/authorize?
///     //   client_id=auth0-app
///     //   &redirect_uri=https://acme.example.com/auth/callback
///     //   &response_type=code
///     //   &scope=openid+profile+email+offline_access
///     //   &state=<signed-base64url-encoded-state>
///     //   &nonce=<random-nonce>
///     //   &code_challenge=<sha256-challenge>
///     //   &code_challenge_method=S256
///     //   &connector_id=auth0
///     //   &organization=org_xyz123
///     //   &prompt=login
///     //   &max_age=300
/// 
///     // 4. Handle callback (when user returns from IdP)
///     let state_param = "received-from-callback"; // From query parameter
///     let client_ip = "192.168.1.100"; // From request
///     let client_user_agent = "Mozilla/5.0..."; // From request headers
/// 
///     let auth_state = auth_builder
///         .retrieve_auth_state(
///             state_param,
///             &org_config,
///             client_ip,
///             client_user_agent,
///         )
///         .await?;
/// 
///     // 5. Exchange authorization code for tokens (using code_verifier from auth_state)
///     // ... token exchange logic ...
/// 
///     // 6. Invalidate the state after successful token exchange
///     auth_builder.consume_auth_state(state_param, &org_config).await?;
/// 
///     Ok(())
/// }
/// ```
/// 
/// # Database Schema Example:
/// 
/// ```sql
/// CREATE TABLE organizations (
///     id TEXT PRIMARY KEY,
///     name TEXT NOT NULL,
///     subdomain TEXT UNIQUE NOT NULL,
///     
///     -- Auth configuration
///     auth0_organization_id TEXT,
///     dex_client_id TEXT NOT NULL,
///     dex_client_secret TEXT NOT NULL, -- Encrypted at rest
///     dex_connector_id TEXT NOT NULL,
///     dex_issuer_url TEXT NOT NULL,
///     dex_auth_url TEXT NOT NULL,
///     dex_token_url TEXT NOT NULL,
///     callback_url TEXT NOT NULL,
///     scopes TEXT[] NOT NULL,
///     session_secret TEXT NOT NULL, -- Encrypted at rest, rotatable
///     pkce_required BOOLEAN DEFAULT TRUE,
///     max_age_seconds INTEGER DEFAULT 300,
///     prompt TEXT,
///     additional_params JSONB,
///     
///     created_at TIMESTAMPTZ DEFAULT NOW(),
///     updated_at TIMESTAMPTZ DEFAULT NOW()
/// );
/// 
/// CREATE INDEX idx_organizations_subdomain ON organizations(subdomain);
/// ```
/// 
/// # Axum Handler Example:
/// 
/// ```rust,ignore
/// use axum::{
///     extract::{Query, State},
///     response::{IntoResponse, Redirect},
/// };
/// use serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct LoginQuery {
///     return_url: Option<String>,
/// }
/// 
/// async fn login_handler(
///     State(app_state): State<AppState>,
///     Query(query): Query<LoginQuery>,
///     // Extract from headers:
///     client_ip: String,
///     user_agent: String,
///     // Extract from host/subdomain:
///     org_subdomain: String,
/// ) -> Result<impl IntoResponse, AppError> {
///     // 1. Lookup organization by subdomain
///     let org_config = app_state
///         .db
///         .get_org_by_subdomain(&org_subdomain)
///         .await?;
/// 
///     // 2. Build authorization URL
///     let auth_request = AuthorizeRequest {
///         org_config,
///         return_url: query.return_url.unwrap_or_else(|| "/dashboard".to_string()),
///         client_ip,
///         client_user_agent: user_agent,
///     };
/// 
///     let auth_url = app_state
///         .auth_builder
///         .build_authorize_url(auth_request)
///         .await?;
/// 
///     Ok(Redirect::to(&auth_url))
/// }
/// 
/// #[derive(Deserialize)]
/// struct CallbackQuery {
///     code: String,
///     state: String,
/// }
/// 
/// async fn callback_handler(
///     State(app_state): State<AppState>,
///     Query(query): Query<CallbackQuery>,
///     client_ip: String,
///     user_agent: String,
///     org_subdomain: String,
/// ) -> Result<impl IntoResponse, AppError> {
///     // 1. Lookup organization
///     let org_config = app_state
///         .db
///         .get_org_by_subdomain(&org_subdomain)
///         .await?;
/// 
///     // 2. Retrieve and validate auth state
///     let auth_state = app_state
///         .auth_builder
///         .retrieve_auth_state(&query.state, &org_config, &client_ip, &user_agent)
///         .await?;
/// 
///     // 3. Exchange authorization code for tokens
///     let tokens = exchange_code_for_tokens(
///         &query.code,
///         &auth_state.code_verifier,
///         &org_config,
///     )
///     .await?;
/// 
///     // 4. Verify nonce in ID token matches auth_state.nonce
///     verify_id_token_nonce(&tokens.id_token, &auth_state.nonce)?;
/// 
///     // 5. Create user session
///     let session = create_user_session(&tokens, &auth_state).await?;
/// 
///     // 6. Invalidate auth state
///     app_state
///         .auth_builder
///         .consume_auth_state(&query.state, &org_config)
///         .await?;
/// 
///     // 7. Redirect to return URL
///     Ok(Redirect::to(&auth_state.return_url))
/// }
/// ```
/// 
/// # Security Best Practices:
/// 
/// 1. **Store session secrets encrypted at rest**
/// 2. **Rotate session secrets periodically** (implement secret versioning)
/// 3. **Use HTTPS only** for all OAuth flows
/// 4. **Set appropriate Redis TTL** (5-10 minutes for auth state)
/// 5. **Validate all inputs** from OAuth callbacks
/// 6. **Log authentication events** for audit trails
/// 7. **Implement rate limiting** on login endpoints
/// 8. **Use secure cookie attributes** (HttpOnly, Secure, SameSite=Strict)
/// 9. **Verify ID token signatures** from the IdP
/// 10. **Implement session timeout** and sliding expiration
/// 
/// # Dex Configuration Example:
/// 
/// ```yaml
/// staticClients:
/// - id: auth0-app
///   redirectURIs:
///   - 'https://acme.example.com/auth/callback'
///   - 'https://globex.example.com/auth/callback'
///   name: 'Multi-Tenant OAuth Application'
///   secret: <client-secret>
///   response_types: [code]
///   scopes: [openid, profile, email, offline_access]
/// 
/// connectors:
/// - type: oidc
///   id: auth0
///   name: Auth0
///   config:
///     issuer: https://your-tenant.auth0.com/
///     clientID: <auth0-client-id>
///     clientSecret: <auth0-client-secret>
///     redirectURI: https://dex.example.com/callback
///     scopes: [openid, profile, email, offline_access]
/// ```
/// 
/// # Redis Configuration:
/// 
/// - Use Redis 6.0+ for better security features
/// - Enable TLS for Redis connections in production
/// - Set maxmemory-policy to `volatile-lru` for automatic eviction
/// - Use Redis Sentinel or Cluster for high availability
/// 
/// # Monitoring & Alerts:
/// 
/// - Monitor failed authentication attempts
/// - Track auth state cache hit/miss rates
/// - Alert on unusual patterns (geographic anomalies, rapid attempts)
/// - Log all state validation failures
/// - Track token exchange success rates

#![allow(dead_code)]

pub fn example_usage_documentation() {
    println!("See the module documentation for comprehensive examples");
}


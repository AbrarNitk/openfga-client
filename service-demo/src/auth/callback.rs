/// OAuth Callback Handler
/// 
/// Handles the OAuth callback with token exchange, user creation/update, and session management

use super::authn::{AuthorizationUrlBuilder, DexAppConfig, OrgAuthConfig};
use super::db_ops;
use super::models::{CreateSession, CreateUser, UpdateUserTokens};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims, CoreProviderMetadata, CoreTokenResponse},
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeVerifier, RedirectUrl,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// Callback Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct CallbackResult {
    pub user_id: String,
    pub session_id: String,
    pub return_url: String,
}

// ============================================================================
// Token Exchange with ID Token Verification
// ============================================================================

/// Exchange authorization code for tokens with automatic ID token signature verification
pub async fn exchange_code_for_tokens(
    dex_config: &DexAppConfig,
    code: &str,
    code_verifier: &str,
    expected_nonce: &str,
) -> Result<(CoreTokenResponse, CoreIdTokenClaims)> {
    // Create HTTP client
    let http_client = HttpClient::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Failed to build HTTP client")?;
    
    // Parse issuer URL and discover provider metadata
    let issuer_url = IssuerUrl::new(dex_config.issuer_url.clone())
        .context("Invalid issuer URL")?;
    
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .context("Failed to discover provider metadata")?;
    
    // Create OIDC client
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(dex_config.client_id.clone()),
        Some(ClientSecret::new(dex_config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(dex_config.redirect_url.clone())
            .context("Invalid redirect URL")?,
    );
    
    // Exchange authorization code for tokens with PKCE
    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .context("Failed to create code exchange request")?
        .set_pkce_verifier(PkceCodeVerifier::new(code_verifier.to_string()))
        .request_async(&http_client)
        .await
        .context("Failed to exchange authorization code for tokens")?;
    
    // Get ID token (this is already parsed and basic validation is done)           
    let id_token = token_response.extra_fields().id_token().ok_or_else(|| anyhow::anyhow!("Server did not return an ID token"))?;
    
    // Verify ID token signature and claims using JWKS
    let id_token_verifier = client.id_token_verifier();
    let nonce_verifier = Nonce::new(expected_nonce.to_string());
    
    let claims = id_token
        .claims(&id_token_verifier, &nonce_verifier)
        .context("Failed to verify ID token")?
        .clone();
    
    Ok((token_response.clone(), claims))
}

// ============================================================================
// Claims Extraction
// ============================================================================

/// Extract user information from verified ID token claims
pub fn extract_user_info(claims: &CoreIdTokenClaims) -> (String, Option<String>, Option<String>, Option<String>) {
    let email = claims
        .email()
        .map(|e| e.as_str().to_string());
    
    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.as_str().to_string());
    
    let picture = claims
        .picture()
        .and_then(|p| p.get(None))
        .map(|p| p.as_str().to_string());
    
    let preferred_username = claims
        .preferred_username()
        .map(|u| u.as_str().to_string());
    
    (
        email.unwrap_or_else(|| format!("{}@unknown", claims.subject().as_str())),
        name,
        picture,
        preferred_username,
    )
}

// ============================================================================
// User Management
// ============================================================================

/// Create or update user from verified ID token claims
pub async fn create_or_update_user(
    db: &PgPool,
    org_id: &str,
    auth_provider: &str,
    claims: &CoreIdTokenClaims,
    token_response: &CoreTokenResponse,
) -> Result<String> {
    // Extract user information from claims
    let (email, name, picture, preferred_username) = extract_user_info(claims);
    let provider_user_id = claims.subject().as_str().to_string();
    
    // Calculate token expiration
    let token_expires_at = token_response
        .expires_in()
        .map(|exp| Utc::now() + Duration::seconds(exp.as_secs() as i64));
    
    // Try to find existing user
    let existing_user = db_ops::find_user_by_provider(
        db,
        org_id,
        &provider_user_id,
        auth_provider,
    )
    .await?;
    
    // Get tokens as strings
    let access_token = token_response.access_token().secret().clone();
    let refresh_token = token_response.refresh_token().map(|t| t.secret().clone());
    let id_token = token_response
        .extra_fields()
        .id_token()
        .map(|t| t.to_string());
    
    match existing_user {
        Some(user) => {
            // Update existing user's tokens and profile
            let update = UpdateUserTokens {
                user_id: user.user_id.clone(),
                access_token: Some(access_token),
                refresh_token,
                id_token,
                token_expires_at,
            };
            
            db_ops::update_user_tokens(db, update).await?;
            
            // Update profile if information has changed
            if name.is_some() || picture.is_some() {
                db_ops::update_user_profile(
                    db,
                    &user.user_id,
                    name,
                    None, // display_name
                    picture,
                ).await?;
            }
            
            Ok(user.user_id)
        }
        None => {
            // Create new user
            let user_id = db_ops::generate_user_id();
            
            let create_user = CreateUser {
                user_id: user_id.clone(),
                email,
                name,
                display_name: preferred_username,
                picture,
                auth_provider: auth_provider.to_string(),
                provider_user_id,
                org_id: org_id.to_string(),
                access_token: Some(access_token),
                refresh_token,
                id_token,
                token_expires_at,
            };
            
            let user = db_ops::create_user(db, create_user).await?;
            Ok(user.user_id)
        }
    }
}

// ============================================================================
// Session Management
// ============================================================================

/// Create a new session for the user
pub async fn create_user_session(
    db: &PgPool,
    user_id: &str,
    org_id: &str,
    ip_address: &str,
    user_agent: &str,
    session_config: &crate::auth::models::SessionConfig,
) -> Result<String> {
    let session_id = db_ops::generate_session_id();
    let expires_at = Utc::now() + Duration::seconds(session_config.max_age_seconds);
    
    let create_session = CreateSession {
        session_id: session_id.clone(),
        user_id: user_id.to_string(),
        org_id: org_id.to_string(),
        ip_address: ip_address.to_string(),
        user_agent: user_agent.to_string(),
        expires_at,
    };
    
    db_ops::create_session(db, create_session).await?;
    
    Ok(session_id)
}

// ============================================================================
// Cookie Management
// ============================================================================

/// Sign session ID using HMAC-SHA256
fn sign_session_id(session_id: &str, secret: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .context("Failed to create HMAC for cookie signing")?;
    
    mac.update(session_id.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Create signed cookie value: session_id.signature
fn create_signed_cookie_value(session_id: &str, secret: &str) -> Result<String> {
    let signature = sign_session_id(session_id, secret)?;
    Ok(format!("{}.{}", session_id, signature))
}

/// Verify and extract session ID from signed cookie
pub fn verify_and_extract_session_id(cookie_value: &str, secret: &str) -> Result<String> {
    let parts: Vec<&str> = cookie_value.split('.').collect();
    
    if parts.len() != 2 {
        anyhow::bail!("Invalid cookie format");
    }
    
    let session_id = parts[0];
    let signature = parts[1];
    
    // Verify signature
    let expected_signature = sign_session_id(session_id, secret)?;
    
    if signature != expected_signature {
        anyhow::bail!("Invalid cookie signature");
    }
    
    Ok(session_id.to_string())
}

/// Set session cookie
pub fn set_session_cookie(
    cookies: &Cookies,
    session_id: &str,
    org_config: &OrgAuthConfig,
) -> Result<()> {
    let session_config = &org_config.session_config;
    
    // Create signed cookie value
    let cookie_value = create_signed_cookie_value(session_id, &session_config.cookie_signing_secret)?;
    
    // Build cookie
    let mut cookie = Cookie::new(session_config.cookie_name.clone(), cookie_value);
    
    // Set cookie attributes
    cookie.set_http_only(session_config.http_only);
    cookie.set_secure(session_config.secure);
    cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(
        session_config.max_age_seconds,
    ));
    
    // Set SameSite attribute
    match &session_config.same_site {
        crate::auth::models::SameSitePolicy::Strict => {
            cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
        }
        crate::auth::models::SameSitePolicy::Lax => {
            cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
        }
        crate::auth::models::SameSitePolicy::None => {
            cookie.set_same_site(tower_cookies::cookie::SameSite::None);
        }
    }
    
    // Set domain if specified
    if let Some(domain) = &session_config.cookie_domain {
        cookie.set_domain(domain.clone());
    }
    
    // Set path
    cookie.set_path("/");
    
    // Add cookie to response
    cookies.add(cookie);
    
    Ok(())
}

// ============================================================================
// Main Callback Handler Logic
// ============================================================================

/// Handle OAuth callback with complete flow
pub async fn handle_callback(
    db: &PgPool,
    dex_config: &DexAppConfig,
    org_config: &OrgAuthConfig,
    auth_builder: &AuthorizationUrlBuilder,
    query: &CallbackQuery,
    cookies: &Cookies,
    client_ip: &str,
    client_user_agent: &str,
) -> Result<CallbackResult> {
    // 1. Retrieve and validate auth state from Redis
    let auth_state = auth_builder
        .retrieve_auth_state(
            &query.state,
            org_config,
            client_ip,
            client_user_agent,
        )
        .await
        .context("Failed to retrieve or validate auth state")?;
    
    // 2. Exchange authorization code for tokens with automatic ID token verification
    // This includes:
    // - Token exchange with PKCE
    // - ID token signature verification using JWKS
    // - Nonce verification
    // - Standard claims validation (iss, aud, exp, iat)
    let (token_response, claims) = exchange_code_for_tokens(
        dex_config,
        &query.code,
        &auth_state.code_verifier,
        &auth_state.nonce,
    )
    .await
    .context("Failed to exchange code for tokens and verify ID token")?;
    
    // 3. Create or update user
    let user_id = create_or_update_user(
        db,
        &org_config.org_id,
        &org_config.dex_connector_id,
        &claims,
        &token_response,
    )
    .await
    .context("Failed to create or update user")?;
    
    // 4. Create session
    let session_id = create_user_session(
        db,
        &user_id,
        &org_config.org_id,
        client_ip,
        client_user_agent,
        &org_config.session_config,
    )
    .await
    .context("Failed to create session")?;
    
    // 5. Set session cookie
    set_session_cookie(cookies, &session_id, org_config)
        .context("Failed to set session cookie")?;
    
    // 6. Invalidate auth state (one-time use)
    auth_builder
        .consume_auth_state(&query.state, org_config)
        .await
        .context("Failed to invalidate auth state")?;
    
    Ok(CallbackResult {
        user_id,
        session_id,
        return_url: auth_state.return_url,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signed_cookie() {
        let session_id = "ses_abc123";
        let secret = "test-secret-key";
        
        let cookie_value = create_signed_cookie_value(session_id, secret).unwrap();
        let extracted = verify_and_extract_session_id(&cookie_value, secret).unwrap();
        
        assert_eq!(extracted, session_id);
        
        // Test with wrong secret
        let result = verify_and_extract_session_id(&cookie_value, "wrong-secret");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_user_info() {
        // This would require creating a CoreIdTokenClaims which is complex
        // User info extraction is tested via integration tests
    }
    
    // Note: Token exchange and ID token verification tests require
    // either mocking the OIDC provider or using integration tests with a real Dex instance
}



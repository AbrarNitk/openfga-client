use axum::extract::{Query, State};
use axum::response::IntoResponse;
use openidconnect::{
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::Mutex;

use crate::context::Ctx;

// Auth0 Configuration (Static for now - replace with your Auth0 tenant details)
const AUTH0_DOMAIN: &str = "genai-157672027117145.jp.auth0.com";
const AUTH0_CLIENT_ID: &str = "LnlvbZ4nYVqvceavKfrcgKS506Us4ze5";
const AUTH0_CLIENT_SECRET: &str =
    "zE5oX1Al14lsKlC7-bhhZruSmi42qbksDOoY1LZyPA8675jPmM_9fBO3MgdJDZ1q";
const AUTH0_REDIRECT_URL: &str = "http://127.0.0.1:5001/auth/auth0/callback";

// Structure to store state data including nonce
#[derive(Debug, Clone)]
struct StateData {
    #[allow(dead_code)]
    nonce: String,
}

// Custom Auth0 token response to handle Auth0-specific fields
#[derive(Debug, Serialize, Deserialize)]
struct Auth0TokenResponse {
    access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    id_token: String,
    token_type: String,
    expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

// Simple in-memory store for state (in production, use a proper session store)
lazy_static::lazy_static! {
    static ref STATE_STORE: Mutex<HashMap<String, StateData>> = Mutex::new(HashMap::new());
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginWithParams {
    pub connection: Option<String>, // Auth0 connection parameter (e.g., "google-oauth2", "github", etc.)
    pub screen_hint: Option<String>, // "signup" or "login" to show specific screen
    pub prompt: Option<String>,     // "login" to force re-authentication, "none" for silent auth
    pub ui_locales: Option<String>, // Language preference (e.g., "en", "es", "fr")
}

pub async fn login_with(
    State(_ctx): State<Ctx>,
    Query(params): Query<LoginWithParams>,
) -> axum::response::Response {
    // Construct Auth0 issuer URL
    let issuer_url =
        IssuerUrl::new(format!("https://{}/", AUTH0_DOMAIN)).expect("Invalid Auth0 issuer URL");

    // Create HTTP client using reqwest
    let http_client = HttpClient::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to create HTTP client");

    // Fetch provider metadata using reqwest async client
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .expect("Failed to discover Auth0 provider metadata");

    // Create OpenID Connect client
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(AUTH0_CLIENT_ID.to_string()),
        Some(ClientSecret::new(AUTH0_CLIENT_SECRET.to_string())),
    )
    .set_redirect_uri(
        RedirectUrl::new(AUTH0_REDIRECT_URL.to_string()).expect("Invalid redirect URL"),
    );

    // Generate CSRF token and nonce for state parameter
    let csrf_token = CsrfToken::new_random();
    let nonce = Nonce::new_random();

    // Store state with nonce for verification
    {
        let mut store = STATE_STORE.lock().unwrap();
        store.insert(
            csrf_token.secret().clone(),
            StateData {
                nonce: nonce.secret().clone(),
            },
        );
    }

    // Create authorization URL with scopes
    let scopes = vec![
        Scope::new("openid".to_string()),
        Scope::new("profile".to_string()),
        Scope::new("email".to_string()),
    ];

    let mut auth_url_builder = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            || csrf_token,
            || nonce,
        )
        .add_scopes(scopes);

    // Add connection parameter if provided (for social logins like Google, GitHub, etc.)
    if let Some(ref connection) = params.connection {
        auth_url_builder = auth_url_builder.add_extra_param("connection", connection);
    }

    // Add screen_hint parameter (signup or login)
    if let Some(ref screen_hint) = params.screen_hint {
        auth_url_builder = auth_url_builder.add_extra_param("screen_hint", screen_hint);
    }

    // Add prompt parameter (login, none, consent, etc.)
    if let Some(ref prompt) = params.prompt {
        auth_url_builder = auth_url_builder.add_extra_param("prompt", prompt);
    }

    // Add ui_locales parameter for language preference
    if let Some(ref ui_locales) = params.ui_locales {
        auth_url_builder = auth_url_builder.add_extra_param("ui_locales", ui_locales);
    }

    let (auth_url, _csrf_token, _nonce) = auth_url_builder.url();

    println!("Auth0 Universal Login URL: {:?}", auth_url);

    // Redirect to Auth0 Universal Login page
    // This will show Auth0's centralized login page where users can:
    // - Sign in with username/password
    // - Sign in with social connections (Google, GitHub, etc.)
    // - Sign up for a new account
    // The Universal Login page is fully customizable in your Auth0 dashboard
    axum::response::Response::builder()
        .header("Location", auth_url.to_string())
        .status(axum::http::StatusCode::FOUND)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct Auth0CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn handle_auth0_callback(
    State(_ctx): State<Ctx>,
    Query(params): Query<Auth0CallbackParams>,
) -> axum::response::Response {
    println!("Auth0 callback params: {:?}", params);

    // Retrieve state data (nonce) from state store to validate the state
    let _state_data = {
        let store = STATE_STORE.lock().unwrap();
        store.get(&params.state).cloned()
    };

    let _state_data = match _state_data {
        Some(data) => data,
        None => {
            println!("No state data found for state: {}", params.state);
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html")
                .body(axum::body::Body::from("Invalid state parameter"))
                .unwrap()
                .into_response();
        }
    };

    // Create HTTP client
    let http_client = HttpClient::new();

    // Manually exchange authorization code for tokens using Auth0's token endpoint
    let token_url = format!("https://{}/oauth/token", AUTH0_DOMAIN);

    let token_params = [
        ("grant_type", "authorization_code"),
        ("client_id", AUTH0_CLIENT_ID),
        ("client_secret", AUTH0_CLIENT_SECRET),
        ("code", &params.code),
        ("redirect_uri", AUTH0_REDIRECT_URL),
    ];

    let token_response_result = http_client
        .post(&token_url)
        .form(&token_params)
        .send()
        .await;

    let token_response_result = match token_response_result {
        Ok(resp) => resp,
        Err(e) => {
            let error_msg = format!("Failed to connect to Auth0: {}", e);
            println!("Token exchange error: {}", error_msg);

            // Clean up state from store
            {
                let mut store = STATE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            return build_error_response(&error_msg);
        }
    };

    // Get response body as text first for debugging
    let status = token_response_result.status();
    let response_text = match token_response_result.text().await {
        Ok(text) => text,
        Err(e) => {
            let error_msg = format!("Failed to read Auth0 response: {}", e);
            println!("Token exchange error: {}", error_msg);

            // Clean up state from store
            {
                let mut store = STATE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            return build_error_response(&error_msg);
        }
    };

    if !status.is_success() {
        let error_msg = format!("Auth0 returned error status {}: {}", status, response_text);
        println!("Token exchange error: {}", error_msg);

        // Clean up state from store
        {
            let mut store = STATE_STORE.lock().unwrap();
            store.remove(&params.state);
        }

        return build_error_response(&error_msg);
    }

    // Parse the token response
    let auth0_token: Auth0TokenResponse = match serde_json::from_str(&response_text) {
        Ok(token) => token,
        Err(e) => {
            let error_msg = format!(
                "Failed to parse Auth0 token response: {}. Response was: {}",
                e, response_text
            );
            println!("Token exchange error: {}", error_msg);

            // Clean up state from store
            {
                let mut store = STATE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            return build_error_response(&error_msg);
        }
    };

    // Decode ID token to extract claims
    let claims_json = match verify_id_token(&auth0_token.id_token) {
        Ok(claims_str) => claims_str,
        Err(e) => {
            println!("Warning: Failed to decode ID token claims: {}", e);
            format!("{{\"error\": \"Failed to decode claims: {}\"}}", e)
        }
    };

    // Clean up state from store
    {
        let mut store = STATE_STORE.lock().unwrap();
        store.remove(&params.state);
    }

    // Return success response with all token details
    build_success_response(
        &params.code,
        &params.state,
        &auth0_token.access_token,
        &auth0_token.refresh_token.as_deref().unwrap_or("N/A"),
        &auth0_token.id_token,
        &claims_json,
    )
}

// Helper function to decode ID token and extract claims
fn verify_id_token(id_token_str: &str) -> Result<String, String> {
    // Decode the ID token JWT to extract claims
    // Note: This does NOT verify the signature. In production, you should use a proper
    // JWT library to verify signatures and validate the token.

    let parts: Vec<&str> = id_token_str.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid ID token format".to_string());
    }

    // Decode the payload (second part)
    use base64::{Engine as _, engine::general_purpose};
    let payload = match general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to decode ID token payload: {}", e)),
    };

    // Parse JSON
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to parse ID token payload as UTF-8: {}", e)),
    };

    // Parse as JSON value to pretty print
    match serde_json::from_str::<serde_json::Value>(&payload_str) {
        Ok(json) => Ok(serde_json::to_string_pretty(&json).unwrap_or(payload_str)),
        Err(e) => Err(format!("Failed to parse ID token claims as JSON: {}", e)),
    }
}

// Helper function to build error response
fn build_error_response(error_msg: &str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(axum::http::StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/html")
        .body(axum::body::Body::from(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Auth0 Error</title>
                <style>
                    body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; background-color: #f5f5f5; }}
                    .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                    .error {{ color: #f44336; font-size: 18px; }}
                    .error-details {{ margin: 20px 0; padding: 15px; background: #ffebee; border-radius: 4px; text-align: left; word-wrap: break-word; }}
                    .back-link {{ display: inline-block; margin-top: 20px; padding: 10px 20px; background: #2196F3; color: white; text-decoration: none; border-radius: 4px; }}
                    .back-link:hover {{ background: #1976D2; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="error">
                        <h1>✗ Auth0 Authentication Failed!</h1>
                    </div>
                    <div class="error-details">
                        <strong>Error Details:</strong><br>
                        {}
                    </div>
                    <a href="/auth/login" class="back-link">Try Again</a>
                </div>
            </body>
            </html>
            "#,
            error_msg
        )))
        .unwrap()
        .into_response()
}

// Helper function to build success response
fn build_success_response(
    code: &str,
    state: &str,
    access_token: &str,
    refresh_token: &str,
    id_token: &str,
    claims_json: &str,
) -> axum::response::Response {
    axum::response::Response::builder()
        .header("Content-Type", "text/html")
        .body(axum::body::Body::from(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Auth0 Success</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }}
                    .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                    .success {{ color: #4CAF50; font-size: 24px; margin-bottom: 20px; }}
                    .section {{ margin: 20px 0; padding: 15px; background: #f9f9f9; border-radius: 4px; }}
                    .section h3 {{ margin-top: 0; color: #333; }}
                    .token {{ word-break: break-all; font-family: monospace; font-size: 12px; background: #fff; padding: 10px; border: 1px solid #ddd; border-radius: 4px; max-height: 150px; overflow-y: auto; }}
                    .claims {{ white-space: pre-wrap; font-family: monospace; font-size: 12px; background: #fff; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }}
                    .label {{ font-weight: bold; color: #555; margin-bottom: 5px; }}
                    .back-link {{ display: inline-block; margin-top: 20px; padding: 10px 20px; background: #2196F3; color: white; text-decoration: none; border-radius: 4px; }}
                    .back-link:hover {{ background: #1976D2; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="success">
                        <h1>Auth0 Authentication Successful!</h1>
                    </div>

                    <div class="section">
                        <h3>Callback Information</h3>
                        <div class="label">Authorization Code:</div>
                        <div class="token">{}</div>
                        <div class="label" style="margin-top: 10px;">State:</div>
                        <div class="token">{}</div>
                    </div>

                    <div class="section">
                        <h3>Access Token</h3>
                        <div class="token">{}</div>
                    </div>

                    <div class="section">
                        <h3>Refresh Token</h3>
                        <div class="token">{}</div>
                    </div>

                    <div class="section">
                        <h3>ID Token</h3>
                        <div class="token">{}</div>
                    </div>

                    <div class="section">
                        <h3>ID Token Claims</h3>
                        <div class="claims">{}</div>
                    </div>

                    <a href="/auth/auth0" class="back-link">Return to Home</a>
                </div>
            </body>
            </html>
            "#,
            code, state, access_token, refresh_token, id_token, claims_json
        )))
        .unwrap()
        .into_response()
}

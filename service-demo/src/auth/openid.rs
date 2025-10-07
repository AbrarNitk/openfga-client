use axum::extract::{Query, State};
use axum::response::IntoResponse;
use openidconnect::{
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use reqwest::Client as HttpClient;

use std::collections::HashMap;
use std::sync::Mutex;

use crate::context::Ctx;

// Simple in-memory store for state (in production, use a proper session store)
lazy_static::lazy_static! {
    static ref STATE_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginWithParams {
    pub tp: String,
}

pub async fn login_with(
    State(ctx): State<Ctx>,
    Query(params): Query<LoginWithParams>,
) -> axum::response::Response {
    let dex_config = ctx
        .dex
        .iter()
        .find(|d| d.client_id == "example-app")
        .expect("Dex config not found");

    // Parse the issuer URL
    let issuer_url = IssuerUrl::new(dex_config.issuer_url.clone()).expect("Invalid issuer URL");

    // Create HTTP client using reqwest
    let http_client = HttpClient::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to create HTTP client");

    // Fetch provider metadata using reqwest async client
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .expect("Failed to discover provider metadata");

    // Create OpenID Connect client using the tp parameter as connector_id (not as client_id)
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(dex_config.client_id.clone()),
        Some(ClientSecret::new(dex_config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(dex_config.redirect_url.clone()).expect("Invalid redirect URL"),
    );

    // Generate CSRF token and nonce for state parameter
    let csrf_token = CsrfToken::new_random();
    let nonce = Nonce::new_random();

    // Store state with connector_id for verification
    {
        let mut store = STATE_STORE.lock().unwrap();
        store.insert(csrf_token.secret().clone(), params.tp.clone());
    }

    // Create authorization URL with scopes
    let scopes: Vec<Scope> = dex_config
        .scopes
        .clone()
        .into_iter()
        .map(|s| Scope::new(s))
        .collect();

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            || csrf_token,
            || nonce,
        )
        .add_scopes(scopes)
        .add_extra_param("connector_id", &params.tp) // Add connector_id for DexIdP
        .url();

    println!("OpenID Connect auth_url: {:?}", auth_url);

    // Redirect to DexIdP OpenID Connect authorization endpoint
    axum::response::Response::builder()
        .header("Location", auth_url.to_string())
        .status(axum::http::StatusCode::FOUND)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenIDCallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn handle_openid_callback(
    State(ctx): State<Ctx>,
    Query(params): Query<OpenIDCallbackParams>,
) -> axum::response::Response {
    use openidconnect::{AuthorizationCode, OAuth2TokenResponse, TokenResponse};

    println!("OpenID Connect callback params: {:?}", params);

    // Retrieve connector_id from state store
    let connector_id = {
        let store = STATE_STORE.lock().unwrap();
        store.get(&params.state).cloned()
    };

    let connector_id = match connector_id {
        Some(id) => id,
        None => {
            println!("No connector_id found for state: {}", params.state);
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html")
                .body(axum::body::Body::from("Invalid state parameter"))
                .unwrap()
                .into_response();
        }
    };

    // Get Dex configuration
    let dex_config = ctx
        .dex
        .iter()
        .find(|d| d.client_id == "example-app")
        .expect("Dex config not found");

    // Parse the issuer URL
    let issuer_url = IssuerUrl::new(dex_config.issuer_url.clone()).expect("Invalid issuer URL");

    // Create HTTP client using reqwest
    let http_client = HttpClient::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to create HTTP client");

    // Fetch provider metadata
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .expect("Failed to discover provider metadata");

    // Create OpenID Connect client
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(dex_config.client_id.clone()),
        Some(ClientSecret::new(dex_config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(dex_config.redirect_url.clone()).expect("Invalid redirect URL"),
    );

    // Exchange authorization code for tokens
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code.clone()))
        .expect("Failed to exchange code")
        .request_async(&http_client)
        .await;

    match token_result {
        Ok(token_response) => {
            // Extract tokens
            let access_token = token_response.access_token().secret();
            let refresh_token = token_response
                .refresh_token()
                .map(|t| t.secret().to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let id_token = token_response.id_token();

            // Extract ID token claims if available
            let (id_token_str, claims_json) = if let Some(id_token) = id_token {
                let id_token_str = id_token.to_string();

                // Get ID token verifier and nonce from client
                // Note: In a real application, you should store and retrieve the nonce from the session
                let id_token_verifier = client.id_token_verifier();

                // For display purposes, we'll create a new nonce
                // In production, you should use the original nonce from the authorization request
                let nonce = Nonce::new_random();

                // Try to verify and extract claims
                match id_token.claims(&id_token_verifier, &nonce) {
                    Ok(claims) => {
                        let claims_json = serde_json::json!({
                            "sub": claims.subject().as_str(),
                            "email": claims.email().map(|e| e.as_str()),
                            "email_verified": claims.email_verified(),
                            "name": claims.name().and_then(|n| n.get(None)).map(|n| n.as_str()),
                            "preferred_username": claims.preferred_username().map(|u| u.as_str()),
                            "issuer": claims.issuer().as_str(),
                            "audiences": claims.audiences().iter().map(|a| a.as_str()).collect::<Vec<_>>(),
                            "expiration": claims.expiration().timestamp(),
                        });

                        (
                            id_token_str,
                            serde_json::to_string_pretty(&claims_json).unwrap_or_default(),
                        )
                    }
                    Err(e) => {
                        println!("Warning: Failed to verify ID token claims: {:?}", e);
                        // Still return the token string even if verification fails
                        (
                            id_token_str,
                            format!("{{\"error\": \"Failed to verify claims: {:?}\"}}", e),
                        )
                    }
                }
            } else {
                ("N/A".to_string(), "{}".to_string())
            };

            // Clean up state from store
            {
                let mut store = STATE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            // Return success response with all token details
            let response = axum::response::Response::builder()
                .header("Content-Type", "text/html")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>OpenID Connect Success</title>
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
                                <h1>OpenID Connect Authentication Successful!</h1>
                            </div>

                            <div class="section">
                                <h3>Callback Information</h3>
                                <div class="label">Authorization Code:</div>
                                <div class="token">{}</div>
                                <div class="label" style="margin-top: 10px;">State:</div>
                                <div class="token">{}</div>
                                <div class="label" style="margin-top: 10px;">Connector ID:</div>
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

                            <a href="/auth" class="back-link">Return to Home</a>
                        </div>
                    </body>
                    </html>
                    "#,
                    params.code,
                    params.state,
                    connector_id,
                    access_token,
                    refresh_token,
                    id_token_str,
                    claims_json
                ))
                .unwrap()
                .into_response();
            response
        }
        Err(e) => {
            println!("Token exchange error: {:?}", e);

            // Clean up state from store even on error
            {
                let mut store = STATE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            let response = axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>OpenID Connect Error</title>
                        <style>
                            body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; background-color: #f5f5f5; }}
                            .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                            .error {{ color: #f44336; font-size: 18px; }}
                            .error-details {{ margin: 20px 0; padding: 15px; background: #ffebee; border-radius: 4px; text-align: left; }}
                            .back-link {{ display: inline-block; margin-top: 20px; padding: 10px 20px; background: #2196F3; color: white; text-decoration: none; border-radius: 4px; }}
                            .back-link:hover {{ background: #1976D2; }}
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <div class="error">
                                <h1>✗ OpenID Connect Authentication Failed!</h1>
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
                    format!("{:?}", e)
                ))
                .unwrap()
                .into_response();
            response
        }
    }
}

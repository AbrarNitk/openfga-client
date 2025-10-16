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

// Structure to store state data including nonce
#[derive(Debug, Clone)]
struct StateData {
    connector_id: String,
    nonce: String,
}

// Simple in-memory store for state (in production, use a proper session store)
lazy_static::lazy_static! {
    static ref STATE_STORE: Mutex<HashMap<String, StateData>> = Mutex::new(HashMap::new());
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

    // Store state with connector_id and nonce for verification
    {
        let mut store = STATE_STORE.lock().unwrap();
        store.insert(
            csrf_token.secret().clone(),
            StateData {
                connector_id: params.tp.clone(),
                nonce: nonce.secret().clone(),
            },
        );
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
        .add_extra_param("organization", "conversight")
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
    pub code: Option<String>,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub async fn handle_openid_callback(
    State(ctx): State<Ctx>,
    Query(params): Query<OpenIDCallbackParams>,
) -> axum::response::Response {
    use openidconnect::{AuthorizationCode, OAuth2TokenResponse, TokenResponse};

    println!("OpenID Connect callback params: {:?}", params);

    // Check if Dex/IdP returned an error
    if let Some(error) = &params.error {
        let error_description = params
            .error_description
            .as_deref()
            .unwrap_or("No additional error description provided");

        let error_msg = format!(
            "OpenID Connect Error: {}\nDescription: {}",
            error, error_description
        );

        println!("IdP returned error: {}", error_msg);

        // Clean up state from store if present
        {
            let mut store = STATE_STORE.lock().unwrap();
            store.remove(&params.state);
        }

        return build_openid_error_response(error, error_description);
    }

    // Extract authorization code (required if no error)
    let code = match &params.code {
        Some(c) => c,
        None => {
            println!("No authorization code provided in callback");
            return build_generic_error_response(
                "No authorization code received from identity provider",
            );
        }
    };

    // Retrieve state data (connector_id and nonce) from state store
    let state_data = {
        let store = STATE_STORE.lock().unwrap();
        store.get(&params.state).cloned()
    };

    let state_data = match state_data {
        Some(data) => data,
        None => {
            println!("No state data found for state: {}", params.state);
            return build_generic_error_response(
                "Invalid state parameter. The session may have expired or the request is invalid.",
            );
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
        .exchange_code(AuthorizationCode::new(code.clone()))
        .expect("Failed to exchange code")
        .request_async(&http_client)
        .await;

    match token_result {
        Ok(token_response) => {
            let tr = serde_json::to_value(token_response.clone());
            println!("token response: {:?}", tr);

            let access_token = token_response.access_token().secret();
            let refresh_token = token_response
                .refresh_token()
                .map(|t| t.secret().to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let id_token = token_response.id_token();

            // Extract ID token claims if available
            let (id_token_str, claims_json) = if let Some(id_token) = id_token {
                let id_token_str = id_token.to_string();

                // Get ID token verifier from client
                let id_token_verifier = client.id_token_verifier();

                // Retrieve the stored nonce from the state data
                let nonce = Nonce::new(state_data.nonce.clone());

                // Try to verify and extract claims
                match id_token.claims(&id_token_verifier, &nonce) {
                    Ok(claims) => {
                        let claims_response = serde_json::to_value(claims.clone());

                        println!("token-claims: {:?}", claims_response);

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
                .header("Content-Type", "text/html; charset=utf-8")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
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
                    code,
                    params.state,
                    state_data.connector_id,
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
                .header("Content-Type", "text/html; charset=utf-8")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
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

// Helper function to build OpenID Connect error response with error code and description
fn build_openid_error_response(error: &str, error_description: &str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(axum::http::StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(axum::body::Body::from(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>OpenID Connect Authentication Error</title>
                <style>
                    body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; background-color: #f5f5f5; }}
                    .container {{ max-width: 700px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                    .error {{ color: #f44336; font-size: 18px; }}
                    .error-icon {{ font-size: 48px; color: #f44336; margin-bottom: 20px; }}
                    .error-code {{ margin: 20px 0; padding: 15px; background: #ffebee; border-radius: 4px; border-left: 4px solid #f44336; text-align: left; }}
                    .error-code-title {{ font-weight: bold; color: #c62828; margin-bottom: 10px; font-size: 16px; }}
                    .error-code-value {{ font-family: monospace; color: #d32f2f; margin-bottom: 15px; }}
                    .error-description {{ color: #555; line-height: 1.6; }}
                    .back-link {{ display: inline-block; margin-top: 20px; padding: 10px 20px; background: #2196F3; color: white; text-decoration: none; border-radius: 4px; }}
                    .back-link:hover {{ background: #1976D2; }}
                    .info-box {{ margin: 20px 0; padding: 15px; background: #e3f2fd; border-radius: 4px; border-left: 4px solid #2196F3; text-align: left; }}
                    .info-title {{ font-weight: bold; color: #1565c0; margin-bottom: 8px; }}
                    .info-text {{ color: #424242; font-size: 14px; line-height: 1.5; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="error-icon">⚠️</div>
                    <div class="error">
                        <h1>Authentication Failed</h1>
                    </div>
                    
                    <div class="error-code">
                        <div class="error-code-title">Error Code:</div>
                        <div class="error-code-value">{}</div>
                        <div class="error-code-title">Error Description:</div>
                        <div class="error-description">{}</div>
                    </div>

                    <div class="info-box">
                        <div class="info-title">Common Causes:</div>
                        <div class="info-text">
                            • <strong>access_denied:</strong> User cancelled the login or doesn't have access<br>
                            • <strong>unauthorized:</strong> Invalid organization or missing permissions<br>
                            • <strong>invalid_request:</strong> Malformed request parameters<br>
                            • <strong>server_error:</strong> Issue with the identity provider
                        </div>
                    </div>

                    <a href="/auth" class="back-link">← Return to Login</a>
                </div>
            </body>
            </html>
            "#,
            error,
            error_description
        )))
        .unwrap()
        .into_response()
}

// Helper function to build generic error response
fn build_generic_error_response(error_msg: &str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(axum::http::StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(axum::body::Body::from(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Authentication Error</title>
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
                        <h1>✗ Authentication Failed!</h1>
                    </div>
                    <div class="error-details">
                        <strong>Error Details:</strong><br>
                        {}
                    </div>
                    <a href="/auth" class="back-link">Try Again</a>
                </div>
            </body>
            </html>
            "#,
            error_msg
        )))
        .unwrap()
        .into_response()
}

use axum::{extract::Query, response::IntoResponse};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as BASE64_ENGINE;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl, basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub preferred_username: Option<String>,
    #[serde(flatten)]
    pub other: Option<Value>,
}

fn parse_jwt_claims(id_token: &str) -> Option<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = parts[1];
    let decoded = BASE64_ENGINE.decode(payload.as_bytes()).ok()?;
    serde_json::from_slice::<IdTokenClaims>(&decoded).ok()
}

// DexIdP OAuth2 Configuration
const DEX_CLIENT_ID: &str = "example-app";
const DEX_CLIENT_SECRET: &str = "ZXhhbXBsZS1hcHAtc2VjcmV0";
const DEX_AUTH_URL: &str = "http://127.0.0.1:5556/dex/auth";
const DEX_TOKEN_URL: &str = "http://127.0.0.1:5556/dex/token";
const DEX_REDIRECT_URL: &str = "http://127.0.0.1:5001/auth/callback";

// OAuth2 scopes for DexIdP
const OAUTH_SCOPES: &[&str] = &["openid", "profile", "email", "offline_access"];

// Simple in-memory store for PKCE verifiers (in production, use a proper session store)
lazy_static::lazy_static! {
    static ref PKCE_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthState {
    pub platform: String,
    pub csrf_token: String,
    pub pkce_verifier: String,
}

pub async fn serve_login_template() -> axum::response::Response {
    let file = std::fs::File::open("service-demo/src/auth/templates/login_with.html").unwrap();
    let contents = std::io::read_to_string(file).unwrap();
    let response = axum::response::Response::builder()
        .header("Content-Type", "text/html")
        .body(contents)
        .unwrap()
        .into_response();
    response
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginWithParams {
    pub tp: String,
}

pub async fn login_with(Query(params): Query<LoginWithParams>) -> axum::response::Response {
    // Create DexIdP OAuth2 client
    let client = BasicClient::new(ClientId::new(DEX_CLIENT_ID.to_string()))
        .set_client_secret(ClientSecret::new(DEX_CLIENT_SECRET.to_string()))
        .set_auth_uri(AuthUrl::new(DEX_AUTH_URL.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(DEX_TOKEN_URL.to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(DEX_REDIRECT_URL.to_string()).unwrap());

    // Generate PKCE challenge for security
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate CSRF token
    let csrf_token = CsrfToken::new_random();

    // Store state for verification (in production, use a secure session store)
    let state = OAuthState {
        platform: params.tp.clone(),
        csrf_token: csrf_token.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
    };

    // Store PKCE verifier for later use in callback
    {
        let mut store = PKCE_STORE.lock().unwrap();
        store.insert(csrf_token.secret().clone(), pkce_verifier.secret().clone());
    }

    // Create authorization URL with scopes and connector_id
    let scopes: Vec<Scope> = OAUTH_SCOPES
        .iter()
        .map(|s| Scope::new(s.to_string()))
        .collect();

    let (auth_url, _) = client
        .authorize_url(|| CsrfToken::new(state.csrf_token.clone()))
        .add_scopes(scopes)
        .set_pkce_challenge(pkce_challenge)
        .add_extra_param("connector_id", &params.tp) // Add connector_id for DexIdP
        .url();

    println!("redirect_uri auth_url: {:?}", auth_url);

    // Redirect to DexIdP OAuth2 authorization endpoint
    let response = axum::response::Response::builder()
        .header("Location", auth_url.to_string())
        .status(axum::http::StatusCode::FOUND)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response();
    response
}

pub async fn make_redirect_uri_to_dex(tp: String) -> String {
    // client-id: example-app
    // client-secret: ZXhhbXBsZS1hcHAtc2VjcmV0
    // redirect-url: http://127.0.0.1:5001/auth/callback

    let connector_id = tp.as_str();

    let path = "/dex";
    let query = format!("connector_id={}", connector_id);

    // build the uri
    let redirect_url = axum::http::Uri::builder()
        .scheme("http")
        .authority("127.0.0.1:5556")
        .path_and_query(format!("{}{}", path, query))
        .build()
        .unwrap();
    redirect_url.to_string()
}

#[derive(Debug, serde::Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn handle_oauth_callback(
    Query(params): Query<OAuthCallbackParams>,
) -> axum::response::Response {
    println!("OAuth callback params: {:?}", params);

    // Retrieve PKCE verifier from store
    let pkce_verifier = {
        let store = PKCE_STORE.lock().unwrap();
        store.get(&params.state).cloned()
    };

    let pkce_verifier = match pkce_verifier {
        Some(verifier) => verifier,
        None => {
            println!("No PKCE verifier found for state: {}", params.state);
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html")
                .body(axum::body::Body::from("Invalid state parameter"))
                .unwrap()
                .into_response();
        }
    };

    // We don't need the OAuth2 client for token exchange since we're doing it manually
    
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Exchange authorization code for access token and id_token with PKCE verifier
    // Use direct HTTP call to get both access_token and id_token in one request
    let form = [
        ("grant_type", "authorization_code"),
        ("code", params.code.as_str()),
        ("redirect_uri", DEX_REDIRECT_URL),
        ("client_id", DEX_CLIENT_ID),
        ("client_secret", DEX_CLIENT_SECRET),
        ("code_verifier", &pkce_verifier),
    ];
    
    let token_result = http_client
        .post(DEX_TOKEN_URL)
        .form(&form)
        .send()
        .await;

    match token_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json_val) => {
                    println!("Token response: {:?}", json_val);

                    // Extract tokens from response
                    let access_token = json_val.get("access_token")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    
                    let id_token = json_val.get("id_token")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    let claims = id_token.as_deref().and_then(parse_jwt_claims);

            // Clean up PKCE verifier from store
            {
                let mut store = PKCE_STORE.lock().unwrap();
                store.remove(&params.state);
            }

            // In production, you would:
            // 1. Verify the state parameter matches what you stored
            // 2. Store the access token securely
            // 3. Use the access token to fetch user information from DexIdP
            // 4. Create a session for the user

            let response = axum::response::Response::builder()
                .header("Content-Type", "text/html")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>OAuth Success</title>
                        <style>
                            body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; }}
                            .success {{ color: green; font-size: 18px; }}
                        </style>
                    </head>
                    <body>
                        <div class="success">
                            <h1>OAuth Authentication Successful!</h1>
                            <p>Authorization Code: {}</p>
                            <p>State: {}</p>
                            <p>Access Token: {}</p>
                            <p>ID Token: {}</p>
                            <pre style="text-align:left;display:inline-block;">{}</pre>
                            <p><a href="/">Return to Home</a></p>
                        </div>
                    </body>
                    </html>
                    "#,
                    params.code,
                    params.state,
                            access_token,
                    id_token.as_deref().unwrap_or("-"),
                    claims.as_ref().map(|c| serde_json::to_string_pretty(c).unwrap_or_default()).unwrap_or_else(||"{}".to_string())
                ))
                .unwrap()
                .into_response();
            response
                }
                Err(e) => {
                    println!("Failed to parse token response as JSON: {:?}", e);
                    let response = axum::response::Response::builder()
                        .status(axum::http::StatusCode::BAD_REQUEST)
                        .header("Content-Type", "text/html")
                        .body(format!(
                            r#"
                            <!DOCTYPE html>
                            <html>
                            <head>
                                <title>OAuth Error</title>
                                <style>
                                    body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; }}
                                    .error {{ color: red; font-size: 18px; }}
                                </style>
                            </head>
                            <body>
                                <div class="error">
                                    <h1>OAuth Token Parsing Failed!</h1>
                                    <p>Error: {}</p>
                                    <p><a href="/auth/login">Try Again</a></p>
                                </div>
                            </body>
                            </html>
                            "#,
                            e
                        ))
                        .unwrap()
                        .into_response();
                    response
                }
            }
        }
        Err(e) => {
            println!("Token exchange error: {:?}", e);
            let response = axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html")
                .body(format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>OAuth Error</title>
                        <style>
                            body {{ font-family: Arial, sans-serif; text-align: center; margin-top: 50px; }}
                            .error {{ color: red; font-size: 18px; }}
                        </style>
                    </head>
                    <body>
                        <div class="error">
                            <h1>OAuth Authentication Failed!</h1>
                            <p>Error: {}</p>
                            <p><a href="/auth/login">Try Again</a></p>
                        </div>
                    </body>
                    </html>
                    "#,
                    e
                ))
                .unwrap()
                .into_response();
            response
        }
    }
}

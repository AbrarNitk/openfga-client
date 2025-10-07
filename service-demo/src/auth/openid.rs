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
    Query(_params): Query<OpenIDCallbackParams>,
) -> axum::response::Response {
    todo!()
}

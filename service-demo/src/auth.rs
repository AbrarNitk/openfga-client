use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::context::Ctx;

/// User information extracted from authentication
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
}

/// Authentication middleware that extracts user ID from headers
pub async fn auth_middleware(
    State(_ctx): State<Arc<Ctx>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<Value>)> {
    // Extract user ID from the "X-User-Id" header
    let user_id = match headers.get("x-user-id") {
        Some(header_value) => match header_value.to_str() {
            Ok(user_id) => {
                if user_id.trim().is_empty() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Invalid user ID",
                            "message": "X-User-Id header cannot be empty"
                        })),
                    ));
                }
                user_id.to_string()
            }
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Invalid header format",
                        "message": "X-User-Id header must be valid UTF-8"
                    })),
                ));
            }
        },
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Missing authentication",
                    "message": "X-User-Id header is required"
                })),
            ));
        }
    };

    tracing::info!("Authenticated user: {}", user_id);

    // Create AuthUser and insert it into request extensions
    let auth_user = AuthUser { user_id };
    request.extensions_mut().insert(auth_user);

    // Continue to the next handler
    Ok(next.run(request).await)
}

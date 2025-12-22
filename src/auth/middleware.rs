use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AppState;
use crate::error::ApiError;

/// Axum middleware for HMAC authentication
pub async fn hmac_auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let validator = match &state.hmac_validator {
        Some(v) => v,
        None => return next.run(request).await,
    };

    let query_string = request.uri().query().unwrap_or("");

    match validator.validate(query_string) {
        Ok(_) => {
            tracing::debug!("HMAC validation successful");
            next.run(request).await
        }
        Err(e) => {
            tracing::warn!("HMAC validation failed: {}", e);
            let api_error: ApiError = e.into();
            api_error.into_response()
        }
    }
}

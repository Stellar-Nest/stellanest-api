use axum::{
    extract::Request,
    extract::State,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::models::Claims;

/// JWT authentication middleware.
/// Extracts the Bearer token, validates it, and injects claims into request extensions.
/// Intended to be used with `axum::middleware::from_fn_with_state` so that
/// the JWT secret is read from `AppState` rather than from request extensions.
pub async fn jwt_auth(
    State(state): State<crate::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = &state.jwt_secret;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Inject claims into request extensions
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

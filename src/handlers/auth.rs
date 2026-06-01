use axum::{extract::State, http::StatusCode, Json};

use crate::AppState;
use crate::models::{AuthChallengeRequest, AuthTokenRequest, Claims};

/// POST /api/v1/auth/challenge
pub async fn challenge(
    State(state): State<AppState>,
    Json(req): Json<AuthChallengeRequest>,
) -> Json<serde_json::Value> {
    // TODO:
    // 1. Build SEP-10 challenge transaction using stellar-sdk
    // 2. Include account, home domain, web auth domain
    // 3. Sign with platform key
    // 4. Return base64-encoded XDR

    let passphrase = if state.stellar_network == "mainnet" {
        "Public Global Stellar Network ; September 2015"
    } else {
        "Test SDF Network ; September 2015"
    };

    Json(serde_json::json!({
        "transaction": "<base64-encoded-xdr>",
        "network_passphrase": passphrase,
    }))
}

/// POST /api/v1/auth/token
pub async fn token(
    State(state): State<AppState>,
    Json(req): Json<AuthTokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO:
    // 1. Decode and verify the signed XDR
    // 2. Verify signatures match the account
    // 3. Verify challenge was issued by us (home domain)
    // 4. Check KYC status
    // 5. Issue JWT

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: "GABC...".into(),
        kyc_status: "none".into(),
        trading_tier: 1,
        exp: now + 86400,
        iat: now,
    };

    let header = jsonwebtoken::Header::default();
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "token": token,
        "account": claims.sub,
    })))
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Extract from JWT middleware
    Json(serde_json::json!({
        "account": "GABC...",
        "kyc_status": "none",
    }))
}

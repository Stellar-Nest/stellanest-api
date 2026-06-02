use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::Row;

use crate::AppState;
use crate::models::{AuthChallengeRequest, AuthTokenRequest, Claims};

/// POST /api/v1/auth/challenge
///
/// Builds a SEP-10 challenge transaction for the requesting account.
/// The client must sign this transaction and submit it to `/auth/token`.
pub async fn challenge(
    State(state): State<AppState>,
    Json(req): Json<AuthChallengeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let passphrase = if state.stellar_network == "mainnet" {
        "Public Global Stellar Network ; September 2015"
    } else {
        "Test SDF Network ; September 2015"
    };

    let home_domain =
        std::env::var("STELLAR_HOME_DOMAIN").unwrap_or_else(|_| "stellanest.io".into());

    let transaction = crate::services::auth::build_challenge(
        &req.account,
        &home_domain,
        passphrase,
    )
    .map_err(|e| {
        tracing::error!("failed to build challenge: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "transaction": transaction,
        "network_passphrase": passphrase,
    })))
}

/// POST /api/v1/auth/token
///
/// Verifies a signed SEP-10 challenge transaction, looks up the user's
/// KYC status and trading tier in the database, and issues a JWT.
pub async fn token(
    State(state): State<AppState>,
    Json(req): Json<AuthTokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let passphrase = if state.stellar_network == "mainnet" {
        "Public Global Stellar Network ; September 2015"
    } else {
        "Test SDF Network ; September 2015"
    };

    // 1. Verify the signed challenge transaction
    let account =
        crate::services::auth::verify_challenge(&req.transaction, passphrase).map_err(|e| {
            tracing::warn!("SEP-10 verification failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // 2. Look up (or create) the user in the database
    let user_row = sqlx::query(
        "SELECT id, stellar_account, kyc_status, trading_tier
         FROM users
         WHERE stellar_account = $1",
    )
    .bind(&account)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("database query failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (kyc_status, trading_tier) = if let Some(row) = user_row {
        let kyc: String = row
            .try_get("kyc_status")
            .unwrap_or_else(|_| "none".to_string());
        let tier: i32 = row.try_get("trading_tier").unwrap_or(1);
        (kyc, tier)
    } else {
        // Auto-register new user
        sqlx::query(
            "INSERT INTO users (id, stellar_account, kyc_status, trading_tier)
             VALUES (gen_random_uuid(), $1, 'none', 1)",
        )
        .bind(&account)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("failed to create user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tracing::info!("auto-registered new user: {}", account);
        ("none".to_string(), 1)
    };

    // 3. Issue JWT with real account data
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: account.clone(),
        kyc_status,
        trading_tier,
        exp: now + 86400,
        iat: now,
    };

    let header = jsonwebtoken::Header::default();
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("JWT encoding failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "token": token,
        "account": claims.sub,
        "kyc_status": claims.kyc_status,
        "trading_tier": claims.trading_tier,
    })))
}

/// GET /api/v1/auth/me
///
/// Returns the authenticated user's profile from JWT claims.
pub async fn me(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    // In a real implementation the JWT middleware would inject Claims
    // into request extensions; this endpoint reads from there.
    // For now, return a placeholder indicating the middleware should be used.
    Json(serde_json::json!({
        "message": "JWT claims would be extracted by middleware"
    }))
}

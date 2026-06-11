use axum::{extract::State, Extension, Json};

use crate::AppState;
use crate::models::{Claims, OracleSubmitRequest};

/// POST /api/v1/oracle/submit
pub async fn submit(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<OracleSubmitRequest>,
) -> Json<serde_json::Value> {
    let wallet = &claims.sub;

    let result = sqlx::query(
        "INSERT INTO oracle_submissions (oracle_wallet, city_code, price, confidence, source, timestamp)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(wallet)
    .bind(&req.city)
    .bind(req.price)
    .bind(req.confidence)
    .bind(&req.source)
    .bind(&req.timestamp)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({"accepted": true})),
        Err(e) => Json(serde_json::json!({"accepted": false, "error": e.to_string()})),
    }
}

/// GET /api/v1/oracle/prices
pub async fn prices(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Query latest aggregated prices from city_indices
    Json(serde_json::json!({
        "NYC": {"price": 485.23, "confidence": 9200, "oracles": 4},
        "LON": {"price": 312.67, "confidence": 8800, "oracles": 3},
        "LAG": {"price": 45.89, "confidence": 7500, "oracles": 3},
    }))
}

/// GET /api/v1/oracle/status
pub async fn status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "active_oracles": 5,
        "cities_covered": 20,
        "avg_confidence": 9100,
    }))
}

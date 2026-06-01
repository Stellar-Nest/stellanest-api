use axum::{extract::State, Json};

use crate::AppState;

/// GET /api/v1/analytics/volume
pub async fn volume(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Aggregate from trades table
    Json(serde_json::json!({
        "period": "24h",
        "cities": [],
    }))
}

/// GET /api/v1/analytics/open-interest
pub async fn open_interest(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Aggregate from positions table
    Json(serde_json::json!({"cities": []}))
}

/// GET /api/v1/analytics/traders
pub async fn top_traders(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Rank by total P&L
    Json(serde_json::json!({"traders": []}))
}

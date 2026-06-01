use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::AppState;
use crate::models::{OpenPositionRequest, OpenPositionResponse, ClosePositionResponse, Position};

/// POST /api/v1/positions/open
pub async fn open(
    State(state): State<AppState>,
    Json(req): Json<OpenPositionRequest>,
) -> Result<Json<OpenPositionResponse>, StatusCode> {
    // TODO:
    // 1. Extract wallet from JWT (middleware sets it in extensions)
    // 2. Validate KYC tier vs leverage
    // 3. Query current index price from oracle
    // 4. Calculate size and liquidation price
    // 5. Build + submit Soroban tx (vault deposit + position open)
    // 6. Insert into positions table

    let entry_price = 0.0; // TODO: query oracle
    let size = req.collateral * req.leverage as f64;
    let liq_price = if req.direction == "long" {
        entry_price * (1.0 - 1.0 / req.leverage as f64 * 0.8)
    } else {
        entry_price * (1.0 + 1.0 / req.leverage as f64 * 0.8)
    };

    Ok(Json(OpenPositionResponse {
        position_id: Uuid::new_v4().to_string(),
        city: req.city,
        direction: req.direction,
        entry_price,
        collateral: req.collateral,
        size,
        leverage: req.leverage,
        liquidation_price: liq_price,
        health_factor: 1.85,
        tx_hash: String::new(),
        opened_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// POST /api/v1/positions/:id/close
pub async fn close(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClosePositionResponse>, StatusCode> {
    // TODO:
    // 1. Verify ownership
    // 2. Query current price
    // 3. Calculate P&L
    // 4. Submit Soroban close tx
    // 5. Update DB

    Ok(Json(ClosePositionResponse {
        position_id: id,
        exit_price: 0.0,
        entry_price: 0.0,
        pnl: 0.0,
        pnl_percent: 0.0,
        collateral_returned: 0.0,
        duration: String::new(),
        tx_hash: String::new(),
    }))
}

/// GET /api/v1/positions/my
pub async fn my(
    State(state): State<AppState>,
) -> Json<Vec<Position>> {
    // TODO: Extract wallet from JWT, query by user_wallet
    Json(vec![])
}

/// GET /api/v1/positions/:id
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Position>, StatusCode> {
    let pos = sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE id = $1"
    )
    .bind(Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    pos.map(Json).ok_or(StatusCode::NOT_FOUND)
}

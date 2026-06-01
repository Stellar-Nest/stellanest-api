use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::AppState;
use crate::models::{CreateOrderRequest, Order, OrderBookResponse, Trade};

/// POST /api/v1/orders
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO:
    // 1. Validate order params
    // 2. Build Stellar DEX manage offer operation
    // 3. Submit to network
    // 4. Store in orders table

    Ok(Json(serde_json::json!({
        "order_id": Uuid::new_v4().to_string(),
        "city": req.city,
        "side": req.side,
        "type": req.order_type,
        "status": "open",
    })))
}

/// DELETE /api/v1/orders/:id
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    // TODO: Build Stellar DEX manage offer (amount=0) to cancel
    Json(serde_json::json!({
        "order_id": id,
        "status": "cancelled",
    }))
}

/// GET /api/v1/orders/my
pub async fn my(
    State(state): State<AppState>,
) -> Json<Vec<Order>> {
    // TODO: Extract wallet from JWT
    Json(vec![])
}

/// GET /api/v1/orders/book/:city
pub async fn book(
    State(state): State<AppState>,
    Path(city): Path<String>,
) -> Json<OrderBookResponse> {
    // TODO: Query Stellar DEX order book for city index asset
    Json(OrderBookResponse {
        city,
        bids: vec![],
        asks: vec![],
        spread: 0.0,
        mid_price: 0.0,
        volume_24h: 0.0,
        trades_24h: 0,
    })
}

/// GET /api/v1/orders/recent/:city
pub async fn recent(
    State(state): State<AppState>,
    Path(city): Path<String>,
) -> Json<Vec<Trade>> {
    let trades = sqlx::query_as::<_, Trade>(
        "SELECT * FROM trades WHERE city_code = $1 ORDER BY time DESC LIMIT 50"
    )
    .bind(&city)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(trades)
}

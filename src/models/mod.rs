use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── City Index ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CityIndex {
    pub city_code: String,
    pub name: String,
    pub country: String,
    pub flag_emoji: Option<String>,
    pub description: Option<String>,
    pub current_value: f64,
    pub change_24h: Option<f64>,
    pub change_7d: Option<f64>,
    pub change_30d: Option<f64>,
    pub change_1y: Option<f64>,
    pub volatility_30d: Option<f64>,
    pub volume_24h: Option<f64>,
    pub open_interest: Option<f64>,
    pub data_source_count: Option<i32>,
    pub stellar_asset_code: Option<String>,
    pub stellar_asset_issuer: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ── Index History ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct IndexHistory {
    pub time: DateTime<Utc>,
    pub city_code: String,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<f64>,
    pub source_count: Option<i32>,
}

// ── User ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_account: String,
    pub kyc_status: Option<String>,
    pub kyc_tier: Option<i32>,
    pub trading_tier: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

// ── Position ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Position {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_wallet: String,
    pub city_code: Option<String>,
    pub direction: String,
    pub leverage: i32,
    pub entry_price: f64,
    pub collateral: f64,
    pub size: f64,
    pub liquidation_price: Option<f64>,
    pub current_price: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    pub realized_pnl: Option<f64>,
    pub funding_paid: Option<f64>,
    pub health_factor: Option<f64>,
    pub status: Option<String>,
    pub close_price: Option<f64>,
    pub close_reason: Option<String>,
    pub soroban_position_id: Option<String>,
    pub open_tx_hash: Option<String>,
    pub close_tx_hash: Option<String>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

// ── Order ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_wallet: String,
    pub city_code: Option<String>,
    pub side: String,
    #[sqlx(rename = "type")]
    pub order_type: String,
    pub price: Option<f64>,
    pub size: f64,
    pub filled: Option<f64>,
    pub status: Option<String>,
    pub stellar_tx_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub filled_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

// ── Trade ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    pub time: DateTime<Utc>,
    pub city_code: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    pub buyer_wallet: Option<String>,
    pub seller_wallet: Option<String>,
    pub tx_hash: Option<String>,
}

// ── Oracle Submission ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OracleSubmission {
    pub id: i64,
    pub oracle_wallet: String,
    pub city_code: String,
    pub price: f64,
    pub confidence: i32,
    pub source: String,
    pub accepted: Option<bool>,
    pub timestamp: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
}

// ── Liquidation ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Liquidation {
    pub id: Uuid,
    pub position_id: Option<Uuid>,
    pub user_wallet: String,
    pub city_code: Option<String>,
    pub direction: Option<String>,
    pub entry_price: Option<f64>,
    pub liquidation_price: Option<f64>,
    pub collateral: Option<f64>,
    pub collateral_seized: Option<f64>,
    pub penalty: Option<f64>,
    pub tx_hash: Option<String>,
    pub liquidated_at: Option<DateTime<Utc>>,
}

// ── Request / Response ──

#[derive(Debug, Deserialize)]
pub struct OpenPositionRequest {
    pub city: String,
    pub direction: String,
    pub collateral: f64,
    pub leverage: i32,
    pub order_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenPositionResponse {
    pub position_id: String,
    pub city: String,
    pub direction: String,
    pub entry_price: f64,
    pub collateral: f64,
    pub size: f64,
    pub leverage: i32,
    pub liquidation_price: f64,
    pub health_factor: f64,
    pub tx_hash: String,
    pub opened_at: String,
}

#[derive(Debug, Serialize)]
pub struct ClosePositionResponse {
    pub position_id: String,
    pub exit_price: f64,
    pub entry_price: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub collateral_returned: f64,
    pub duration: String,
    pub tx_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub city: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub price: Option<f64>,
    pub size: f64,
}

#[derive(Debug, Serialize)]
pub struct OrderBookResponse {
    pub city: String,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
    pub spread: f64,
    pub mid_price: f64,
    pub volume_24h: f64,
    pub trades_24h: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderLevel {
    pub price: f64,
    pub size: f64,
    pub orders: i32,
}

#[derive(Debug, Deserialize)]
pub struct OracleSubmitRequest {
    pub city: String,
    pub price: f64,
    pub confidence: i32,
    pub source: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthChallengeRequest {
    pub account: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthTokenRequest {
    pub transaction: String,
}

#[derive(Debug, Serialize)]
pub struct Claims {
    pub sub: String,
    pub kyc_status: String,
    pub trading_tier: i32,
    pub exp: usize,
    pub iat: usize,
}

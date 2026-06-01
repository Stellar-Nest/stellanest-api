use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use std::collections::HashMap;

use crate::AppState;
use crate::models::CityIndex;

/// GET /api/v1/indices
pub async fn list(State(state): State<AppState>) -> Json<Vec<CityIndex>> {
    let rows = sqlx::query_as::<_, CityIndex>(
        "SELECT * FROM city_indices WHERE status = 'active' ORDER BY volume_24h DESC"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(rows)
}

/// GET /api/v1/indices/:city
pub async fn get(
    State(state): State<AppState>,
    Path(city): Path<String>,
) -> Json<serde_json::Value> {
    let row = sqlx::query_as::<_, CityIndex>(
        "SELECT * FROM city_indices WHERE city_code = $1"
    )
    .bind(&city)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match row {
        Some(idx) => Json(serde_json::to_value(&idx).unwrap_or_default()),
        None => Json(serde_json::json!({"error": "city not found"})),
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub interval: Option<String>,
}

/// GET /api/v1/indices/:city/history
pub async fn history(
    State(state): State<AppState>,
    Path(city): Path<String>,
    Query(params): Query<HistoryParams>,
) -> Json<Vec<serde_json::Value>> {
    let rows = sqlx::query_as::<_, crate::models::IndexHistory>(
        "SELECT * FROM index_history WHERE city_code = $1 ORDER BY time DESC LIMIT 1000"
    )
    .bind(&city)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({
            "timestamp": r.time,
            "open": r.open,
            "high": r.high,
            "low": r.low,
            "close": r.close,
            "volume": r.volume,
        })
    }).collect();

    Json(result)
}

#[derive(Debug, Deserialize)]
pub struct CompareParams {
    pub cities: Option<String>,
    pub period: Option<String>,
}

/// GET /api/v1/indices/compare
pub async fn compare(
    State(state): State<AppState>,
    Query(params): Query<CompareParams>,
) -> Json<serde_json::Value> {
    let cities_str = params.cities.unwrap_or_default();
    let cities: Vec<&str> = cities_str.split(',').collect();

    let mut results = serde_json::Map::new();
    for city in cities {
        if let Some(idx) = sqlx::query_as::<_, CityIndex>(
            "SELECT * FROM city_indices WHERE city_code = $1"
        )
        .bind(city.trim())
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        {
            results.insert(city.trim().to_string(), serde_json::json!({
                "current_value": idx.current_value,
                "change_1y": idx.change_1y,
                "volatility_30d": idx.volatility_30d,
            }));
        }
    }

    Json(serde_json::json!({
        "period": params.period.unwrap_or_else(|| "1y".into()),
        "cities": results,
    }))
}

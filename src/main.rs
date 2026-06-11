use axum::{middleware, routing::{get, post, delete}, Router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handlers;
mod middleware;
mod models;
mod services;
mod ws;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::aio::Connection,
    pub jwt_secret: String,
    pub stellar_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "stellanest_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://stellanest:stellanest@localhost:5432/stellanest".into());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".into());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-me-in-production".into());
    let stellar_network = std::env::var("STELLAR_NETWORK")
        .unwrap_or_else(|_| "testnet".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    // Database
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;
    tracing::info!("connected to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await
        .map_err(|e| anyhow::anyhow!("migration failed: {}", e))?;
    tracing::info!("migrations applied");

    // Redis
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = redis_client.get_async_connection().await?;
    tracing::info!("connected to Redis");

    let state = AppState {
        db,
        redis: redis_conn,
        jwt_secret,
        stellar_network,
    };

    // WebSocket hub
    let ws_hub = ws::Hub::new();
    let ws_handle = ws_hub.start();

    // Public routes
    let public_routes = Router::new()
        // Auth
        .route("/api/v1/auth/challenge", post(handlers::auth::challenge))
        .route("/api/v1/auth/token", post(handlers::auth::token))
        .route("/api/v1/auth/me", get(handlers::auth::me))
        // Indices (public)
        .route("/api/v1/indices", get(handlers::index::list))
        .route("/api/v1/indices/:city", get(handlers::index::get))
        .route("/api/v1/indices/:city/history", get(handlers::index::history))
        .route("/api/v1/indices/compare", get(handlers::index::compare))
        // Analytics (public)
        .route("/api/v1/analytics/volume", get(handlers::analytics::volume))
        .route("/api/v1/analytics/open-interest", get(handlers::analytics::open_interest))
        .route("/api/v1/analytics/traders", get(handlers::analytics::top_traders))
        // WebSocket
        .route("/ws", get(ws::ws_handler));

    // Authenticated routes (JWT required)
    let authenticated_routes = Router::new()
        // Positions
        .route("/api/v1/positions/open", post(handlers::position::open))
        .route("/api/v1/positions/:id/close", post(handlers::position::close))
        .route("/api/v1/positions/my", get(handlers::position::my))
        .route("/api/v1/positions/:id", get(handlers::position::get))
        // Orders
        .route("/api/v1/orders", post(handlers::order::create))
        .route("/api/v1/orders/:id", delete(handlers::order::cancel))
        .route("/api/v1/orders/my", get(handlers::order::my))
        .route("/api/v1/orders/book/:city", get(handlers::order::book))
        .route("/api/v1/orders/recent/:city", get(handlers::order::recent))
        // Oracle
        .route("/api/v1/oracle/submit", post(handlers::oracle::submit))
        .route("/api/v1/oracle/prices", get(handlers::oracle::prices))
        .route("/api/v1/oracle/status", get(handlers::oracle::status))
        .route_layer(middleware::from_fn(middleware::jwt_auth));

    let app = public_routes
        .merge(authenticated_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Stellanest API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

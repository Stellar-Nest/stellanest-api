# Stellanest — Backend API

Rust (Axum) backend API for the Stellanest real estate index trading platform.

## Stack

- **Framework:** [Axum](https://github.com/tokio-rs/axum) (Tokio)
- **Database:** PostgreSQL + TimescaleDB (via sqlx)
- **Cache:** Redis
- **Auth:** SEP-10 challenge-response → JWT
- **Real-time:** WebSocket (axum::extract::ws)

## Structure

```
src/
  main.rs              Entrypoint, router, state
  handlers/
    index.rs           City index queries
    position.rs        Position open/close/query
    order.rs           DEX order management
    auth.rs            SEP-10 auth → JWT
    oracle.rs          Oracle price submission
    analytics.rs       Volume, OI, leaderboard
  middleware/
    mod.rs             JWT auth middleware
  models/
    mod.rs             DB models, request/response types
  services/
    mod.rs             SEP-10 auth, index calc, position health
  ws/
    mod.rs             WebSocket hub and connection handler
  db/
    mod.rs             PostgreSQL connection
```

## Setup

```bash
# Set environment variables
export DATABASE_URL="postgres://stellanest:stellanest@localhost:5432/stellanest"
export REDIS_URL="redis://localhost:6379"
export JWT_SECRET="your-secret-key"
export STELLAR_NETWORK="testnet"

# Run
cargo run
```

## API Endpoints

| Group | Endpoint | Description |
|---|---|---|
| Auth | `POST /api/v1/auth/challenge` | SEP-10 challenge |
| Auth | `POST /api/v1/auth/token` | JWT issuance |
| Indices | `GET /api/v1/indices` | List all indices |
| Indices | `GET /api/v1/indices/:city` | Get city index |
| Indices | `GET /api/v1/indices/:city/history` | OHLCV data |
| Positions | `POST /api/v1/positions/open` | Open position |
| Positions | `POST /api/v1/positions/:id/close` | Close position |
| Positions | `GET /api/v1/positions/my` | User positions |
| Orders | `POST /api/v1/orders` | Place order |
| Orders | `DELETE /api/v1/orders/:id` | Cancel order |
| Orders | `GET /api/v1/orders/book/:city` | Order book |
| Oracle | `POST /api/v1/oracle/submit` | Submit price |
| Oracle | `GET /api/v1/oracle/prices` | Aggregated prices |
| Analytics | `GET /api/v1/analytics/volume` | Trading volume |
| WS | `GET /ws` | WebSocket streams |

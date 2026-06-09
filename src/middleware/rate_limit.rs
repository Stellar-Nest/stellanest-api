use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A single token bucket for rate limiting.
struct TokenBucket {
    /// Current number of available tokens.
    tokens: f64,
    /// Maximum tokens the bucket can hold (= burst size).
    max_tokens: f64,
    /// Tokens refilled per second.
    refill_rate: f64,
    /// Last time the bucket was refilled.
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume one token. Returns true if allowed, false if rate limited.
    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate limit configuration for a single route pattern.
#[derive(Clone)]
pub struct RateLimitRule {
    /// Maximum burst size (number of requests allowed in a short burst).
    pub max_burst: f64,
    /// Sustained requests per second allowed.
    pub refill_per_sec: f64,
}

/// In-memory per-IP rate limiter.
///
/// Keys are built as `"{ip}:{endpoint}"` so each IP+route pair has its own
/// token bucket.  The bucket map is wrapped in `Arc<Mutex<…>>` so all clones
/// share the same state.
///
/// This is an in-memory starting point; swap the inner map for Redis
/// counters when you need distributed rate limiting.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    rules: Arc<HashMap<String, RateLimitRule>>,
    /// Default rule applied when no route-specific rule matches.
    default_rule: RateLimitRule,
}

impl RateLimiter {
    /// Create a new rate limiter with route-specific rules.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let limiter = RateLimiter::new(
    ///     60,  // default: 60 requests burst, 1 req/sec refill
    ///     1.0,
    ///     vec![
    ///         ("/api/v1/auth/challenge".into(), RateLimitRule { max_burst: 10.0, refill_per_sec: 0.2 }),
    ///         ("/api/v1/auth/token".into(),      RateLimitRule { max_burst: 10.0, refill_per_sec: 0.2 }),
    ///     ],
    /// );
    /// ```
    pub fn new(
        default_max_burst: f64,
        default_refill_per_sec: f64,
        rules: Vec<(String, RateLimitRule)>,
    ) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            rules: Arc::new(rules.into_iter().collect()),
            default_rule: RateLimitRule {
                max_burst: default_max_burst,
                refill_per_sec: default_refill_per_sec,
            },
        }
    }

    /// Check whether the request from `ip` to `endpoint` is allowed.
    /// Returns `Ok(())` if allowed, `Err(StatusCode::TOO_MANY_REQUESTS)` otherwise.
    fn check(&self, ip: &str, endpoint: &str) -> Result<(), StatusCode> {
        let key = format!("{}:{}", ip, endpoint);
        let rule = self
            .rules
            .get(endpoint)
            .unwrap_or(&self.default_rule);

        let mut buckets = self.buckets.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let bucket = buckets
            .entry(key)
            .or_insert_with(|| TokenBucket::new(rule.max_burst, rule.refill_per_sec));

        if bucket.try_consume() {
            Ok(())
        } else {
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

/// Axum middleware: rate limit by IP address using the [`RateLimiter`]
/// stored in `AppState`.
///
/// Key format: `"rate_limit:{ip}:{endpoint}"`.
/// Returns `429 Too Many Requests` when the limit is exceeded.
pub async fn rate_limit_ip(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip().to_string();
    let endpoint = req.uri().path().to_string();

    limiter.check(&ip, &endpoint)?;

    Ok(next.run(req).await)
}

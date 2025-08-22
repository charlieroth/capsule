use axum::{
    Json,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};

use crate::auth::dtos::ErrorResponse;

#[derive(Clone)]
pub struct RateLimit {
    store: Arc<DashMap<String, RateLimitData>>,
    max_requests: u32,
    window_seconds: i64,
}

#[derive(Debug, Clone)]
struct RateLimitData {
    count: u32,
    window_start: DateTime<Utc>,
}

impl RateLimit {
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            max_requests,
            window_seconds,
        }
    }
}

/// IP-based rate limiting middleware.
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::State(rate_limit): axum::extract::State<RateLimit>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip().to_string();
    let now = Utc::now();

    let mut entry = rate_limit.store.entry(ip).or_insert_with(|| RateLimitData {
        count: 0,
        window_start: now,
    });

    let data = entry.value_mut();

    // Check if we need to reset the window
    if now.signed_duration_since(data.window_start) >= Duration::seconds(rate_limit.window_seconds)
    {
        data.count = 0;
        data.window_start = now;
    }

    data.count += 1;

    if data.count > rate_limit.max_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "Rate limit exceeded".to_string(),
            }),
        )
            .into_response();
    }

    next.run(req).await
}

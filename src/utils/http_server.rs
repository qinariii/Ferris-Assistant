//! HTTP server providing health check and Prometheus metrics endpoints.

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use crate::db::Pool;
use crate::utils::metrics;
use crate::utils::redis_cache;

/// Shared state for the HTTP server.
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub start_time: Instant,
}

/// Start the HTTP server on the given port.
/// Provides `/health`, `/metrics`, and `/ready` endpoints.
pub async fn start_http_server(pool: Pool, port: u16) {
    let state = Arc::new(AppState {
        pool,
        start_time: Instant::now(),
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind HTTP server");

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|e| log::error!("HTTP server error: {}", e));
    });
}

/// Health check endpoint - returns 200 if the bot process is running.
async fn health_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Json<serde_json::Value> {
    let uptime = state.start_time.elapsed().as_secs();
    axum::response::Json(serde_json::json!({
        "status": "ok",
        "uptime_seconds": uptime,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Readiness probe - checks DB and Redis connectivity.
async fn readiness_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> (axum::http::StatusCode, axum::response::Json<serde_json::Value>) {
    // Check PostgreSQL
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    // Check Redis
    let redis_ok = redis_cache::is_healthy().await;

    let all_ok = db_ok && redis_ok;
    let status = if all_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        axum::response::Json(serde_json::json!({
            "status": if all_ok { "ready" } else { "degraded" },
            "checks": {
                "database": db_ok,
                "redis": redis_ok,
            }
        })),
    )
}

/// Prometheus metrics endpoint.
async fn metrics_handler() -> (axum::http::StatusCode, String) {
    match metrics::gather_metrics() {
        Ok(output) => (axum::http::StatusCode::OK, output),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error gathering metrics: {}", e),
        ),
    }
}

//! # Canary API
//!
//! REST API server with WebSocket streaming for automotive diagnostics.
//!
//! Endpoints:
//! - `POST /api/v1/diagnostics/session` - Start a diagnostic session
//! - `GET /api/v1/diagnostics/dtc` - Read DTCs
//! - `POST /api/v1/diagnostics/clear-dtc` - Clear DTCs
//! - `GET /api/v1/data/ecus` - List available ECUs
//! - `WS /api/v1/stream/live` - WebSocket live streaming

use axum::Router;

pub mod handlers;
pub mod models;
pub mod router;
pub mod websocket;
pub mod error;

pub use error::ApiError;
pub use router::create_router;

/// Application configuration
#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
}

/// Create and configure the application
pub async fn create_app(config: AppConfig) -> Result<Router, Box<dyn std::error::Error>> {
    // Initialize database connection pool
    canary_database::initialize(&config.database_url).await?;

    // Create and return the router
    Ok(create_router())
}

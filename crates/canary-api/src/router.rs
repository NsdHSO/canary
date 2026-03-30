//! API router configuration with OpenAPI documentation.

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::models::*;
use crate::error::ErrorResponse;
use crate::websocket::{self, WsState};

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health_check,
        handlers::create_session,
        handlers::read_dtcs,
        handlers::clear_dtcs,
        handlers::list_ecus,
        websocket::ws_handler,
    ),
    components(schemas(
        HealthResponse,
        CreateSessionRequest,
        SessionResponse,
        DtcResponse,
        ReadDtcRequest,
        ClearDtcRequest,
        ClearDtcResponse,
        EcuInfo,
        LiveDataPoint,
        WsMessage,
        ErrorResponse,
    )),
    info(
        title = "Canary Diagnostics API",
        version = "1.0.0",
        description = "REST API for automotive diagnostics with real-time WebSocket streaming",
        license(name = "MIT OR Apache-2.0"),
    ),
    tags(
        (name = "system", description = "System health and status"),
        (name = "diagnostics", description = "Diagnostic operations (DTC read/clear, sessions)"),
        (name = "data", description = "ECU data and information"),
        (name = "streaming", description = "Real-time WebSocket data streaming"),
    )
)]
pub struct ApiDoc;

/// Create the main API router with all routes and middleware
pub fn create_router() -> Router {
    let ws_state = WsState::new(1024);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Railway health check (simple endpoint for monitoring)
        .route("/health", get(handlers::health))
        // Health check
        .route("/api/v1/health", get(handlers::health_check))
        // Diagnostics
        .route(
            "/api/v1/diagnostics/session",
            post(handlers::create_session),
        )
        .route("/api/v1/diagnostics/dtc", get(handlers::read_dtcs))
        .route(
            "/api/v1/diagnostics/clear-dtc",
            post(handlers::clear_dtcs),
        )
        // Data
        .route("/api/v1/data/ecus", get(handlers::list_ecus))
        // WebSocket streaming
        .route("/api/v1/stream/live", get(websocket::ws_handler))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // State
        .with_state(ws_state)
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec).unwrap();

        // Verify spec contains expected paths
        assert!(json.contains("/api/v1/health"));
        assert!(json.contains("/api/v1/diagnostics/session"));
        assert!(json.contains("/api/v1/diagnostics/dtc"));
        assert!(json.contains("/api/v1/diagnostics/clear-dtc"));
        assert!(json.contains("/api/v1/data/ecus"));
        assert!(json.contains("/api/v1/stream/live"));

        // Verify spec contains expected schemas
        assert!(json.contains("CreateSessionRequest"));
        assert!(json.contains("DtcResponse"));
        assert!(json.contains("EcuInfo"));
        assert!(json.contains("LiveDataPoint"));
    }

    #[test]
    fn test_router_creation() {
        // Should not panic
        let _router = create_router();
    }
}

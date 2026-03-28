//! API request handlers for diagnostic endpoints.

use axum::extract::Query;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;

use crate::error::{ApiError, ApiResult, ErrorResponse};
use crate::models::*;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse)
    ),
    tag = "system"
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0, // Would track actual uptime in production
    })
}

/// Create a new diagnostic session
#[utoipa::path(
    post,
    path = "/api/v1/diagnostics/session",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created", body = SessionResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "diagnostics"
)]
pub async fn create_session(
    Json(req): Json<CreateSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    // Validate session type
    let valid_types = ["default", "extended", "programming"];
    if !valid_types.contains(&req.session_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid session type '{}'. Valid types: {:?}",
            req.session_type, valid_types
        )));
    }

    let session = SessionResponse {
        session_id: uuid::Uuid::new_v4().to_string(),
        ecu_id: req.ecu_id,
        session_type: req.session_type,
        status: "active".to_string(),
        created_at: Utc::now(),
    };

    Ok(Json(session))
}

/// Query parameters for reading DTCs
#[derive(Debug, Deserialize)]
pub struct DtcQueryParams {
    pub ecu_id: Option<u32>,
    pub status_mask: Option<u8>,
}

/// Read DTCs from ECUs
#[utoipa::path(
    get,
    path = "/api/v1/diagnostics/dtc",
    params(
        ("ecu_id" = Option<u32>, Query, description = "Target ECU CAN ID"),
        ("status_mask" = Option<u8>, Query, description = "DTC status mask (0xFF for all)"),
    ),
    responses(
        (status = 200, description = "DTCs retrieved", body = Vec<DtcResponse>),
    ),
    tag = "diagnostics"
)]
pub async fn read_dtcs(
    Query(params): Query<DtcQueryParams>,
) -> ApiResult<Json<Vec<DtcResponse>>> {
    // In production, this would communicate with the hardware adapter
    // and UDS session to read actual DTCs. For now, return sample data.
    let _ecu_id = params.ecu_id.unwrap_or(0x7E0);
    let _status_mask = params.status_mask.unwrap_or(0xFF);

    let dtcs = vec![
        DtcResponse {
            code: "P0301".to_string(),
            description: "Cylinder 1 Misfire Detected".to_string(),
            status: 0x08,
            severity: "high".to_string(),
            ecu_id: 0x7E0,
            system: "Powertrain".to_string(),
        },
        DtcResponse {
            code: "P0420".to_string(),
            description: "Catalyst System Efficiency Below Threshold (Bank 1)".to_string(),
            status: 0x00,
            severity: "medium".to_string(),
            ecu_id: 0x7E0,
            system: "Powertrain".to_string(),
        },
    ];

    Ok(Json(dtcs))
}

/// Clear DTCs from an ECU
#[utoipa::path(
    post,
    path = "/api/v1/diagnostics/clear-dtc",
    request_body = ClearDtcRequest,
    responses(
        (status = 200, description = "DTCs cleared", body = ClearDtcResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "diagnostics"
)]
pub async fn clear_dtcs(
    Json(req): Json<ClearDtcRequest>,
) -> ApiResult<Json<ClearDtcResponse>> {
    // In production, send UDS 0x14 (Clear Diagnostic Information)
    let cleared_count = if req.codes.is_empty() {
        // Clear all DTCs
        2 // sample count
    } else {
        req.codes.len() as u32
    };

    Ok(Json(ClearDtcResponse {
        success: true,
        cleared_count,
        message: format!(
            "Cleared {} DTCs from ECU 0x{:03X}",
            cleared_count, req.ecu_id
        ),
    }))
}

/// Query parameters for listing ECUs
#[derive(Debug, Deserialize)]
pub struct EcuQueryParams {
    pub manufacturer: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// List available ECUs
#[utoipa::path(
    get,
    path = "/api/v1/data/ecus",
    params(
        ("manufacturer" = Option<String>, Query, description = "Filter by manufacturer"),
        ("page" = Option<u32>, Query, description = "Page number (1-indexed)"),
        ("per_page" = Option<u32>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "ECU list", body = PaginatedResponse<EcuInfo>),
    ),
    tag = "data"
)]
pub async fn list_ecus(
    Query(params): Query<EcuQueryParams>,
) -> ApiResult<Json<PaginatedResponse<EcuInfo>>> {
    let _page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // In production, this queries the canary-data crate
    let ecus = vec![
        EcuInfo {
            id: "vw_golf_2020_ecm".to_string(),
            manufacturer: "Volkswagen".to_string(),
            model: "Golf".to_string(),
            year_range: "2018-2022".to_string(),
            ecu_type: "ECM".to_string(),
            can_id: 0x7E0,
            protocols: vec!["UDS".to_string(), "KWP2000".to_string()],
        },
        EcuInfo {
            id: "gm_silverado_2021_ecm".to_string(),
            manufacturer: "General Motors".to_string(),
            model: "Silverado".to_string(),
            year_range: "2019-2022".to_string(),
            ecu_type: "ECM".to_string(),
            can_id: 0x7E0,
            protocols: vec!["UDS".to_string()],
        },
    ];

    // Apply manufacturer filter
    let filtered: Vec<EcuInfo> = if let Some(ref mfr) = params.manufacturer {
        ecus.into_iter()
            .filter(|e| e.manufacturer.to_lowercase().contains(&mfr.to_lowercase()))
            .collect()
    } else {
        ecus
    };

    let total = filtered.len() as u64;

    Ok(Json(PaginatedResponse {
        items: filtered,
        total,
        page: _page,
        per_page,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.status, "healthy");
    }

    #[tokio::test]
    async fn test_create_session_valid() {
        let req = CreateSessionRequest {
            ecu_id: 0x7E0,
            session_type: "extended".to_string(),
            adapter: None,
        };
        let result = create_session(Json(req)).await;
        assert!(result.is_ok());
        let session = result.unwrap().0;
        assert_eq!(session.ecu_id, 0x7E0);
        assert_eq!(session.session_type, "extended");
        assert_eq!(session.status, "active");
    }

    #[tokio::test]
    async fn test_create_session_invalid_type() {
        let req = CreateSessionRequest {
            ecu_id: 0x7E0,
            session_type: "invalid".to_string(),
            adapter: None,
        };
        let result = create_session(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_dtcs_all() {
        let req = ClearDtcRequest {
            ecu_id: 0x7E0,
            codes: vec![],
        };
        let result = clear_dtcs(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap().0;
        assert!(resp.success);
    }

    #[tokio::test]
    async fn test_clear_dtcs_specific() {
        let req = ClearDtcRequest {
            ecu_id: 0x7E0,
            codes: vec!["P0301".into()],
        };
        let result = clear_dtcs(Json(req)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.cleared_count, 1);
    }
}

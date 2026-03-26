// Re-export models
pub use canary_models::{embedded, internal, CanaryError, Result};

// Re-export services
pub use canary_pinout::PinoutService;
pub use canary_protocol::{CanDecoder, KLineDecoder, ProtocolDecoder, ProtocolFactory};
pub use canary_dtc::DtcService;
pub use canary_service_proc::ServiceProcedureService;

/// Initialize the canary library with optional database connection
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> Result<(), canary_core::CanaryError> {
/// // Initialize without database (embedded data only)
/// canary_core::initialize(None).await?;
///
/// // Initialize with SQLite database
/// canary_core::initialize(Some("sqlite://canary.db")).await?;
///
/// // Initialize with PostgreSQL
/// canary_core::initialize(Some("postgresql://user:pass@localhost/canary_db")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn initialize(db_url: Option<&str>) -> Result<()> {
    if let Some(url) = db_url {
        canary_database::initialize(url)
            .await
            .map_err(|e| CanaryError::DatabaseError(e))?;
    }
    Ok(())
}

/// Check if database has been initialized
pub fn is_database_initialized() -> bool {
    canary_database::is_initialized()
}

/// Main facade providing convenient access to all services
///
/// # Examples
///
/// ```rust
/// use canary_core::{PinoutService, ProtocolFactory, ProtocolDecoder, DtcService, ServiceProcedureService};
///
/// // Pin lookups
/// let obd2 = PinoutService::get_obd2_pinout().unwrap();
/// println!("Pin 6: {}", obd2.pins[5].signal_name);
///
/// // Protocol decoding
/// let decoder = ProtocolFactory::create_can_decoder().unwrap();
/// let raw_bytes = vec![0x00, 0x00, 0x01, 0x23, 0x01, 0x02];
/// let frame = decoder.decode(&raw_bytes).unwrap();
///
/// // DTC lookup
/// let dtc = DtcService::lookup_code("P0301").unwrap();
/// println!("DTC: {}", dtc.description);
///
/// // Service procedures
/// let proc = ServiceProcedureService::get_procedure("oil_change").unwrap();
/// println!("Steps: {}", proc.steps.len());
/// ```
pub struct Canary;

impl Canary {
    /// Access pinout service
    pub fn pinout() -> &'static PinoutService {
        &PinoutService
    }

    /// Access protocol factory
    pub fn protocol() -> ProtocolFactory {
        ProtocolFactory
    }

    /// Access DTC service
    pub fn dtc() -> &'static DtcService {
        &DtcService
    }

    /// Access service procedure service
    pub fn procedures() -> &'static ServiceProcedureService {
        &ServiceProcedureService
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canary_pinout_access() {
        let pinout = PinoutService::get_obd2_pinout().unwrap();
        assert_eq!(pinout.pins.len(), 16);
    }

    #[test]
    fn test_canary_protocol_access() {
        let protocols = ProtocolFactory::list_available_protocols();
        assert!(!protocols.is_empty());
    }

    #[test]
    fn test_canary_dtc_access() {
        let dtc = DtcService::lookup_code("P0301").unwrap();
        assert_eq!(dtc.system, embedded::DtcSystem::Powertrain);
    }

    #[test]
    fn test_canary_procedures_access() {
        let proc = ServiceProcedureService::get_procedure("oil_change").unwrap();
        assert_eq!(proc.category, embedded::ProcedureCategory::Maintenance);
    }

    #[tokio::test]
    async fn test_initialize_without_database() {
        let result = initialize(None).await;
        assert!(result.is_ok());
    }
}

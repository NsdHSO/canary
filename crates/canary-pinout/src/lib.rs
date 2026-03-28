use canary_data::PINOUTS;
use canary_models::{
    embedded::{ConnectorPinout, EcuPinout, ModuleType},
    Result,
    CanaryError,
};

pub struct PinoutService;

impl PinoutService {
    /// Get universal OBD-II J1962 16-pin pinout
    pub fn get_obd2_pinout() -> Result<&'static ConnectorPinout> {
        PINOUTS
            .get("obd2_j1962")
            .ok_or_else(|| CanaryError::NotFound("OBD-II pinout".into()))
    }

    /// Get manufacturer-specific pinouts matching vehicle criteria
    ///
    /// Uses declarative filter/collect pattern
    pub fn get_manufacturer_pinout(
        manufacturer: &str,
        model: &str,
        year: u16,
    ) -> Result<Vec<&'static ConnectorPinout>> {
        let results: Vec<&ConnectorPinout> = PINOUTS
            .values()
            .filter(|p| p.matches_vehicle(manufacturer, model, year))
            .collect();

        if results.is_empty() {
            Err(CanaryError::NotFound(format!(
                "No pinouts found for {} {} {}",
                manufacturer, model, year
            )))
        } else {
            Ok(results)
        }
    }

    /// Get all available pinouts
    pub fn list_all() -> Vec<&'static ConnectorPinout> {
        PINOUTS.values().collect()
    }

    /// Get pinout by ID
    pub fn get_by_id(id: &str) -> Result<&'static ConnectorPinout> {
        PINOUTS
            .get(id)
            .ok_or_else(|| CanaryError::NotFound(format!("Pinout {}", id)))
    }

    /// List available manufacturers
    pub fn list_manufacturers() -> Vec<&'static str> {
        canary_data::list_manufacturers()
    }

    /// Get ECUs by manufacturer
    pub fn get_ecus_by_manufacturer(manufacturer: &str) -> Result<Vec<&'static EcuPinout>> {
        if let Some(ecus) = canary_data::load_manufacturer_ecus(manufacturer) {
            Ok(ecus.values().collect())
        } else {
            Err(CanaryError::NotFound(format!("Manufacturer: {}", manufacturer)))
        }
    }

    /// Get ECUs by module type (across all manufacturers)
    pub fn get_ecus_by_module_type(module_type: ModuleType) -> Result<Vec<&'static EcuPinout>> {
        let mut result = Vec::new();

        for manufacturer in Self::list_manufacturers() {
            if let Some(ecus) = canary_data::load_manufacturer_ecus(manufacturer) {
                result.extend(
                    ecus.values()
                        .filter(|ecu| ecu.module_type == module_type)
                );
            }
        }

        Ok(result)
    }

    /// Get specific ECU by ID
    pub fn get_ecu_by_id(id: &str) -> Result<&'static EcuPinout> {
        // Extract manufacturer from ID (format: mfr_model_year_module)
        let manufacturer = id.split('_').next()
            .ok_or_else(|| CanaryError::NotFound(format!("Invalid ECU ID: {}", id)))?;

        if let Some(ecus) = canary_data::load_manufacturer_ecus(manufacturer) {
            ecus.get(id)
                .ok_or_else(|| CanaryError::NotFound(format!("ECU ID: {}", id)))
        } else {
            Err(CanaryError::NotFound(format!("Manufacturer: {}", manufacturer)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obd2_pinout() {
        let pinout = PinoutService::get_obd2_pinout().unwrap();
        assert_eq!(pinout.id, "obd2_j1962");
        assert_eq!(pinout.pins.len(), 16);
    }

    #[test]
    fn test_get_by_id() {
        let pinout = PinoutService::get_by_id("obd2_j1962").unwrap();
        assert_eq!(pinout.connector_type, "J1962 16-pin");
    }

    #[test]
    fn test_get_by_invalid_id() {
        let result = PinoutService::get_by_id("invalid_id");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_all() {
        let pinouts = PinoutService::list_all();
        assert!(!pinouts.is_empty());
    }
}

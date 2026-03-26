use canary_data::DTC_CODES;
use canary_models::{
    embedded::{DiagnosticCode, DtcSystem},
    CanaryError, Result,
};

pub struct DtcService;

impl DtcService {
    /// Lookup DTC code in embedded database
    pub fn lookup_code(code: &str) -> Result<&'static DiagnosticCode> {
        DTC_CODES
            .get(code)
            .ok_or_else(|| CanaryError::DtcNotFound(code.into()))
    }

    /// Parse DTC system from code using declarative pattern matching
    pub fn parse_system(code: &str) -> Result<DtcSystem> {
        code.chars()
            .next()
            .and_then(|c| match c {
                'P' => Some(DtcSystem::Powertrain),
                'B' => Some(DtcSystem::Body),
                'C' => Some(DtcSystem::Chassis),
                'U' => Some(DtcSystem::Network),
                _ => None,
            })
            .ok_or(CanaryError::InvalidDtcFormat)
    }

    /// Get all DTCs for a specific system
    pub fn get_by_system(system: DtcSystem) -> Vec<&'static DiagnosticCode> {
        DTC_CODES
            .values()
            .filter(|dtc| dtc.system == system)
            .collect()
    }

    /// List all available DTC codes
    pub fn list_all() -> Vec<&'static DiagnosticCode> {
        DTC_CODES.values().collect()
    }

    /// Search DTCs by description keyword (case-insensitive)
    pub fn search_by_description(keyword: &str) -> Vec<&'static DiagnosticCode> {
        let keyword_lower = keyword.to_lowercase();

        DTC_CODES
            .values()
            .filter(|dtc| dtc.description.to_lowercase().contains(&keyword_lower))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_code() {
        let dtc = DtcService::lookup_code("P0301").unwrap();
        assert_eq!(dtc.code, "P0301");
        assert_eq!(dtc.system, DtcSystem::Powertrain);
    }

    #[test]
    fn test_lookup_invalid_code() {
        let result = DtcService::lookup_code("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_system() {
        assert_eq!(
            DtcService::parse_system("P0301").unwrap(),
            DtcSystem::Powertrain
        );
        assert_eq!(
            DtcService::parse_system("B0001").unwrap(),
            DtcSystem::Body
        );
        assert_eq!(
            DtcService::parse_system("C0123").unwrap(),
            DtcSystem::Chassis
        );
        assert_eq!(
            DtcService::parse_system("U0001").unwrap(),
            DtcSystem::Network
        );
    }

    #[test]
    fn test_parse_invalid_system() {
        let result = DtcService::parse_system("X0001");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_by_system() {
        let powertrain_codes = DtcService::get_by_system(DtcSystem::Powertrain);
        assert!(!powertrain_codes.is_empty());
        assert!(powertrain_codes.iter().all(|dtc| dtc.system == DtcSystem::Powertrain));
    }

    #[test]
    fn test_search_by_description() {
        let results = DtcService::search_by_description("misfire");
        assert!(!results.is_empty());
        assert!(results.iter().all(|dtc| dtc.description.to_lowercase().contains("misfire")));
    }

    #[test]
    fn test_list_all() {
        let all_codes = DtcService::list_all();
        assert!(!all_codes.is_empty());
    }
}

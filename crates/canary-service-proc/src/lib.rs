use canary_data::PROCEDURES;
use canary_models::{
    embedded::{ProcedureCategory, ServiceProcedure},
    CanaryError, Result,
};

pub struct ServiceProcedureService;

impl ServiceProcedureService {
    /// Get service procedure by ID
    pub fn get_procedure(id: &str) -> Result<&'static ServiceProcedure> {
        PROCEDURES
            .get(id)
            .ok_or_else(|| CanaryError::ProcedureNotFound(id.into()))
    }

    /// Search procedures by category using declarative filter
    pub fn search_by_category(category: ProcedureCategory) -> Vec<&'static ServiceProcedure> {
        PROCEDURES
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Search procedures by name keyword (case-insensitive)
    pub fn search_by_name(keyword: &str) -> Vec<&'static ServiceProcedure> {
        let keyword_lower = keyword.to_lowercase();

        PROCEDURES
            .values()
            .filter(|p| p.name.to_lowercase().contains(&keyword_lower))
            .collect()
    }

    /// List all available procedures
    pub fn list_all() -> Vec<&'static ServiceProcedure> {
        PROCEDURES.values().collect()
    }

    /// Get procedures within estimated time range
    pub fn get_by_time_range(
        min_minutes: u32,
        max_minutes: u32,
    ) -> Vec<&'static ServiceProcedure> {
        PROCEDURES
            .values()
            .filter(|p| {
                p.estimated_time_minutes
                    .map_or(false, |t| t >= min_minutes && t <= max_minutes)
            })
            .collect()
    }

    /// Get all maintenance procedures
    pub fn get_maintenance_procedures() -> Vec<&'static ServiceProcedure> {
        Self::search_by_category(ProcedureCategory::Maintenance)
    }

    /// Get all repair procedures
    pub fn get_repair_procedures() -> Vec<&'static ServiceProcedure> {
        Self::search_by_category(ProcedureCategory::Repair)
    }

    /// Get all diagnostic procedures
    pub fn get_diagnostic_procedures() -> Vec<&'static ServiceProcedure> {
        Self::search_by_category(ProcedureCategory::Diagnostic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_procedure() {
        let proc = ServiceProcedureService::get_procedure("oil_change").unwrap();
        assert_eq!(proc.id, "oil_change");
        assert_eq!(proc.category, ProcedureCategory::Maintenance);
    }

    #[test]
    fn test_get_invalid_procedure() {
        let result = ServiceProcedureService::get_procedure("invalid_id");
        assert!(result.is_err());
    }

    #[test]
    fn test_search_by_category() {
        let maintenance = ServiceProcedureService::search_by_category(ProcedureCategory::Maintenance);
        assert!(!maintenance.is_empty());
        assert!(maintenance.iter().all(|p| p.category == ProcedureCategory::Maintenance));
    }

    #[test]
    fn test_search_by_name() {
        let results = ServiceProcedureService::search_by_name("oil");
        assert!(!results.is_empty());
        assert!(results.iter().all(|p| p.name.to_lowercase().contains("oil")));
    }

    #[test]
    fn test_get_maintenance_procedures() {
        let procedures = ServiceProcedureService::get_maintenance_procedures();
        assert!(!procedures.is_empty());
    }

    #[test]
    fn test_list_all() {
        let all_procedures = ServiceProcedureService::list_all();
        assert!(!all_procedures.is_empty());
    }

    #[test]
    fn test_get_by_time_range() {
        let procedures = ServiceProcedureService::get_by_time_range(20, 40);
        assert!(procedures.iter().all(|p| {
            p.estimated_time_minutes.map_or(false, |t| t >= 20 && t <= 40)
        }));
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProcedure {
    pub id: String,
    pub name: String,
    pub category: ProcedureCategory,
    pub description: String,
    pub steps: Vec<ProcedureStep>,
    pub estimated_time_minutes: Option<u32>,
    pub tools_required: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcedureCategory {
    Maintenance,
    Repair,
    Diagnostic,
    Installation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    pub order: u8,
    pub instruction: String,
    pub warnings: Vec<String>,
}

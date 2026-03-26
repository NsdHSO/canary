use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCode {
    pub code: String,
    pub system: DtcSystem,
    pub description: String,
    pub manufacturer_specific: Option<ManufacturerDtcInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DtcSystem {
    Powertrain,  // P codes
    Body,        // B codes
    Chassis,     // C codes
    Network,     // U codes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturerDtcInfo {
    pub manufacturer_id: String,
    pub additional_notes: Option<String>,
    pub common_causes: Vec<String>,
}

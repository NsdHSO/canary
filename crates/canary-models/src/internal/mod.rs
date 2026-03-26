use serde::{Deserialize, Serialize};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPinoutRequest {
    pub connector_type: String,
    pub vehicle: VehicleInfo,
    pub pins: Vec<PinData>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    pub manufacturer: String,
    pub brand: String,
    pub model: String,
    pub year: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinData {
    pub pin_number: u8,
    pub signal_name: String,
    pub voltage: Option<f32>,
    pub protocol: Option<String>,
    pub notes: Option<String>,
}

// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPinoutResponse {
    pub id: i64,
    pub user_id: Option<String>,
    pub connector_type: String,
    pub vehicle_info: VehicleInfo,
    pub pin_mappings: Vec<PinData>,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtcNoteRequest {
    pub code: String,
    pub vehicle: Option<VehicleInfo>,
    pub notes: String,
    pub actions: Vec<RepairAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    pub description: String,
    pub parts_replaced: Vec<String>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtcNoteResponse {
    pub id: i64,
    pub user_id: Option<String>,
    pub dtc_code: String,
    pub vehicle_info: Option<VehicleInfo>,
    pub notes: String,
    pub repair_actions: Vec<RepairAction>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLogRequest {
    pub vehicle: VehicleInfo,
    pub procedure_id: String,
    pub date: chrono::NaiveDate,
    pub mileage: Option<i32>,
    pub parts: Vec<PartUsed>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartUsed {
    pub part_name: String,
    pub part_number: Option<String>,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLogResponse {
    pub id: i64,
    pub user_id: Option<String>,
    pub vehicle_info: VehicleInfo,
    pub procedure_id: Option<String>,
    pub performed_date: chrono::NaiveDate,
    pub mileage: Option<i32>,
    pub parts_used: Vec<PartUsed>,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

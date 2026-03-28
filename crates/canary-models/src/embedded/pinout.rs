use serde::{Deserialize, Serialize};
use super::manufacturer::VehicleModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorPinout {
    pub id: String,
    pub connector_type: String,
    pub manufacturer_id: Option<String>,
    pub vehicle_models: Vec<VehicleModel>,
    pub pins: Vec<PinMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinMapping {
    pub pin_number: u8,
    pub signal_name: String,
    pub voltage: Option<f32>,
    pub protocol: Option<String>,
    pub notes: Option<String>,
}

impl ConnectorPinout {
    pub fn matches_vehicle(&self, manufacturer: &str, model: &str, year: u16) -> bool {
        self.vehicle_models
            .iter()
            .any(|vm| {
                vm.manufacturer == manufacturer
                    && vm.model == model
                    && vm.years.contains(&year)
            })
    }
}

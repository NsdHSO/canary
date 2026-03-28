use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manufacturer {
    pub id: String,
    pub name: String,
    pub brands: Vec<Brand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brand {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleModel {
    pub manufacturer: String,
    pub model: String,
    pub years: Vec<u16>,
    pub engine: Option<String>,
}

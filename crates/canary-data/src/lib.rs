use canary_models::embedded::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Embedded data files (compile-time inclusion)
static MANUFACTURERS_TOML: &str = include_str!("../data/manufacturers.toml");
static OBD2_PINOUT_TOML: &str = include_str!("../data/pinouts/universal/obd2_j1962.toml");
static CAN_PROTOCOL_TOML: &str = include_str!("../data/protocols/can_2.0b.toml");
static POWERTRAIN_DTC_TOML: &str = include_str!("../data/dtc/powertrain.toml");
static OIL_CHANGE_PROC_TOML: &str = include_str!("../data/service_procedures/oil_change.toml");
static BRAKE_BLEEDING_PROC_TOML: &str = include_str!("../data/service_procedures/brake_bleeding.toml");

// Lazy static data structures (parsed once, cached forever)
pub static MANUFACTURERS: Lazy<HashMap<String, Manufacturer>> = Lazy::new(|| {
    #[derive(serde::Deserialize)]
    struct ManufacturerList {
        manufacturers: Vec<Manufacturer>,
    }

    toml::from_str::<ManufacturerList>(MANUFACTURERS_TOML)
        .expect("Failed to parse manufacturers.toml")
        .manufacturers
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect()
});

pub static PINOUTS: Lazy<HashMap<String, ConnectorPinout>> = Lazy::new(|| {
    let obd2 = toml::from_str::<ConnectorPinout>(OBD2_PINOUT_TOML)
        .expect("Failed to parse obd2_j1962.toml");

    [(obd2.id.clone(), obd2)]
        .into_iter()
        .collect()
});

pub static PROTOCOLS: Lazy<HashMap<String, ProtocolSpec>> = Lazy::new(|| {
    let can = toml::from_str::<ProtocolSpec>(CAN_PROTOCOL_TOML)
        .expect("Failed to parse can_2.0b.toml");

    [(can.id.clone(), can)]
        .into_iter()
        .collect()
});

pub static DTC_CODES: Lazy<HashMap<String, DiagnosticCode>> = Lazy::new(|| {
    #[derive(serde::Deserialize)]
    struct DtcList {
        dtc_codes: Vec<DiagnosticCode>,
    }

    toml::from_str::<DtcList>(POWERTRAIN_DTC_TOML)
        .expect("Failed to parse powertrain.toml")
        .dtc_codes
        .into_iter()
        .map(|dtc| (dtc.code.clone(), dtc))
        .collect()
});

pub static PROCEDURES: Lazy<HashMap<String, ServiceProcedure>> = Lazy::new(|| {
    let oil_change = toml::from_str::<ServiceProcedure>(OIL_CHANGE_PROC_TOML)
        .expect("Failed to parse oil_change.toml");

    let brake_bleeding = toml::from_str::<ServiceProcedure>(BRAKE_BLEEDING_PROC_TOML)
        .expect("Failed to parse brake_bleeding.toml");

    [
        (oil_change.id.clone(), oil_change),
        (brake_bleeding.id.clone(), brake_bleeding),
    ]
    .into_iter()
    .collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturers_load() {
        assert!(!MANUFACTURERS.is_empty());
        assert!(MANUFACTURERS.contains_key("universal"));
        assert!(MANUFACTURERS.contains_key("vw_group"));
    }

    #[test]
    fn test_obd2_pinout_loads() {
        let obd2 = PINOUTS.get("obd2_j1962").expect("OBD-II pinout not found");
        assert_eq!(obd2.pins.len(), 16);
        assert_eq!(obd2.pins[5].signal_name, "CAN_H (ISO 15765-4)");
    }

    #[test]
    fn test_protocols_load() {
        let can = PROTOCOLS.get("can_2.0b").expect("CAN protocol not found");
        assert_eq!(can.name, "CAN 2.0B");
        assert_eq!(can.bit_rate, 500000);
    }

    #[test]
    fn test_dtc_codes_load() {
        assert!(!DTC_CODES.is_empty());
        let p0301 = DTC_CODES.get("P0301").expect("P0301 not found");
        assert_eq!(p0301.system, DtcSystem::Powertrain);
    }

    #[test]
    fn test_procedures_load() {
        let oil_change = PROCEDURES.get("oil_change").expect("Oil change procedure not found");
        assert_eq!(oil_change.category, ProcedureCategory::Maintenance);
        assert_eq!(oil_change.steps.len(), 10);
    }
}

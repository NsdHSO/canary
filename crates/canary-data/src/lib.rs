pub mod lazy;

use canary_models::embedded::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Embedded data files (compile-time inclusion)
static MANUFACTURERS_TOML: &str = include_str!("../data/manufacturers.toml");
static OBD2_PINOUT_TOML: &str = include_str!("../data/universal/obd2_j1962.toml");
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

/// Helper struct for deserializing ECU TOML files
#[derive(serde::Deserialize)]
struct EcuFile {
    ecu: Vec<EcuPinout>,
}

/// Lazy-loaded VW ECU data
pub static VW_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/vw/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress VW ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse VW ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    // Load TCM
    const TCM_GZ: &[u8] = include_bytes!("../data/manufacturers/vw/tcm.toml.gz");
    let tcm_toml = lazy::decompress_gzip(TCM_GZ).expect("Failed to decompress VW TCM");
    let tcm_file: EcuFile = toml::from_str(&tcm_toml).expect("Failed to parse VW TCM");
    for ecu in tcm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded Audi ECU data
pub static AUDI_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/audi/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress Audi ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse Audi ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded GM ECU data
pub static GM_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/gm/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress GM ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse GM ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    // Load PCM
    const PCM_GZ: &[u8] = include_bytes!("../data/manufacturers/gm/pcm.toml.gz");
    let pcm_toml = lazy::decompress_gzip(PCM_GZ).expect("Failed to decompress GM PCM");
    let pcm_file: EcuFile = toml::from_str(&pcm_toml).expect("Failed to parse GM PCM");
    for ecu in pcm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded Ford ECU data
pub static FORD_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load PCM
    const PCM_GZ: &[u8] = include_bytes!("../data/manufacturers/ford/pcm.toml.gz");
    let pcm_toml = lazy::decompress_gzip(PCM_GZ).expect("Failed to decompress Ford PCM");
    let pcm_file: EcuFile = toml::from_str(&pcm_toml).expect("Failed to parse Ford PCM");
    for ecu in pcm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    // Load BCM
    const BCM_GZ: &[u8] = include_bytes!("../data/manufacturers/ford/bcm.toml.gz");
    let bcm_toml = lazy::decompress_gzip(BCM_GZ).expect("Failed to decompress Ford BCM");
    let bcm_file: EcuFile = toml::from_str(&bcm_toml).expect("Failed to parse Ford BCM");
    for ecu in bcm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded Toyota ECU data
pub static TOYOTA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/toyota/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress Toyota ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse Toyota ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    // Load RAV4 ECM
    const RAV4_ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/toyota/rav4_ecm.toml.gz");
    let rav4_ecm_toml = lazy::decompress_gzip(RAV4_ECM_GZ).expect("Failed to decompress Toyota RAV4 ECM");
    let rav4_ecm_file: EcuFile = toml::from_str(&rav4_ecm_toml).expect("Failed to parse Toyota RAV4 ECM");
    for ecu in rav4_ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded BMW ECU data
pub static BMW_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/bmw/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress BMW ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse BMW ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Lazy-loaded Skoda ECU data
pub static SKODA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Load ECM
    const ECM_GZ: &[u8] = include_bytes!("../data/manufacturers/skoda/ecm.toml.gz");
    let ecm_toml = lazy::decompress_gzip(ECM_GZ).expect("Failed to decompress Skoda ECM");
    let ecm_file: EcuFile = toml::from_str(&ecm_toml).expect("Failed to parse Skoda ECM");
    for ecu in ecm_file.ecu {
        map.insert(ecu.id.clone(), ecu);
    }

    map
});

/// Load manufacturer ECU data
pub fn load_manufacturer_ecus(manufacturer: &str) -> Option<&'static HashMap<String, EcuPinout>> {
    match manufacturer {
        "vw" => Some(&VW_ECUS),
        "audi" => Some(&AUDI_ECUS),
        "gm" => Some(&GM_ECUS),
        "ford" => Some(&FORD_ECUS),
        "toyota" => Some(&TOYOTA_ECUS),
        "bmw" => Some(&BMW_ECUS),
        "skoda" => Some(&SKODA_ECUS),
        _ => None,
    }
}

/// List available manufacturers
pub fn list_manufacturers() -> Vec<&'static str> {
    vec!["vw", "audi", "gm", "ford", "toyota", "bmw", "skoda"]
}

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

    #[test]
    fn test_list_manufacturers() {
        let manufacturers = list_manufacturers();
        assert!(manufacturers.contains(&"vw"));
        assert!(manufacturers.contains(&"gm"));
        assert!(manufacturers.len() >= 6);
    }

    #[test]
    fn test_load_manufacturer_ecus() {
        let vw_ecus = load_manufacturer_ecus("vw");
        assert!(vw_ecus.is_some());

        let invalid = load_manufacturer_ecus("invalid");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_lazy_load_performance_vw() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*VW_ECUS;
        let duration = start.elapsed();
        println!("VW loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "VW loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "VW ECUs should not be empty");
    }

    #[test]
    fn test_lazy_load_performance_audi() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*AUDI_ECUS;
        let duration = start.elapsed();
        println!("Audi loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "Audi loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "Audi ECUs should not be empty");
    }

    #[test]
    fn test_lazy_load_performance_gm() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*GM_ECUS;
        let duration = start.elapsed();
        println!("GM loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "GM loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "GM ECUs should not be empty");
    }

    #[test]
    fn test_lazy_load_performance_ford() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*FORD_ECUS;
        let duration = start.elapsed();
        println!("Ford loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "Ford loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "Ford ECUs should not be empty");
    }

    #[test]
    fn test_lazy_load_performance_toyota() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*TOYOTA_ECUS;
        let duration = start.elapsed();
        println!("Toyota loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "Toyota loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "Toyota ECUs should not be empty");
    }

    #[test]
    fn test_lazy_load_performance_bmw() {
        use std::time::Instant;
        let start = Instant::now();
        let ecus = &*BMW_ECUS;
        let duration = start.elapsed();
        println!("BMW loaded in {}ms with {} ECUs", duration.as_millis(), ecus.len());
        assert!(duration.as_millis() < 100, "BMW loaded in {}ms", duration.as_millis());
        assert!(!ecus.is_empty(), "BMW ECUs should not be empty");
    }

    #[test]
    fn test_vw_ecus_content() {
        let ecus = &*VW_ECUS;
        // Verify we have ECM and TCM data
        assert!(ecus.len() >= 2, "Expected at least 2 ECUs (ECM + TCM)");

        // Check that ECU IDs are present
        for (id, ecu) in ecus.iter() {
            assert_eq!(id, &ecu.id);
            assert!(!ecu.connectors.is_empty(), "ECU {} should have connectors", id);
        }
    }

    #[test]
    fn test_all_manufacturers_accessible() {
        let manufacturers = list_manufacturers();
        for manufacturer in manufacturers {
            let ecus = load_manufacturer_ecus(manufacturer);
            assert!(ecus.is_some(), "Manufacturer {} should have ECU data", manufacturer);
            assert!(!ecus.unwrap().is_empty(), "Manufacturer {} should have at least one ECU", manufacturer);
        }
    }
}

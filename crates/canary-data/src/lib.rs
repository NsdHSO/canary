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

/// Helper to load and parse a gzipped ECU TOML file
fn load_gz_ecus(gz_data: &[u8], label: &str) -> Vec<EcuPinout> {
    let toml_str = lazy::decompress_gzip(gz_data)
        .unwrap_or_else(|e| panic!("Failed to decompress {}: {}", label, e));
    let file: EcuFile = toml::from_str(&toml_str)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", label, e));
    file.ecu
}

/// Helper to insert ECUs into a map
fn insert_ecus(map: &mut HashMap<String, EcuPinout>, ecus: Vec<EcuPinout>) {
    for ecu in ecus {
        map.insert(ecu.id.clone(), ecu);
    }
}

/// Lazy-loaded VW ECU data
pub static VW_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/vw/ecm.toml.gz"), "VW ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/vw/tcm.toml.gz"), "VW TCM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/vw/additional.toml.gz"), "VW Additional"));
    map
});

/// Lazy-loaded Audi ECU data
pub static AUDI_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/audi/ecm.toml.gz"), "Audi ECM"));
    map
});

/// Lazy-loaded GM ECU data
pub static GM_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/gm/ecm.toml.gz"), "GM ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/gm/pcm.toml.gz"), "GM PCM"));
    map
});

/// Lazy-loaded Ford ECU data
pub static FORD_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/ford/pcm.toml.gz"), "Ford PCM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/ford/bcm.toml.gz"), "Ford BCM"));
    map
});

/// Lazy-loaded Toyota ECU data
pub static TOYOTA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/toyota/ecm.toml.gz"), "Toyota ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/toyota/rav4_ecm.toml.gz"), "Toyota RAV4 ECM"));
    map
});

/// Lazy-loaded BMW ECU data
pub static BMW_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/bmw/ecm.toml.gz"), "BMW ECM"));
    map
});

/// Lazy-loaded Skoda ECU data
pub static SKODA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/skoda/ecm.toml.gz"), "Skoda ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/skoda/additional.toml.gz"), "Skoda Additional"));
    map
});

/// Lazy-loaded Honda ECU data
pub static HONDA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/honda/ecm.toml.gz"), "Honda ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/honda/tcm.toml.gz"), "Honda TCM"));
    map
});

/// Lazy-loaded Nissan ECU data
pub static NISSAN_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/nissan/ecm.toml.gz"), "Nissan ECM"));
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/nissan/tcm.toml.gz"), "Nissan TCM"));
    map
});

/// Lazy-loaded Mercedes ECU data
pub static MERCEDES_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/mercedes/ecm.toml.gz"), "Mercedes ECM"));
    map
});

/// Lazy-loaded Hyundai ECU data
pub static HYUNDAI_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/hyundai/ecm.toml.gz"), "Hyundai ECM"));
    map
});

/// Lazy-loaded Kia ECU data
pub static KIA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/kia/ecm.toml.gz"), "Kia ECM"));
    map
});

/// Lazy-loaded Mazda ECU data
pub static MAZDA_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/mazda/ecm.toml.gz"), "Mazda ECM"));
    map
});

/// Lazy-loaded Subaru ECU data
pub static SUBARU_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/subaru/ecm.toml.gz"), "Subaru ECM"));
    map
});

/// Lazy-loaded Jeep ECU data
pub static JEEP_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/jeep/ecm.toml.gz"), "Jeep ECM"));
    map
});

/// Lazy-loaded Dodge/RAM ECU data
pub static DODGE_RAM_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    let mut map = HashMap::new();
    insert_ecus(&mut map, load_gz_ecus(include_bytes!("../data/manufacturers/dodge_ram/ecm.toml.gz"), "Dodge/RAM ECM"));
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
        "honda" => Some(&HONDA_ECUS),
        "nissan" => Some(&NISSAN_ECUS),
        "mercedes" => Some(&MERCEDES_ECUS),
        "hyundai" => Some(&HYUNDAI_ECUS),
        "kia" => Some(&KIA_ECUS),
        "mazda" => Some(&MAZDA_ECUS),
        "subaru" => Some(&SUBARU_ECUS),
        "jeep" => Some(&JEEP_ECUS),
        "dodge_ram" => Some(&DODGE_RAM_ECUS),
        _ => None,
    }
}

/// List available manufacturers
pub fn list_manufacturers() -> Vec<&'static str> {
    vec![
        "vw", "audi", "gm", "ford", "toyota", "bmw", "skoda",
        "honda", "nissan", "mercedes", "hyundai", "kia",
        "mazda", "subaru", "jeep", "dodge_ram",
    ]
}

/// Get total ECU count across all manufacturers
pub fn total_ecu_count() -> usize {
    list_manufacturers()
        .iter()
        .filter_map(|m| load_manufacturer_ecus(m))
        .map(|ecus| ecus.len())
        .sum()
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
        assert!(manufacturers.contains(&"honda"));
        assert!(manufacturers.contains(&"nissan"));
        assert!(manufacturers.contains(&"mercedes"));
        assert!(manufacturers.contains(&"hyundai"));
        assert!(manufacturers.contains(&"kia"));
        assert!(manufacturers.contains(&"mazda"));
        assert!(manufacturers.contains(&"subaru"));
        assert!(manufacturers.contains(&"jeep"));
        assert!(manufacturers.contains(&"dodge_ram"));
        assert_eq!(manufacturers.len(), 16, "Should have 16 manufacturers");
    }

    #[test]
    fn test_load_manufacturer_ecus() {
        let vw_ecus = load_manufacturer_ecus("vw");
        assert!(vw_ecus.is_some());

        let invalid = load_manufacturer_ecus("invalid");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_all_manufacturers_load_under_10ms() {
        use std::time::Instant;
        for mfr in list_manufacturers() {
            let start = Instant::now();
            let ecus = load_manufacturer_ecus(mfr);
            let duration = start.elapsed();
            assert!(ecus.is_some(), "Manufacturer {} should have ECU data", mfr);
            assert!(!ecus.unwrap().is_empty(), "{} should have at least one ECU", mfr);
            println!("{}: {}ms ({} ECUs)", mfr, duration.as_millis(), ecus.unwrap().len());
        }
    }

    #[test]
    fn test_vw_ecus_content() {
        let ecus = &*VW_ECUS;
        // VW has ECM + TCM + 12 additional = 14 total
        assert!(ecus.len() >= 14, "Expected at least 14 VW ECUs, got {}", ecus.len());

        for (id, ecu) in ecus.iter() {
            assert_eq!(id, &ecu.id);
            assert!(!ecu.connectors.is_empty(), "ECU {} should have connectors", id);
        }
    }

    #[test]
    fn test_total_ecu_count() {
        let total = total_ecu_count();
        assert!(total >= 95, "Expected at least 95 total ECUs, got {}", total);
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

    #[test]
    fn test_new_manufacturer_honda() {
        let ecus = &*HONDA_ECUS;
        assert!(ecus.len() >= 10, "Honda should have at least 10 ECUs, got {}", ecus.len());
    }

    #[test]
    fn test_new_manufacturer_nissan() {
        let ecus = &*NISSAN_ECUS;
        assert!(ecus.len() >= 10, "Nissan should have at least 10 ECUs, got {}", ecus.len());
    }

    #[test]
    fn test_new_manufacturer_mercedes() {
        let ecus = &*MERCEDES_ECUS;
        assert!(ecus.len() >= 8, "Mercedes should have at least 8 ECUs, got {}", ecus.len());
    }
}

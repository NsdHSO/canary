use canary_models::embedded::*;

#[test]
fn test_complete_ecu_pinout() {
    // Create vehicle model
    let vehicle = VehicleModel {
        manufacturer: "Volkswagen".to_string(),
        model: "Golf Mk7".to_string(),
        years: vec![2015, 2016, 2017],
        engine: Some("2.0L TDI".to_string()),
    };

    // Create pin mappings
    let pins = vec![
        EcuPinMapping {
            pin_number: 1,
            signal_name: "CAN_H".to_string(),
            wire_color: Some("White/Blue".to_string()),
            voltage: Some(3.0),
            current_max: None,
            signal_type: SignalType::CAN,
            protocol: Some("CAN 2.0B".to_string()),
            notes: Some("High-speed CAN bus".to_string()),
            sensor_actuator: None,
        },
        EcuPinMapping {
            pin_number: 2,
            signal_name: "CAN_L".to_string(),
            wire_color: Some("White/Green".to_string()),
            voltage: Some(3.0),
            current_max: None,
            signal_type: SignalType::CAN,
            protocol: Some("CAN 2.0B".to_string()),
            notes: Some("High-speed CAN bus".to_string()),
            sensor_actuator: None,
        },
    ];

    // Create connector
    let connector = ConnectorSpec {
        connector_id: "C1".to_string(),
        connector_type: "121-pin ECU".to_string(),
        color: Some("Black".to_string()),
        pins,
        standard_connector_id: None,
    };

    // Create power spec
    let power = PowerSpec {
        voltage_min: 9.0,
        voltage_nominal: 12.0,
        voltage_max: 16.0,
        current_max: Some(15.0),
        power_max: Some(180.0),
        fuse_rating: Some(20.0),
    };

    // Create memory spec
    let memory = MemorySpec {
        flash_size_kb: 2048,
        ram_size_kb: 64,
        eeprom_size_kb: Some(16),
        cpu: "Infineon TC1798".to_string(),
    };

    // Create complete ECU pinout
    let ecu = EcuPinout {
        id: "vw_golf_mk7_ecm".to_string(),
        name: "Golf Mk7 Engine Control Module".to_string(),
        module_type: ModuleType::ECM,
        manufacturer_id: "volkswagen".to_string(),
        ecu_manufacturer: "Bosch".to_string(),
        vehicle_models: vec![vehicle],
        connectors: vec![connector],
        power_requirements: power,
        flash_memory: Some(memory),
        supported_protocols: vec!["CAN 2.0B".to_string(), "K-Line".to_string()],
        part_numbers: vec!["03L906018JJ".to_string()],
    };

    // Verify structure
    assert_eq!(ecu.id, "vw_golf_mk7_ecm");
    assert_eq!(ecu.module_type, ModuleType::ECM);
    assert_eq!(ecu.connectors.len(), 1);
    assert_eq!(ecu.connectors[0].pins.len(), 2);
    assert_eq!(ecu.supported_protocols.len(), 2);
}

#[test]
fn test_universal_connector_integration() {
    let obd2_pins = vec![
        StandardPinMapping {
            pin_number: 4,
            signal_name: "CHASSIS_GND".to_string(),
            description: "Chassis ground".to_string(),
            mandatory: true,
        },
        StandardPinMapping {
            pin_number: 6,
            signal_name: "CAN_H".to_string(),
            description: "CAN High (ISO 15765-4)".to_string(),
            mandatory: false,
        },
        StandardPinMapping {
            pin_number: 14,
            signal_name: "CAN_L".to_string(),
            description: "CAN Low (ISO 15765-4)".to_string(),
            mandatory: false,
        },
    ];

    let obd2 = UniversalConnector {
        standard_id: "J1962".to_string(),
        name: "OBD-II 16-pin".to_string(),
        connector_type: "D-shaped".to_string(),
        pin_count: 16,
        standard_pins: obd2_pins,
    };

    assert_eq!(obd2.standard_id, "J1962");
    assert_eq!(obd2.pin_count, 16);
    assert_eq!(obd2.standard_pins.len(), 3);
}

#[test]
fn test_data_metadata_integration() {
    let metadata = DataMetadata {
        source: DataSource::MultiSourceVerified,
        confidence: 0.98,
        date_added: "2026-03-26".to_string(),
        last_verified: Some("2026-03-26".to_string()),
        last_modified: Some("2026-03-26".to_string()),
        version: "1.0.0".to_string(),
        contributors: vec![
            "engineer1".to_string(),
            "engineer2".to_string(),
        ],
        license: "CC-BY-SA-4.0".to_string(),
        references: vec![
            "VW Service Manual".to_string(),
            "Physical ECU Verification".to_string(),
        ],
        notes: Some("Cross-referenced with official VW documentation and physical ECU".to_string()),
    };

    assert!(metadata.confidence > 0.9);
    assert_eq!(metadata.contributors.len(), 2);
    assert_eq!(metadata.references.len(), 2);
}

#[test]
fn test_ecu_pinout_toml_serialization() {
    let vehicle = VehicleModel {
        manufacturer: "Test".to_string(),
        model: "Model".to_string(),
        years: vec![2020],
        engine: None,
    };

    let power = PowerSpec {
        voltage_min: 9.0,
        voltage_nominal: 12.0,
        voltage_max: 16.0,
        current_max: None,
        power_max: None,
        fuse_rating: None,
    };

    let ecu = EcuPinout {
        id: "test_ecu".to_string(),
        name: "Test ECU".to_string(),
        module_type: ModuleType::ECM,
        manufacturer_id: "test".to_string(),
        ecu_manufacturer: "Test Manufacturer".to_string(),
        vehicle_models: vec![vehicle],
        connectors: vec![],
        power_requirements: power,
        flash_memory: None,
        supported_protocols: vec![],
        part_numbers: vec![],
    };

    // Serialize to TOML
    let toml_string = toml::to_string(&ecu).expect("Failed to serialize to TOML");
    assert!(toml_string.contains("id = \"test_ecu\""));
    assert!(toml_string.contains("name = \"Test ECU\""));

    // Deserialize from TOML
    let deserialized: EcuPinout = toml::from_str(&toml_string).expect("Failed to deserialize from TOML");
    assert_eq!(deserialized.id, ecu.id);
    assert_eq!(deserialized.module_type, ecu.module_type);
}

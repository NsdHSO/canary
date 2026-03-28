use serde::{Deserialize, Serialize};

/// Type of ECU module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    /// Engine Control Module
    ECM,
    /// Powertrain Control Module
    PCM,
    /// Transmission Control Module
    TCM,
    /// Body Control Module
    BCM,
    /// Driver Door Module
    DDM,
    /// Passenger Door Module
    PDM,
    /// HVAC Control Module
    HVAC,
    /// Anti-lock Braking System
    ABS,
    /// Supplemental Restraint System (Airbag)
    SRS,
    /// Electronic Parking Brake
    EPB,
    /// Instrument Panel Cluster
    IPC,
    /// Infotainment Center
    InfoCenter,
    /// Gateway Module
    Gateway,
    /// Telematics Control Unit
    Telematics,
    /// On-Board Diagnostics
    OBD,
}

/// Type of electrical signal on a pin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Power supply
    Power,
    /// Ground reference
    Ground,
    /// Digital signal (0/1)
    Digital,
    /// Analog signal (variable voltage)
    Analog,
    /// Pulse Width Modulation
    PWM,
    /// CAN bus signal
    CAN,
    /// LIN bus signal
    LIN,
    /// FlexRay bus signal
    FlexRay,
    /// Sensor input
    SensorInput,
    /// Actuator output
    ActuatorOutput,
}

/// Mapping of a single pin in a connector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinMapping {
    /// Pin number in connector
    pub pin_number: u8,
    /// Name of the signal on this pin
    pub signal_name: String,
    /// Wire color (manufacturer-specific)
    pub wire_color: Option<String>,
    /// Voltage level in volts
    pub voltage: Option<f32>,
    /// Maximum current in amperes
    pub current_max: Option<f32>,
    /// Type of signal
    pub signal_type: SignalType,
    /// Protocol used (if applicable)
    pub protocol: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Connected sensor or actuator
    pub sensor_actuator: Option<String>,
}

/// Specification of a physical connector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorSpec {
    /// Connector identifier (e.g., "C1", "C2")
    pub connector_id: String,
    /// Type of connector (e.g., "16-pin OBD-II", "121-pin ECU")
    pub connector_type: String,
    /// Connector color (if known)
    pub color: Option<String>,
    /// Pin mappings for this connector
    pub pins: Vec<PinMapping>,
    /// Standard connector ID (e.g., "J1962" for OBD-II)
    pub standard_connector_id: Option<String>,
}

/// Power specifications for an ECU
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerSpec {
    /// Minimum operating voltage in volts
    pub voltage_min: f32,
    /// Nominal operating voltage in volts
    pub voltage_nominal: f32,
    /// Maximum operating voltage in volts
    pub voltage_max: f32,
    /// Maximum current draw in amperes
    pub current_max: Option<f32>,
    /// Maximum power consumption in watts
    pub power_max: Option<f32>,
    /// Fuse rating in amperes
    pub fuse_rating: Option<f32>,
}

/// Memory specifications for an ECU
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySpec {
    /// Flash memory size in kilobytes
    pub flash_size_kb: u32,
    /// RAM size in kilobytes
    pub ram_size_kb: u32,
    /// EEPROM size in kilobytes (optional)
    pub eeprom_size_kb: Option<u32>,
    /// CPU/Processor model
    pub cpu: String,
}

/// Complete ECU pinout specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuPinout {
    /// Unique identifier (e.g., "vw_golf_mk7_ecm")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of ECU module
    pub module_type: ModuleType,
    /// Manufacturer ID
    pub manufacturer_id: String,
    /// ECU manufacturer (may differ from vehicle manufacturer)
    pub ecu_manufacturer: String,
    /// Compatible vehicle models
    pub vehicle_models: Vec<crate::embedded::manufacturer::VehicleModel>,
    /// Physical connectors
    pub connectors: Vec<ConnectorSpec>,
    /// Power requirements
    pub power_requirements: PowerSpec,
    /// Flash memory specifications
    pub flash_memory: Option<MemorySpec>,
    /// Supported communication protocols
    pub supported_protocols: Vec<String>,
    /// Manufacturer part numbers
    pub part_numbers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_type_variants() {
        let variants = vec![
            ModuleType::ECM,
            ModuleType::PCM,
            ModuleType::TCM,
            ModuleType::BCM,
            ModuleType::DDM,
            ModuleType::PDM,
            ModuleType::HVAC,
            ModuleType::ABS,
            ModuleType::SRS,
            ModuleType::EPB,
            ModuleType::IPC,
            ModuleType::InfoCenter,
            ModuleType::Gateway,
            ModuleType::Telematics,
            ModuleType::OBD,
        ];
        assert_eq!(variants.len(), 15);
    }

    #[test]
    fn test_signal_type_variants() {
        let variants = vec![
            SignalType::Power,
            SignalType::Ground,
            SignalType::Digital,
            SignalType::Analog,
            SignalType::PWM,
            SignalType::CAN,
            SignalType::LIN,
            SignalType::FlexRay,
            SignalType::SensorInput,
            SignalType::ActuatorOutput,
        ];
        assert_eq!(variants.len(), 10);
    }

    #[test]
    fn test_pin_mapping_creation() {
        let pin = PinMapping {
            pin_number: 1,
            signal_name: "CAN_H".to_string(),
            wire_color: Some("White/Blue".to_string()),
            voltage: Some(12.0),
            current_max: Some(0.1),
            signal_type: SignalType::CAN,
            protocol: Some("CAN 2.0B".to_string()),
            notes: Some("High-speed CAN".to_string()),
            sensor_actuator: None,
        };

        assert_eq!(pin.pin_number, 1);
        assert_eq!(pin.signal_name, "CAN_H");
        assert_eq!(pin.signal_type, SignalType::CAN);
        assert_eq!(pin.voltage, Some(12.0));
        assert_eq!(pin.current_max, Some(0.1));
    }

    #[test]
    fn test_connector_spec_creation() {
        let pins = vec![
            PinMapping {
                pin_number: 1,
                signal_name: "CAN_H".to_string(),
                wire_color: None,
                voltage: Some(12.0),
                current_max: None,
                signal_type: SignalType::CAN,
                protocol: Some("CAN 2.0B".to_string()),
                notes: None,
                sensor_actuator: None,
            },
        ];

        let connector = ConnectorSpec {
            connector_id: "C1".to_string(),
            connector_type: "16-pin OBD-II".to_string(),
            color: Some("Black".to_string()),
            pins,
            standard_connector_id: Some("J1962".to_string()),
        };

        assert_eq!(connector.connector_id, "C1");
        assert_eq!(connector.pins.len(), 1);
    }

    #[test]
    fn test_power_spec_creation() {
        let power = PowerSpec {
            voltage_min: 9.0,
            voltage_nominal: 12.0,
            voltage_max: 16.0,
            current_max: Some(15.0),
            power_max: Some(180.0),
            fuse_rating: Some(20.0),
        };

        assert_eq!(power.voltage_nominal, 12.0);
        assert_eq!(power.voltage_min, 9.0);
        assert_eq!(power.voltage_max, 16.0);
        assert_eq!(power.fuse_rating, Some(20.0));
    }

    #[test]
    fn test_ecu_pinout_creation() {
        use crate::embedded::manufacturer::VehicleModel;

        let vehicle = VehicleModel {
            manufacturer: "Volkswagen".to_string(),
            model: "Golf Mk7".to_string(),
            years: vec![2015, 2016, 2017],
            engine: Some("2.0L TDI".to_string()),
        };

        let power = PowerSpec {
            voltage_min: 9.0,
            voltage_nominal: 12.0,
            voltage_max: 16.0,
            current_max: Some(15.0),
            power_max: None,
            fuse_rating: Some(20.0),
        };

        let ecu = EcuPinout {
            id: "vw_golf_mk7_ecm".to_string(),
            name: "Golf Mk7 Engine Control Module".to_string(),
            module_type: ModuleType::ECM,
            manufacturer_id: "volkswagen".to_string(),
            ecu_manufacturer: "Bosch".to_string(),
            vehicle_models: vec![vehicle],
            connectors: vec![],
            power_requirements: power,
            flash_memory: None,
            supported_protocols: vec!["CAN 2.0B".to_string()],
            part_numbers: vec!["03L906018JJ".to_string()],
        };

        assert_eq!(ecu.id, "vw_golf_mk7_ecm");
        assert_eq!(ecu.module_type, ModuleType::ECM);
        assert_eq!(ecu.manufacturer_id, "volkswagen");
        assert_eq!(ecu.ecu_manufacturer, "Bosch");
    }
}

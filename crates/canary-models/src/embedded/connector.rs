use serde::{Deserialize, Serialize};

/// Gender/type of connector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectorGender {
    /// Male connector (pins)
    Male,
    /// Female connector (sockets)
    Female,
    /// Hermaphroditic/genderless connector
    Hermaphroditic,
}

/// Mounting type for connector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MountingType {
    /// Panel mount connector
    PanelMount,
    /// Cable mount connector
    CableMount,
    /// PCB mount connector
    PCBMount,
    /// Chassis mount connector
    ChassisMount,
}

/// Complete connector definition with physical specifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorDefinition {
    /// Unique identifier
    pub id: String,
    /// Standard identifier (e.g., "J1962", "J1979")
    pub standard_id: Option<String>,
    /// Common name (e.g., "OBD-II 16-pin")
    pub name: String,
    /// Manufacturer part number
    pub part_number: Option<String>,
    /// Number of pins
    pub pin_count: u8,
    /// Connector gender
    pub gender: ConnectorGender,
    /// Mounting type
    pub mounting_type: MountingType,
    /// Physical dimensions (length x width x height in mm)
    pub dimensions_mm: Option<(f32, f32, f32)>,
    /// Keying/orientation features
    pub keying: Option<String>,
    /// Locking mechanism description
    pub locking_mechanism: Option<String>,
    /// Standard pin mappings
    pub standard_pins: Vec<StandardPinMapping>,
    /// Notes about connector
    pub notes: Option<String>,
}

/// Universal connector type (e.g., OBD-II J1962)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniversalConnector {
    /// Standard identifier (e.g., "J1962", "J1979")
    pub standard_id: String,
    /// Common name (e.g., "OBD-II 16-pin")
    pub name: String,
    /// Physical connector type
    pub connector_type: String,
    /// Pin count
    pub pin_count: u8,
    /// Standard pin mappings
    pub standard_pins: Vec<StandardPinMapping>,
}

/// Standard pin mapping for universal connectors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardPinMapping {
    /// Pin number
    pub pin_number: u8,
    /// Standard signal name
    pub signal_name: String,
    /// Signal description
    pub description: String,
    /// Whether this pin is mandatory in the standard
    pub mandatory: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_connector_creation() {
        let connector = UniversalConnector {
            standard_id: "J1962".to_string(),
            name: "OBD-II 16-pin".to_string(),
            connector_type: "D-shaped".to_string(),
            pin_count: 16,
            standard_pins: vec![],
        };

        assert_eq!(connector.standard_id, "J1962");
        assert_eq!(connector.pin_count, 16);
    }

    #[test]
    fn test_standard_pin_mapping() {
        let pin = StandardPinMapping {
            pin_number: 6,
            signal_name: "CAN_H".to_string(),
            description: "CAN High (ISO 15765-4)".to_string(),
            mandatory: true,
        };

        assert_eq!(pin.pin_number, 6);
        assert!(pin.mandatory);
    }

    #[test]
    fn test_connector_gender_variants() {
        let genders = vec![
            ConnectorGender::Male,
            ConnectorGender::Female,
            ConnectorGender::Hermaphroditic,
        ];
        assert_eq!(genders.len(), 3);
    }

    #[test]
    fn test_mounting_type_variants() {
        let types = vec![
            MountingType::PanelMount,
            MountingType::CableMount,
            MountingType::PCBMount,
            MountingType::ChassisMount,
        ];
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_connector_definition_creation() {
        let connector = ConnectorDefinition {
            id: "obd2_j1962".to_string(),
            standard_id: Some("J1962".to_string()),
            name: "OBD-II Diagnostic Connector".to_string(),
            part_number: None,
            pin_count: 16,
            gender: ConnectorGender::Female,
            mounting_type: MountingType::PanelMount,
            dimensions_mm: Some((48.0, 30.0, 15.0)),
            keying: Some("D-shaped shell".to_string()),
            locking_mechanism: Some("Friction fit".to_string()),
            standard_pins: vec![],
            notes: Some("SAE J1962 standard connector".to_string()),
        };

        assert_eq!(connector.id, "obd2_j1962");
        assert_eq!(connector.pin_count, 16);
        assert_eq!(connector.gender, ConnectorGender::Female);
        assert_eq!(connector.mounting_type, MountingType::PanelMount);
    }
}

use crate::ecu_simulator::{Dtc, EcuSimulator};
use std::collections::HashMap;

/// VW Golf ECM (Engine Control Module) Simulator
pub struct VwGolfEcm {
    ecu: EcuSimulator,
}

impl VwGolfEcm {
    /// Create a new VW Golf ECM simulator
    pub fn new() -> Self {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Load VW Golf-specific DTCs
        ecu.dtc_database = Self::load_vw_golf_dtcs();

        // Load VW Golf-specific live data
        ecu.live_data = Self::simulate_engine_running();

        Self { ecu }
    }

    /// Handle UDS request (delegates to base ECU)
    pub fn handle_uds_request(&mut self, request: &[u8]) -> Vec<u8> {
        self.ecu.handle_uds_request(request)
    }

    /// Get CAN ID for this ECU
    pub fn can_id(&self) -> u32 {
        self.ecu.can_id
    }

    /// Load VW Golf-specific DTCs
    fn load_vw_golf_dtcs() -> HashMap<String, Dtc> {
        HashMap::from([
            (
                "P0301".to_string(),
                Dtc {
                    code: "P0301".to_string(),
                    status: 0x08, // Confirmed, MIL on
                    description: "Cylinder 1 Misfire Detected".to_string(),
                },
            ),
            (
                "P0420".to_string(),
                Dtc {
                    code: "P0420".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Catalyst System Efficiency Below Threshold".to_string(),
                },
            ),
            (
                "P0171".to_string(),
                Dtc {
                    code: "P0171".to_string(),
                    status: 0x04, // Pending
                    description: "System Too Lean (Bank 1)".to_string(),
                },
            ),
            (
                "P0506".to_string(),
                Dtc {
                    code: "P0506".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Idle Control System RPM Lower Than Expected".to_string(),
                },
            ),
        ])
    }

    /// Simulate engine running with typical VW Golf parameters
    fn simulate_engine_running() -> HashMap<u16, Vec<u8>> {
        HashMap::from([
            // VIN (Vehicle Identification Number) - 0xF190
            (
                0xF190,
                b"WVWZZZ1KZBW123456".to_vec(),
            ),
            // Engine RPM - 0x0C (PID) - 850 RPM (idle)
            (0x0C, vec![0x03, 0x54]), // (850 * 4) = 3400 -> 0x0D54
            // Vehicle Speed - 0x0D - 0 km/h
            (0x0D, vec![0x00]),
            // Coolant Temperature - 0x05 - 90°C
            (0x05, vec![0xDA]), // 90 + 40 = 130 = 0x82
            // Throttle Position - 0x11 - 5%
            (0x11, vec![0x0D]), // (5 / 100) * 255 = 12.75 ≈ 13
            // Engine Load - 0x04 - 15%
            (0x04, vec![0x26]), // (15 / 100) * 255 = 38.25 ≈ 38
            // MAF (Mass Air Flow) - 0x10 - 2.5 g/s
            (0x10, vec![0x00, 0xFA]), // 2.5 * 100 = 250 = 0x00FA
            // Intake Air Temperature - 0x0F - 25°C
            (0x0F, vec![0x41]), // 25 + 40 = 65 = 0x41
            // Fuel System Status - 0x03 - Closed loop
            (0x03, vec![0x02]),
            // Calculated Engine Load - 0x04
            (0x04, vec![0x26]),
            // Short Term Fuel Trim Bank 1 - 0x06 - 0%
            (0x06, vec![0x80]), // 0% = 128 = 0x80
            // Long Term Fuel Trim Bank 1 - 0x07 - +2%
            (0x07, vec![0x83]), // +2% = 130 = 0x82
            // Fuel Pressure - 0x0A - 300 kPa
            (0x0A, vec![0x64]), // 300 / 3 = 100 = 0x64
            // Timing Advance - 0x0E - 10 degrees
            (0x0E, vec![0x54]), // (10 + 64) * 2 = 148 = 0x94 / 2 = 74 = 0x4A
        ])
    }
}

impl Default for VwGolfEcm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vw_golf_creation() {
        let ecu = VwGolfEcm::new();
        assert_eq!(ecu.can_id(), 0x7E0);
    }

    #[test]
    fn test_vw_golf_dtcs() {
        let mut ecu = VwGolfEcm::new();

        // Read DTCs with status mask 0xFF
        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59); // Positive response
        assert!(response.len() > 3); // Should have DTCs
    }

    #[test]
    fn test_vw_golf_read_vin() {
        let mut ecu = VwGolfEcm::new();

        // Read VIN (0xF190)
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response[1..3], [0xF1, 0x90]); // DID echo

        // Check VIN starts with WVWZZZ (VW prefix)
        let vin = &response[3..];
        assert!(vin.starts_with(b"WVWZZZ"));
    }

    #[test]
    fn test_vw_golf_read_engine_rpm() {
        let mut ecu = VwGolfEcm::new();

        // Read Engine RPM (0x000C)
        let response = ecu.handle_uds_request(&[0x22, 0x00, 0x0C]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response.len(), 5); // Header + DID + 2 bytes data
    }

    #[test]
    fn test_vw_golf_session_control() {
        let mut ecu = VwGolfEcm::new();

        // Switch to extended session
        let response = ecu.handle_uds_request(&[0x10, 0x03]);
        assert_eq!(response[0], 0x50); // Positive response
        assert_eq!(response[1], 0x03); // Extended session
    }
}

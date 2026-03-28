use crate::ecu_simulator::{Dtc, EcuSimulator};
use std::collections::HashMap;

/// Ford F-150 PCM (Powertrain Control Module) Simulator
pub struct FordF150Pcm {
    ecu: EcuSimulator,
}

impl FordF150Pcm {
    /// Create a new Ford F-150 PCM simulator
    pub fn new() -> Self {
        let mut ecu = EcuSimulator::new(0x7E2);

        // Load Ford F-150-specific DTCs
        ecu.dtc_database = Self::load_ford_f150_dtcs();

        // Load Ford F-150-specific live data
        ecu.live_data = Self::simulate_ecoboost_engine_running();

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

    /// Load Ford F-150-specific DTCs
    fn load_ford_f150_dtcs() -> HashMap<String, Dtc> {
        HashMap::from([
            (
                "P0234".to_string(),
                Dtc {
                    code: "P0234".to_string(),
                    status: 0x08, // Confirmed, MIL on
                    description: "Turbo/Supercharger Overboost Condition".to_string(),
                },
            ),
            (
                "P0299".to_string(),
                Dtc {
                    code: "P0299".to_string(),
                    status: 0x04, // Pending
                    description: "Turbocharger Underboost".to_string(),
                },
            ),
            (
                "P0401".to_string(),
                Dtc {
                    code: "P0401".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Exhaust Gas Recirculation Flow Insufficient".to_string(),
                },
            ),
            (
                "P0562".to_string(),
                Dtc {
                    code: "P0562".to_string(),
                    status: 0x00, // Stored, not active
                    description: "System Voltage Low".to_string(),
                },
            ),
            (
                "P2263".to_string(),
                Dtc {
                    code: "P2263".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Turbo/Supercharger Boost System Performance".to_string(),
                },
            ),
            (
                "P0193".to_string(),
                Dtc {
                    code: "P0193".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Fuel Rail Pressure Sensor Circuit High".to_string(),
                },
            ),
        ])
    }

    /// Simulate EcoBoost V6 turbo engine running with typical Ford F-150 parameters
    fn simulate_ecoboost_engine_running() -> HashMap<u16, Vec<u8>> {
        HashMap::from([
            // VIN (Vehicle Identification Number) - 0xF190
            (
                0xF190,
                b"1FTFW1E85MFA12345".to_vec(), // Ford VIN format
            ),
            // Engine RPM - 0x0C - 750 RPM (idle)
            (0x0C, vec![0x02, 0xEE]), // (750 * 4) = 3000 -> 0x0BB8
            // Vehicle Speed - 0x0D - 0 km/h
            (0x0D, vec![0x00]),
            // Coolant Temperature - 0x05 - 92°C
            (0x05, vec![0x84]), // 92 + 40 = 132 = 0x84
            // Throttle Position - 0x11 - 6%
            (0x11, vec![0x0F]), // (6 / 100) * 255 = 15.3 ≈ 15
            // Engine Load - 0x04 - 18%
            (0x04, vec![0x2E]), // (18 / 100) * 255 = 45.9 ≈ 46
            // MAF (Mass Air Flow) - 0x10 - 5.2 g/s (turbo engine)
            (0x10, vec![0x02, 0x08]), // 5.2 * 100 = 520 = 0x0208
            // Intake Air Temperature - 0x0F - 28°C
            (0x0F, vec![0x44]), // 28 + 40 = 68 = 0x44
            // Fuel System Status - 0x03 - Closed loop
            (0x03, vec![0x02]),
            // Calculated Engine Load - 0x04
            (0x04, vec![0x2E]),
            // Short Term Fuel Trim Bank 1 - 0x06 - -1%
            (0x06, vec![0x7F]),
            // Long Term Fuel Trim Bank 1 - 0x07 - +1%
            (0x07, vec![0x81]),
            // Short Term Fuel Trim Bank 2 - 0x08 - 0%
            (0x08, vec![0x80]),
            // Long Term Fuel Trim Bank 2 - 0x09 - +1%
            (0x09, vec![0x81]),
            // Fuel Pressure - 0x0A - 400 kPa (higher for turbo)
            (0x0A, vec![0x85]), // 400 / 3 = 133.33 ≈ 133
            // Intake Manifold Pressure - 0x0B - 35 kPa (slightly boosted at idle)
            (0x0B, vec![0x23]),
            // Timing Advance - 0x0E - 12 degrees
            (0x0E, vec![0x4C]),
            // O2 Sensor Voltage Bank 1 Sensor 1 - 0x14
            (0x14, vec![0x80, 0xFF]),
            // O2 Sensor Voltage Bank 1 Sensor 2 - 0x15
            (0x15, vec![0x7E, 0xFF]),
            // O2 Sensor Voltage Bank 2 Sensor 1 - 0x16
            (0x16, vec![0x81, 0xFF]),
            // O2 Sensor Voltage Bank 2 Sensor 2 - 0x17
            (0x17, vec![0x7F, 0xFF]),
            // Boost Pressure (custom PID for turbo) - 0xF1A0
            (0xF1A0, vec![0x00, 0x64]), // 100 mbar = 1 bar (no boost at idle)
            // Turbo RPM (custom PID) - 0xF1A1
            (0xF1A1, vec![0x4E, 0x20]), // 20000 RPM idle
        ])
    }
}

impl Default for FordF150Pcm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ford_f150_creation() {
        let ecu = FordF150Pcm::new();
        assert_eq!(ecu.can_id(), 0x7E2);
    }

    #[test]
    fn test_ford_f150_dtcs() {
        let mut ecu = FordF150Pcm::new();

        // Read DTCs with status mask 0xFF
        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59); // Positive response
        assert!(response.len() > 3); // Should have DTCs
    }

    #[test]
    fn test_ford_f150_turbo_dtcs() {
        let mut ecu = FordF150Pcm::new();

        // Read DTCs - should include turbo-specific codes
        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59);

        // Should have DTCs (at least the response header)
        // Each DTC is 3 bytes (2 for code + 1 for status)
        // Response: [0x59, 0x02, availability_mask, ...DTCs...]
        assert!(response.len() > 3); // Has DTCs beyond header
    }

    #[test]
    fn test_ford_f150_read_vin() {
        let mut ecu = FordF150Pcm::new();

        // Read VIN (0xF190)
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response[1..3], [0xF1, 0x90]); // DID echo

        // Check VIN starts with 1FT (Ford truck USA prefix)
        let vin = &response[3..];
        assert!(vin.starts_with(b"1FT"));
    }

    #[test]
    fn test_ford_f150_read_boost_pressure() {
        let mut ecu = FordF150Pcm::new();

        // Read Boost Pressure (0xF1A0) - custom PID for turbo
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0xA0]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response.len(), 5); // Header + DID + 2 bytes data
    }

    #[test]
    fn test_ford_f150_read_turbo_rpm() {
        let mut ecu = FordF150Pcm::new();

        // Read Turbo RPM (0xF1A1) - custom PID
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0xA1]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response.len(), 5); // Header + DID + 2 bytes data

        // Turbo should be spinning even at idle
        let turbo_rpm = u16::from_be_bytes([response[3], response[4]]);
        assert!(turbo_rpm > 10000); // > 10k RPM
    }

    #[test]
    fn test_ford_f150_session_control() {
        let mut ecu = FordF150Pcm::new();

        // Switch to extended session
        let response = ecu.handle_uds_request(&[0x10, 0x03]);
        assert_eq!(response[0], 0x50); // Positive response
        assert_eq!(response[1], 0x03); // Extended session
    }

    #[test]
    fn test_ford_f150_security_access() {
        let mut ecu = FordF150Pcm::new();

        // Request seed
        let response = ecu.handle_uds_request(&[0x27, 0x01]);
        assert_eq!(response[0], 0x67); // Positive response
        assert_eq!(response.len(), 6); // Header + seed (4 bytes)

        // Send key
        let response = ecu.handle_uds_request(&[0x27, 0x02, 0xED, 0xCB, 0xA9, 0x87]);
        assert_eq!(response[0], 0x67); // Positive response
    }
}

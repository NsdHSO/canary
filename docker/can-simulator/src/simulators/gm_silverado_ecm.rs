use crate::ecu_simulator::{Dtc, EcuSimulator};
use std::collections::HashMap;

/// GM Silverado ECM (Engine Control Module) Simulator
pub struct GmSilveradoEcm {
    ecu: EcuSimulator,
}

impl GmSilveradoEcm {
    /// Create a new GM Silverado ECM simulator
    pub fn new() -> Self {
        let mut ecu = EcuSimulator::new(0x7E1);

        // Load GM Silverado-specific DTCs
        ecu.dtc_database = Self::load_gm_silverado_dtcs();

        // Load GM Silverado-specific live data
        ecu.live_data = Self::simulate_v8_engine_running();

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

    /// Load GM Silverado-specific DTCs
    fn load_gm_silverado_dtcs() -> HashMap<String, Dtc> {
        HashMap::from([
            (
                "P0128".to_string(),
                Dtc {
                    code: "P0128".to_string(),
                    status: 0x08, // Confirmed, MIL on
                    description: "Coolant Thermostat (Coolant Temperature Below Thermostat Regulating Temperature)".to_string(),
                },
            ),
            (
                "P0300".to_string(),
                Dtc {
                    code: "P0300".to_string(),
                    status: 0x04, // Pending
                    description: "Random/Multiple Cylinder Misfire Detected".to_string(),
                },
            ),
            (
                "P0455".to_string(),
                Dtc {
                    code: "P0455".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Evaporative Emission System Leak Detected (Large Leak)".to_string(),
                },
            ),
            (
                "P0101".to_string(),
                Dtc {
                    code: "P0101".to_string(),
                    status: 0x00, // Stored, not active
                    description: "Mass or Volume Air Flow Circuit Range/Performance Problem".to_string(),
                },
            ),
            (
                "P0606".to_string(),
                Dtc {
                    code: "P0606".to_string(),
                    status: 0x00, // Stored, not active
                    description: "PCM Processor Fault".to_string(),
                },
            ),
        ])
    }

    /// Simulate V8 engine running with typical GM Silverado parameters
    fn simulate_v8_engine_running() -> HashMap<u16, Vec<u8>> {
        HashMap::from([
            // VIN (Vehicle Identification Number) - 0xF190
            (
                0xF190,
                b"1GCUYDED0MZ123456".to_vec(), // GM VIN format
            ),
            // Engine RPM - 0x0C - 700 RPM (V8 idle)
            (0x0C, vec![0x02, 0xBC]), // (700 * 4) = 2800 -> 0x0AF0
            // Vehicle Speed - 0x0D - 0 km/h
            (0x0D, vec![0x00]),
            // Coolant Temperature - 0x05 - 95°C
            (0x05, vec![0x87]), // 95 + 40 = 135 = 0x87
            // Throttle Position - 0x11 - 8%
            (0x11, vec![0x14]), // (8 / 100) * 255 = 20.4 ≈ 20
            // Engine Load - 0x04 - 20%
            (0x04, vec![0x33]), // (20 / 100) * 255 = 51
            // MAF (Mass Air Flow) - 0x10 - 4.5 g/s (larger engine)
            (0x10, vec![0x01, 0xC2]), // 4.5 * 100 = 450 = 0x01C2
            // Intake Air Temperature - 0x0F - 30°C
            (0x0F, vec![0x46]), // 30 + 40 = 70 = 0x46
            // Fuel System Status - 0x03 - Closed loop
            (0x03, vec![0x02]),
            // Calculated Engine Load - 0x04
            (0x04, vec![0x33]),
            // Short Term Fuel Trim Bank 1 - 0x06 - +1%
            (0x06, vec![0x81]),
            // Long Term Fuel Trim Bank 1 - 0x07 - +3%
            (0x07, vec![0x84]),
            // Short Term Fuel Trim Bank 2 - 0x08 - 0%
            (0x08, vec![0x80]),
            // Long Term Fuel Trim Bank 2 - 0x09 - +2%
            (0x09, vec![0x83]),
            // Fuel Pressure - 0x0A - 380 kPa
            (0x0A, vec![0x7F]), // 380 / 3 = 126.67 ≈ 127
            // Intake Manifold Pressure - 0x0B - 30 kPa
            (0x0B, vec![0x1E]),
            // Timing Advance - 0x0E - 15 degrees
            (0x0E, vec![0x4F]),
            // O2 Sensor Voltage Bank 1 Sensor 1 - 0x14
            (0x14, vec![0x80, 0xFF]),
            // O2 Sensor Voltage Bank 1 Sensor 2 - 0x15
            (0x15, vec![0x7F, 0xFF]),
        ])
    }
}

impl Default for GmSilveradoEcm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gm_silverado_creation() {
        let ecu = GmSilveradoEcm::new();
        assert_eq!(ecu.can_id(), 0x7E1);
    }

    #[test]
    fn test_gm_silverado_dtcs() {
        let mut ecu = GmSilveradoEcm::new();

        // Read DTCs with status mask 0xFF
        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59); // Positive response
        assert!(response.len() > 3); // Should have DTCs
    }

    #[test]
    fn test_gm_silverado_read_vin() {
        let mut ecu = GmSilveradoEcm::new();

        // Read VIN (0xF190)
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response[1..3], [0xF1, 0x90]); // DID echo

        // Check VIN starts with 1G (GM USA prefix)
        let vin = &response[3..];
        assert!(vin.starts_with(b"1G"));
    }

    #[test]
    fn test_gm_silverado_read_maf() {
        let mut ecu = GmSilveradoEcm::new();

        // Read MAF (0x0010)
        let response = ecu.handle_uds_request(&[0x22, 0x00, 0x10]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response.len(), 5); // Header + DID + 2 bytes data

        // V8 should have higher MAF than 4-cylinder
        let maf_value = u16::from_be_bytes([response[3], response[4]]);
        assert!(maf_value > 200); // > 2.0 g/s
    }

    #[test]
    fn test_gm_silverado_session_control() {
        let mut ecu = GmSilveradoEcm::new();

        // Switch to programming session
        let response = ecu.handle_uds_request(&[0x10, 0x02]);
        assert_eq!(response[0], 0x50); // Positive response
        assert_eq!(response[1], 0x02); // Programming session
    }

    #[test]
    fn test_gm_silverado_fuel_trims() {
        let mut ecu = GmSilveradoEcm::new();

        // Read Short Term Fuel Trim Bank 1 (0x0006)
        let response = ecu.handle_uds_request(&[0x22, 0x00, 0x06]);
        assert_eq!(response[0], 0x62);

        // Read Short Term Fuel Trim Bank 2 (0x0008) - V8 has 2 banks
        let response = ecu.handle_uds_request(&[0x22, 0x00, 0x08]);
        assert_eq!(response[0], 0x62);
    }
}

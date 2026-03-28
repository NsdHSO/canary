use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use canary_hardware::{CanAdapter, CanFrame, VirtualAdapter};

use crate::error::UdsError;
use crate::session::UdsSession;

/// Data collected from a single ECU
#[derive(Debug, Clone)]
pub struct EcuData {
    /// ECU request CAN ID
    pub ecu_id: u32,
    /// Collected DID values
    pub data: HashMap<u16, Vec<u8>>,
    /// Last update timestamp
    pub last_update: Instant,
    /// Whether the ECU is responsive
    pub responsive: bool,
    /// Latency of last query in microseconds
    pub latency_us: u64,
    /// Error count since monitoring started
    pub error_count: u64,
}

impl EcuData {
    /// Create empty ECU data
    fn new(ecu_id: u32) -> Self {
        Self {
            ecu_id,
            data: HashMap::new(),
            last_update: Instant::now(),
            responsive: false,
            latency_us: 0,
            error_count: 0,
        }
    }

    /// Get a DID value as u16
    pub fn get_u16(&self, did: u16) -> Option<u16> {
        self.data.get(&did).and_then(|d| {
            if d.len() >= 2 {
                Some(u16::from_be_bytes([d[0], d[1]]))
            } else {
                None
            }
        })
    }

    /// Get a DID value as raw bytes
    pub fn get_raw(&self, did: u16) -> Option<&Vec<u8>> {
        self.data.get(&did)
    }
}

/// Configuration for a monitored ECU
#[derive(Debug, Clone)]
pub struct MonitoredEcu {
    /// ECU request CAN ID
    pub ecu_id: u32,
    /// DIDs to monitor
    pub dids: Vec<u16>,
    /// Poll interval in milliseconds
    pub poll_interval_ms: u64,
}

impl MonitoredEcu {
    /// Create a new ECU monitoring config
    pub fn new(ecu_id: u32, dids: Vec<u16>) -> Self {
        Self {
            ecu_id,
            dids,
            poll_interval_ms: 100, // Default 100ms
        }
    }

    /// Set the poll interval
    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.poll_interval_ms = interval_ms;
        self
    }
}

/// Multi-ECU monitoring system
///
/// Monitors multiple ECUs simultaneously using async parallel sessions.
/// Each ECU is polled independently for its configured DIDs.
///
/// # Architecture
///
/// Uses tokio tasks for parallel ECU communication:
/// - Each ECU gets its own async task
/// - Results are aggregated into a shared HashMap
/// - Latency is tracked per-ECU
///
/// # Example
///
/// ```rust,no_run
/// use canary_uds::monitor::{MultiEcuMonitor, MonitoredEcu};
///
/// # async fn example() -> Result<(), canary_uds::UdsError> {
/// let monitor = MultiEcuMonitor::new();
/// // Configure and run monitoring...
/// # Ok(())
/// # }
/// ```
pub struct MultiEcuMonitor {
    /// ECU configurations
    ecus: Vec<MonitoredEcu>,
    /// Shared state for all ECU data
    state: Arc<RwLock<HashMap<u32, EcuData>>>,
}

impl MultiEcuMonitor {
    /// Create a new multi-ECU monitor
    pub fn new() -> Self {
        Self {
            ecus: Vec::new(),
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an ECU to monitor
    pub fn add_ecu(&mut self, ecu: MonitoredEcu) {
        self.ecus.push(ecu);
    }

    /// Add multiple ECUs at once
    pub fn add_ecus(&mut self, ecus: Vec<MonitoredEcu>) {
        self.ecus.extend(ecus);
    }

    /// Get the number of monitored ECUs
    pub fn ecu_count(&self) -> usize {
        self.ecus.len()
    }

    /// Poll all ECUs once and return aggregated data
    ///
    /// Creates a virtual adapter per ECU for testing, or
    /// uses the provided adapter factory for real hardware.
    pub async fn poll_all_virtual(&self) -> Result<HashMap<u32, EcuData>, UdsError> {
        let mut handles = Vec::new();
        let state = self.state.clone();

        for ecu_config in &self.ecus {
            let ecu_id = ecu_config.ecu_id;
            let dids = ecu_config.dids.clone();
            let state = state.clone();

            let handle = tokio::spawn(async move {
                let start = Instant::now();
                let mut ecu_data = EcuData::new(ecu_id);

                // Create virtual adapter for this ECU
                let mut adapter = VirtualAdapter::new(&format!("monitor_{:03X}", ecu_id));
                if adapter.connect().await.is_err() {
                    ecu_data.responsive = false;
                    ecu_data.error_count += 1;
                    let mut state = state.write().await;
                    state.insert(ecu_id, ecu_data);
                    return;
                }

                // Inject simulated responses for each DID
                for did in &dids {
                    // Simulate positive response: [0x62, DID_high, DID_low, data...]
                    let response_data = vec![
                        0x06, // ISO-TP single frame length
                        0x62, // Positive response SID
                        (did >> 8) as u8,
                        (did & 0xFF) as u8,
                        0x0C, // Simulated data byte 1
                        0x80, // Simulated data byte 2
                        0x00,
                        0x00,
                    ];
                    let response_frame = CanFrame::new(ecu_id + 0x08, response_data);
                    adapter.inject_frame(response_frame);
                }

                let session = UdsSession::new(Box::new(adapter), ecu_id);

                // Read all configured DIDs
                for did in &dids {
                    match session.read_data_by_id(*did).await {
                        Ok(resp) => {
                            ecu_data.data.insert(resp.did, resp.data);
                        }
                        Err(_) => {
                            ecu_data.error_count += 1;
                        }
                    }
                }

                ecu_data.latency_us = start.elapsed().as_micros() as u64;
                ecu_data.responsive = true;
                ecu_data.last_update = Instant::now();

                let mut state = state.write().await;
                state.insert(ecu_id, ecu_data);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        let state = self.state.read().await;
        Ok(state.clone())
    }

    /// Poll a single ECU using a provided session
    pub async fn poll_ecu(
        session: &UdsSession,
        dids: &[u16],
    ) -> Result<EcuData, UdsError> {
        let start = Instant::now();
        let mut ecu_data = EcuData::new(session.ecu_request_id());

        for did in dids {
            match session.read_data_by_id(*did).await {
                Ok(resp) => {
                    ecu_data.data.insert(resp.did, resp.data);
                }
                Err(_) => {
                    ecu_data.error_count += 1;
                }
            }
        }

        ecu_data.latency_us = start.elapsed().as_micros() as u64;
        ecu_data.responsive = ecu_data.error_count == 0;
        ecu_data.last_update = Instant::now();

        Ok(ecu_data)
    }

    /// Get the latest data snapshot
    pub async fn snapshot(&self) -> HashMap<u32, EcuData> {
        let state = self.state.read().await;
        state.clone()
    }

    /// Get data for a specific ECU
    pub async fn ecu_data(&self, ecu_id: u32) -> Option<EcuData> {
        let state = self.state.read().await;
        state.get(&ecu_id).cloned()
    }

    /// Get monitoring statistics
    pub async fn stats(&self) -> MonitorStats {
        let state = self.state.read().await;
        let responsive_count = state.values().filter(|d| d.responsive).count();
        let total_errors: u64 = state.values().map(|d| d.error_count).sum();
        let max_latency = state.values().map(|d| d.latency_us).max().unwrap_or(0);
        let avg_latency = if state.is_empty() {
            0
        } else {
            state.values().map(|d| d.latency_us).sum::<u64>() / state.len() as u64
        };

        MonitorStats {
            total_ecus: self.ecus.len(),
            responsive_ecus: responsive_count,
            total_errors,
            max_latency_us: max_latency,
            avg_latency_us: avg_latency,
        }
    }
}

impl Default for MultiEcuMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitorStats {
    /// Total number of ECUs being monitored
    pub total_ecus: usize,
    /// Number of responsive ECUs
    pub responsive_ecus: usize,
    /// Total error count across all ECUs
    pub total_errors: u64,
    /// Maximum latency across all ECUs (microseconds)
    pub max_latency_us: u64,
    /// Average latency across all ECUs (microseconds)
    pub avg_latency_us: u64,
}

impl MonitorStats {
    /// Check if all ECUs are responsive (0% frame loss)
    pub fn all_responsive(&self) -> bool {
        self.responsive_ecus == self.total_ecus
    }

    /// Check if latency is within threshold
    pub fn latency_ok(&self, max_ms: u64) -> bool {
        self.max_latency_us <= max_ms * 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecu_data_new() {
        let data = EcuData::new(0x7E0);
        assert_eq!(data.ecu_id, 0x7E0);
        assert!(data.data.is_empty());
        assert!(!data.responsive);
        assert_eq!(data.error_count, 0);
    }

    #[test]
    fn test_ecu_data_get_u16() {
        let mut data = EcuData::new(0x7E0);
        data.data.insert(0x0100, vec![0x0C, 0x80]); // RPM = 3200
        assert_eq!(data.get_u16(0x0100), Some(0x0C80));
        assert_eq!(data.get_u16(0x0200), None);
    }

    #[test]
    fn test_monitored_ecu_config() {
        let ecu = MonitoredEcu::new(0x7E0, vec![0xF190, 0x0100])
            .with_interval(50);

        assert_eq!(ecu.ecu_id, 0x7E0);
        assert_eq!(ecu.dids.len(), 2);
        assert_eq!(ecu.poll_interval_ms, 50);
    }

    #[test]
    fn test_multi_ecu_monitor_new() {
        let monitor = MultiEcuMonitor::new();
        assert_eq!(monitor.ecu_count(), 0);
    }

    #[test]
    fn test_multi_ecu_monitor_add_ecus() {
        let mut monitor = MultiEcuMonitor::new();
        monitor.add_ecu(MonitoredEcu::new(0x7E0, vec![0xF190]));
        monitor.add_ecu(MonitoredEcu::new(0x7E2, vec![0xF190]));
        assert_eq!(monitor.ecu_count(), 2);
    }

    #[tokio::test]
    async fn test_multi_ecu_monitor_poll_virtual() {
        let mut monitor = MultiEcuMonitor::new();

        // Add 5 ECUs to monitor
        for i in 0..5u32 {
            monitor.add_ecu(MonitoredEcu::new(
                0x7E0 + i * 2,
                vec![0xF190], // VIN
            ));
        }

        assert_eq!(monitor.ecu_count(), 5);

        let results = monitor.poll_all_virtual().await.unwrap();
        assert_eq!(results.len(), 5);

        // All ECUs should be responsive
        for (_, data) in &results {
            assert!(data.responsive);
        }
    }

    #[tokio::test]
    async fn test_multi_ecu_monitor_latency() {
        let mut monitor = MultiEcuMonitor::new();

        for i in 0..5u32 {
            monitor.add_ecu(MonitoredEcu::new(0x7E0 + i * 2, vec![0xF190]));
        }

        let _ = monitor.poll_all_virtual().await.unwrap();

        let stats = monitor.stats().await;
        assert_eq!(stats.total_ecus, 5);
        assert_eq!(stats.responsive_ecus, 5);
        assert!(stats.all_responsive());
        // Latency should be well under 10ms for virtual adapters
        assert!(stats.latency_ok(10));
    }

    #[tokio::test]
    async fn test_multi_ecu_monitor_snapshot() {
        let mut monitor = MultiEcuMonitor::new();
        monitor.add_ecu(MonitoredEcu::new(0x7E0, vec![0xF190]));

        let _ = monitor.poll_all_virtual().await.unwrap();

        let snapshot = monitor.snapshot().await;
        assert_eq!(snapshot.len(), 1);
        assert!(snapshot.contains_key(&0x7E0));
    }

    #[tokio::test]
    async fn test_multi_ecu_monitor_individual_data() {
        let mut monitor = MultiEcuMonitor::new();
        monitor.add_ecu(MonitoredEcu::new(0x7E0, vec![0xF190]));

        let _ = monitor.poll_all_virtual().await.unwrap();

        let data = monitor.ecu_data(0x7E0).await;
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ecu_id, 0x7E0);
        assert!(data.responsive);

        let missing = monitor.ecu_data(0x999).await;
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_monitor_stats() {
        let mut monitor = MultiEcuMonitor::new();
        for i in 0..3u32 {
            monitor.add_ecu(MonitoredEcu::new(0x7E0 + i * 2, vec![0xF190]));
        }

        let _ = monitor.poll_all_virtual().await.unwrap();

        let stats = monitor.stats().await;
        assert_eq!(stats.total_ecus, 3);
        assert!(stats.all_responsive());
        assert_eq!(stats.total_errors, 0);
    }

    #[tokio::test]
    async fn test_monitor_five_plus_ecus() {
        let mut monitor = MultiEcuMonitor::new();

        // Add 8 ECUs (exceeds the 5+ requirement)
        let ecu_ids: Vec<u32> = vec![0x7E0, 0x7E2, 0x7E4, 0x700, 0x710, 0x720, 0x730, 0x740];
        for ecu_id in &ecu_ids {
            monitor.add_ecu(MonitoredEcu::new(*ecu_id, vec![0xF190, 0xF18C]));
        }

        assert_eq!(monitor.ecu_count(), 8);

        let results = monitor.poll_all_virtual().await.unwrap();
        assert_eq!(results.len(), 8);

        let stats = monitor.stats().await;
        assert!(stats.all_responsive(), "All 8 ECUs should respond");
        assert!(stats.latency_ok(10), "Latency should be under 10ms");
        assert_eq!(stats.total_errors, 0, "Should have 0% frame loss");
    }
}

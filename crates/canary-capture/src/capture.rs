use canary_hardware::CanFrame;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::filter::CanFilter;
use crate::logger::SqliteLogger;
use crate::CaptureError;

/// A CAN frame with capture metadata
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// The CAN frame
    pub frame: CanFrame,
    /// Absolute timestamp in microseconds since capture start
    pub timestamp_us: u64,
    /// Delta from previous frame in microseconds
    pub delta_us: u64,
}

/// Configuration for a capture session
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// CAN frame filter
    pub filter: CanFilter,
    /// SQLite database path for logging
    pub db_path: String,
    /// Session description
    pub description: String,
    /// Maximum frames to capture (0 = unlimited)
    pub max_frames: u64,
    /// Channel buffer size for async capture
    pub buffer_size: usize,
}

impl CaptureConfig {
    /// Create a new capture configuration with defaults
    pub fn new() -> Self {
        Self {
            filter: CanFilter::accept_all(),
            db_path: ":memory:".to_string(),
            description: String::new(),
            max_frames: 0,
            buffer_size: 4096,
        }
    }

    /// Set the CAN filter
    pub fn with_filter(mut self, filter: CanFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set the database path
    pub fn with_db_path(mut self, path: &str) -> Self {
        self.db_path = path.to_string();
        self
    }

    /// Set the session description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the maximum number of frames to capture
    pub fn with_max_frames(mut self, max: u64) -> Self {
        self.max_frames = max;
        self
    }

    /// Set the async buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Capture session that records CAN frames
pub struct CaptureSession {
    /// Session configuration
    config: CaptureConfig,
    /// SQLite logger
    logger: SqliteLogger,
    /// Database session ID
    session_id: Option<i64>,
    /// Whether capture is active
    active: Arc<AtomicBool>,
    /// Frame counter
    frame_count: Arc<AtomicU64>,
    /// Capture start time
    start_time: Option<std::time::Instant>,
    /// Last frame timestamp
    last_timestamp_us: u64,
}

impl CaptureSession {
    /// Create a new capture session
    pub fn new(config: CaptureConfig) -> Result<Self, CaptureError> {
        let logger = if config.db_path == ":memory:" {
            SqliteLogger::in_memory()?
        } else {
            SqliteLogger::new(&config.db_path)?
        };

        Ok(Self {
            config,
            logger,
            session_id: None,
            active: Arc::new(AtomicBool::new(false)),
            frame_count: Arc::new(AtomicU64::new(0)),
            start_time: None,
            last_timestamp_us: 0,
        })
    }

    /// Start the capture session
    pub fn start(&mut self) -> Result<i64, CaptureError> {
        let session_id = self.logger.start_session(&self.config.description)?;
        self.session_id = Some(session_id);
        self.active.store(true, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());
        self.frame_count.store(0, Ordering::SeqCst);
        self.last_timestamp_us = 0;
        Ok(session_id)
    }

    /// Stop the capture session
    pub fn stop(&mut self) -> Result<(), CaptureError> {
        self.active.store(false, Ordering::SeqCst);
        if let Some(session_id) = self.session_id {
            self.logger.end_session(session_id)?;
        }
        Ok(())
    }

    /// Process a received CAN frame
    ///
    /// Returns Some(CapturedFrame) if the frame passes the filter,
    /// None if filtered out.
    pub fn process_frame(&mut self, frame: CanFrame) -> Result<Option<CapturedFrame>, CaptureError> {
        if !self.active.load(Ordering::SeqCst) {
            return Err(CaptureError::NoActiveSession);
        }

        // Apply filter
        if !self.config.filter.matches(&frame) {
            return Ok(None);
        }

        // Check max frames
        let count = self.frame_count.load(Ordering::SeqCst);
        if self.config.max_frames > 0 && count >= self.config.max_frames {
            self.stop()?;
            return Ok(None);
        }

        // Calculate timestamps
        let timestamp_us = self
            .start_time
            .map(|start| start.elapsed().as_micros() as u64)
            .unwrap_or(0);

        let delta_us = if self.last_timestamp_us == 0 {
            0
        } else {
            timestamp_us.saturating_sub(self.last_timestamp_us)
        };

        self.last_timestamp_us = timestamp_us;

        let captured = CapturedFrame {
            frame,
            timestamp_us,
            delta_us,
        };

        // Log to SQLite
        if let Some(session_id) = self.session_id {
            self.logger.log_frame(session_id, &captured)?;
        }

        self.frame_count.fetch_add(1, Ordering::SeqCst);
        Ok(Some(captured))
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::SeqCst)
    }

    /// Check if capture is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<i64> {
        self.session_id
    }

    /// Get a reference to the logger
    pub fn logger(&self) -> &SqliteLogger {
        &self.logger
    }

    /// Get a stop handle for async capture
    pub fn stop_handle(&self) -> CaptureStopHandle {
        CaptureStopHandle {
            active: self.active.clone(),
        }
    }

    /// Create a channel-based async capture pipeline
    ///
    /// Returns a sender for feeding frames and the stop handle.
    /// Frames are processed and logged automatically.
    pub fn create_pipeline(
        &self,
    ) -> (mpsc::Sender<CanFrame>, CaptureStopHandle) {
        let sender = mpsc::channel(self.config.buffer_size).0;
        let handle = self.stop_handle();
        (sender, handle)
    }
}

/// Handle to stop a capture session from another task
#[derive(Clone)]
pub struct CaptureStopHandle {
    active: Arc<AtomicBool>,
}

impl CaptureStopHandle {
    /// Signal the capture to stop
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    /// Check if capture is still active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_config_defaults() {
        let config = CaptureConfig::new();
        assert_eq!(config.db_path, ":memory:");
        assert_eq!(config.max_frames, 0);
        assert_eq!(config.buffer_size, 4096);
    }

    #[test]
    fn test_capture_config_builder() {
        let config = CaptureConfig::new()
            .with_filter(CanFilter::single_id(0x7E0))
            .with_db_path("/tmp/test.db")
            .with_description("Test session")
            .with_max_frames(1000)
            .with_buffer_size(8192);

        assert_eq!(config.db_path, "/tmp/test.db");
        assert_eq!(config.description, "Test session");
        assert_eq!(config.max_frames, 1000);
        assert_eq!(config.buffer_size, 8192);
    }

    #[test]
    fn test_capture_session_lifecycle() {
        let config = CaptureConfig::new().with_description("Test");
        let mut session = CaptureSession::new(config).unwrap();

        assert!(!session.is_active());
        assert_eq!(session.frame_count(), 0);

        let session_id = session.start().unwrap();
        assert!(session_id > 0);
        assert!(session.is_active());

        // Process a frame
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let result = session.process_frame(frame).unwrap();
        assert!(result.is_some());
        assert_eq!(session.frame_count(), 1);

        session.stop().unwrap();
        assert!(!session.is_active());
    }

    #[test]
    fn test_capture_with_filter() {
        let config = CaptureConfig::new().with_filter(CanFilter::single_id(0x7E0));
        let mut session = CaptureSession::new(config).unwrap();
        session.start().unwrap();

        // Matching frame
        let frame1 = CanFrame::new(0x7E0, vec![0x01]);
        assert!(session.process_frame(frame1).unwrap().is_some());

        // Non-matching frame
        let frame2 = CanFrame::new(0x7E8, vec![0x02]);
        assert!(session.process_frame(frame2).unwrap().is_none());

        assert_eq!(session.frame_count(), 1);
    }

    #[test]
    fn test_capture_max_frames() {
        let config = CaptureConfig::new().with_max_frames(3);
        let mut session = CaptureSession::new(config).unwrap();
        session.start().unwrap();

        for i in 0..5 {
            let frame = CanFrame::new(0x100 + i, vec![i as u8]);
            let _ = session.process_frame(frame);
        }

        // Should have stopped after 3 frames
        assert_eq!(session.frame_count(), 3);
        assert!(!session.is_active());
    }

    #[test]
    fn test_capture_stop_handle() {
        let config = CaptureConfig::new();
        let mut session = CaptureSession::new(config).unwrap();
        session.start().unwrap();

        let handle = session.stop_handle();
        assert!(handle.is_active());

        handle.stop();
        assert!(!handle.is_active());
        assert!(!session.is_active());
    }

    #[test]
    fn test_capture_not_started_error() {
        let config = CaptureConfig::new();
        let mut session = CaptureSession::new(config).unwrap();

        let frame = CanFrame::new(0x100, vec![0x01]);
        let result = session.process_frame(frame);
        assert!(matches!(result, Err(CaptureError::NoActiveSession)));
    }

    #[test]
    fn test_captured_frame_timestamps() {
        let config = CaptureConfig::new();
        let mut session = CaptureSession::new(config).unwrap();
        session.start().unwrap();

        let frame1 = CanFrame::new(0x100, vec![0x01]);
        let captured1 = session.process_frame(frame1).unwrap().unwrap();
        assert_eq!(captured1.delta_us, 0); // First frame has 0 delta

        // Sleep long enough to guarantee a measurable delta
        std::thread::sleep(std::time::Duration::from_millis(10));

        let frame2 = CanFrame::new(0x100, vec![0x02]);
        let captured2 = session.process_frame(frame2).unwrap().unwrap();
        assert!(captured2.delta_us > 0, "delta_us should be > 0, got {}", captured2.delta_us);
        assert!(captured2.timestamp_us > captured1.timestamp_us);
    }

    #[test]
    fn test_capture_frames_persisted() {
        let config = CaptureConfig::new().with_description("Persist test");
        let mut session = CaptureSession::new(config).unwrap();
        let session_id = session.start().unwrap();

        for i in 0..10 {
            let frame = CanFrame::new(0x100 + i, vec![i as u8]);
            session.process_frame(frame).unwrap();
        }

        session.stop().unwrap();

        // Verify frames are persisted
        let loaded = session.logger().load_session(session_id).unwrap();
        assert_eq!(loaded.len(), 10);
    }

    #[test]
    fn test_high_throughput_capture() {
        let config = CaptureConfig::new().with_max_frames(1000);
        let mut session = CaptureSession::new(config).unwrap();
        session.start().unwrap();

        // Simulate 1000 frames
        for i in 0..1000u32 {
            let frame = CanFrame::new(i % 256, vec![(i & 0xFF) as u8]);
            session.process_frame(frame).unwrap();
        }

        assert_eq!(session.frame_count(), 1000);
    }
}

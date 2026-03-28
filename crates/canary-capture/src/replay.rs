use canary_hardware::CanAdapter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::capture::CapturedFrame;
use crate::logger::SqliteLogger;
use crate::CaptureError;

/// Speed multiplier for replay
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplaySpeed {
    /// Original speed (1x)
    Original,
    /// Custom speed multiplier (e.g., 2.0 = 2x faster)
    Multiplier(f64),
    /// As fast as possible (no delays)
    MaxSpeed,
}

impl ReplaySpeed {
    /// Calculate the actual delay for a given delta
    pub fn apply_to_delta(&self, delta_us: u64) -> u64 {
        match self {
            ReplaySpeed::Original => delta_us,
            ReplaySpeed::Multiplier(mult) => {
                if *mult <= 0.0 {
                    0
                } else {
                    (delta_us as f64 / mult) as u64
                }
            }
            ReplaySpeed::MaxSpeed => 0,
        }
    }
}

/// Configuration for replay operations
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Replay speed
    pub speed: ReplaySpeed,
    /// Whether to loop the replay
    pub loop_replay: bool,
    /// Maximum number of loops (0 = infinite when loop_replay is true)
    pub max_loops: u32,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            speed: ReplaySpeed::Original,
            loop_replay: false,
            max_loops: 0,
        }
    }
}

impl ReplayConfig {
    /// Create with original speed
    pub fn original_speed() -> Self {
        Self::default()
    }

    /// Create with custom speed multiplier
    pub fn with_speed(mut self, speed: ReplaySpeed) -> Self {
        self.speed = speed;
        self
    }

    /// Enable looping
    pub fn with_loop(mut self, max_loops: u32) -> Self {
        self.loop_replay = true;
        self.max_loops = max_loops;
        self
    }
}

/// Engine for replaying captured CAN frames
pub struct ReplayEngine {
    /// Replay configuration
    config: ReplayConfig,
    /// Whether replay is active
    active: Arc<AtomicBool>,
    /// Frames replayed counter
    frames_replayed: u64,
}

impl ReplayEngine {
    /// Create a new replay engine with the given configuration
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            active: Arc::new(AtomicBool::new(false)),
            frames_replayed: 0,
        }
    }

    /// Create a replay engine with default settings (original speed)
    pub fn original_speed() -> Self {
        Self::new(ReplayConfig::original_speed())
    }

    /// Replay frames from a list onto a CAN adapter
    pub async fn replay_frames(
        &mut self,
        adapter: &dyn CanAdapter,
        frames: &[CapturedFrame],
    ) -> Result<u64, CaptureError> {
        if frames.is_empty() {
            return Ok(0);
        }

        self.active.store(true, Ordering::SeqCst);
        self.frames_replayed = 0;

        let mut loop_count = 0u32;

        loop {
            for (i, captured) in frames.iter().enumerate() {
                if !self.active.load(Ordering::SeqCst) {
                    return Ok(self.frames_replayed);
                }

                // Apply timing delay (skip for first frame)
                if i > 0 {
                    let delay = self.config.speed.apply_to_delta(captured.delta_us);
                    if delay > 0 {
                        tokio::time::sleep(std::time::Duration::from_micros(delay)).await;
                    }
                }

                // Send the frame
                adapter.send_frame(&captured.frame).await?;
                self.frames_replayed += 1;
            }

            loop_count += 1;

            if !self.config.loop_replay {
                break;
            }

            if self.config.max_loops > 0 && loop_count >= self.config.max_loops {
                break;
            }
        }

        self.active.store(false, Ordering::SeqCst);
        Ok(self.frames_replayed)
    }

    /// Replay a session from the database onto a CAN adapter
    pub async fn replay_session(
        &mut self,
        adapter: &dyn CanAdapter,
        logger: &SqliteLogger,
        session_id: i64,
    ) -> Result<u64, CaptureError> {
        let frames = logger.load_session(session_id)?;
        if frames.is_empty() {
            return Err(CaptureError::SessionNotFound(session_id));
        }
        self.replay_frames(adapter, &frames).await
    }

    /// Get a stop handle for the replay
    pub fn stop_handle(&self) -> ReplayStopHandle {
        ReplayStopHandle {
            active: self.active.clone(),
        }
    }

    /// Check if replay is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Get the number of frames replayed
    pub fn frames_replayed(&self) -> u64 {
        self.frames_replayed
    }
}

/// Handle to stop a replay from another task
#[derive(Clone)]
pub struct ReplayStopHandle {
    active: Arc<AtomicBool>,
}

impl ReplayStopHandle {
    /// Signal the replay to stop
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    /// Check if replay is still active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canary_hardware::{CanFrame, VirtualAdapter};

    fn make_frames(count: usize) -> Vec<CapturedFrame> {
        (0..count)
            .map(|i| CapturedFrame {
                frame: CanFrame::new(0x100 + i as u32, vec![i as u8]),
                timestamp_us: i as u64 * 1000,
                delta_us: if i == 0 { 0 } else { 1000 },
            })
            .collect()
    }

    #[test]
    fn test_replay_speed_original() {
        let speed = ReplaySpeed::Original;
        assert_eq!(speed.apply_to_delta(1000), 1000);
        assert_eq!(speed.apply_to_delta(0), 0);
    }

    #[test]
    fn test_replay_speed_multiplier() {
        let speed = ReplaySpeed::Multiplier(2.0);
        assert_eq!(speed.apply_to_delta(1000), 500);

        let speed = ReplaySpeed::Multiplier(0.5);
        assert_eq!(speed.apply_to_delta(1000), 2000);
    }

    #[test]
    fn test_replay_speed_max() {
        let speed = ReplaySpeed::MaxSpeed;
        assert_eq!(speed.apply_to_delta(1000), 0);
        assert_eq!(speed.apply_to_delta(999999), 0);
    }

    #[test]
    fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert!(matches!(config.speed, ReplaySpeed::Original));
        assert!(!config.loop_replay);
    }

    #[test]
    fn test_replay_config_builder() {
        let config = ReplayConfig::original_speed()
            .with_speed(ReplaySpeed::Multiplier(3.0))
            .with_loop(5);

        assert!(matches!(config.speed, ReplaySpeed::Multiplier(s) if (s - 3.0).abs() < f64::EPSILON));
        assert!(config.loop_replay);
        assert_eq!(config.max_loops, 5);
    }

    #[tokio::test]
    async fn test_replay_frames_basic() {
        let mut adapter = VirtualAdapter::new("test_replay");
        adapter.connect().await.unwrap();

        let frames = make_frames(5);
        let mut engine = ReplayEngine::new(ReplayConfig::default().with_speed(ReplaySpeed::MaxSpeed));

        let count = engine.replay_frames(&adapter, &frames).await.unwrap();
        assert_eq!(count, 5);
        assert_eq!(engine.frames_replayed(), 5);

        // Verify frames were sent
        let sent = adapter.get_sent_frames();
        assert_eq!(sent.len(), 5);
        assert_eq!(sent[0].id, 0x100);
        assert_eq!(sent[4].id, 0x104);
    }

    #[tokio::test]
    async fn test_replay_empty_frames() {
        let mut adapter = VirtualAdapter::new("test_empty");
        adapter.connect().await.unwrap();

        let mut engine = ReplayEngine::original_speed();
        let count = engine.replay_frames(&adapter, &[]).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_replay_with_loop() {
        let mut adapter = VirtualAdapter::new("test_loop");
        adapter.connect().await.unwrap();

        let frames = make_frames(3);
        let config = ReplayConfig::default()
            .with_speed(ReplaySpeed::MaxSpeed)
            .with_loop(2);

        let mut engine = ReplayEngine::new(config);
        let count = engine.replay_frames(&adapter, &frames).await.unwrap();
        assert_eq!(count, 6); // 3 frames * 2 loops
    }

    #[tokio::test]
    async fn test_replay_stop_handle() {
        let mut adapter = VirtualAdapter::new("test_stop");
        adapter.connect().await.unwrap();

        // Use delays so we can stop mid-replay
        let frames: Vec<CapturedFrame> = (0..100)
            .map(|i| CapturedFrame {
                frame: CanFrame::new(0x100 + i as u32, vec![i as u8]),
                timestamp_us: i as u64 * 10_000,
                delta_us: if i == 0 { 0 } else { 10_000 }, // 10ms between frames
            })
            .collect();

        let mut engine = ReplayEngine::new(
            ReplayConfig::default().with_speed(ReplaySpeed::Original),
        );
        let handle = engine.stop_handle();

        // Spawn a task that stops after a short delay
        let stop_handle = handle.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            stop_handle.stop();
        });

        let count = engine.replay_frames(&adapter, &frames).await.unwrap();
        // Should have stopped well before all 100 frames (each takes 10ms)
        assert!(count < 100, "count should be < 100, got {}", count);
        assert!(!handle.is_active());
    }

    #[tokio::test]
    async fn test_replay_session_from_db() {
        let mut adapter = VirtualAdapter::new("test_db_replay");
        adapter.connect().await.unwrap();

        // Create a logger with some frames
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("replay test").unwrap();

        for i in 0..5 {
            let frame = CapturedFrame {
                frame: CanFrame::new(0x200 + i, vec![i as u8]),
                timestamp_us: i as u64 * 1000,
                delta_us: if i == 0 { 0 } else { 1000 },
            };
            logger.log_frame(session_id, &frame).unwrap();
        }
        logger.end_session(session_id).unwrap();

        // Replay from database
        let mut engine = ReplayEngine::new(
            ReplayConfig::default().with_speed(ReplaySpeed::MaxSpeed),
        );
        let count = engine
            .replay_session(&adapter, &logger, session_id)
            .await
            .unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_replay_session_not_found() {
        let mut adapter = VirtualAdapter::new("test_not_found");
        adapter.connect().await.unwrap();

        let logger = SqliteLogger::in_memory().unwrap();
        let mut engine = ReplayEngine::original_speed();

        let result = engine.replay_session(&adapter, &logger, 999).await;
        assert!(matches!(result, Err(CaptureError::SessionNotFound(999))));
    }
}

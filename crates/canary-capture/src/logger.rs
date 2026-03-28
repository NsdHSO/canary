use rusqlite::{params, Connection};

use crate::capture::CapturedFrame;
use crate::CaptureError;

/// SQLite-based logger for CAN frame capture sessions
///
/// Stores captured frames in a SQLite database organized by sessions.
/// Each session has metadata (start time, description) and contains
/// the captured frames with their timestamps.
pub struct SqliteLogger {
    conn: Connection,
}

impl SqliteLogger {
    /// Create a new SQLite logger, creating the database if needed
    pub fn new(db_path: &str) -> Result<Self, CaptureError> {
        let conn = Connection::open(db_path)?;
        let logger = Self { conn };
        logger.init_tables()?;
        Ok(logger)
    }

    /// Create an in-memory SQLite logger (for testing)
    pub fn in_memory() -> Result<Self, CaptureError> {
        let conn = Connection::open_in_memory()?;
        let logger = Self { conn };
        logger.init_tables()?;
        Ok(logger)
    }

    /// Initialize database tables
    fn init_tables(&self) -> Result<(), CaptureError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS capture_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL DEFAULT '',
                start_time_us INTEGER NOT NULL,
                end_time_us INTEGER,
                frame_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS captured_frames (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                can_id INTEGER NOT NULL,
                data BLOB NOT NULL,
                extended INTEGER NOT NULL DEFAULT 0,
                timestamp_us INTEGER NOT NULL,
                delta_us INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (session_id) REFERENCES capture_sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_frames_session
                ON captured_frames(session_id);
            CREATE INDEX IF NOT EXISTS idx_frames_can_id
                ON captured_frames(session_id, can_id);
            CREATE INDEX IF NOT EXISTS idx_frames_timestamp
                ON captured_frames(session_id, timestamp_us);",
        )?;
        Ok(())
    }

    /// Start a new capture session
    pub fn start_session(&self, description: &str) -> Result<i64, CaptureError> {
        let now_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        self.conn.execute(
            "INSERT INTO capture_sessions (description, start_time_us) VALUES (?1, ?2)",
            params![description, now_us],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// End a capture session
    pub fn end_session(&self, session_id: i64) -> Result<(), CaptureError> {
        let now_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        let frame_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM captured_frames WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "UPDATE capture_sessions SET end_time_us = ?1, frame_count = ?2 WHERE id = ?3",
            params![now_us, frame_count, session_id],
        )?;

        Ok(())
    }

    /// Log a single captured frame
    pub fn log_frame(
        &self,
        session_id: i64,
        frame: &CapturedFrame,
    ) -> Result<(), CaptureError> {
        self.conn.execute(
            "INSERT INTO captured_frames (session_id, can_id, data, extended, timestamp_us, delta_us)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_id,
                frame.frame.id as i64,
                &frame.frame.data,
                frame.frame.extended as i32,
                frame.timestamp_us as i64,
                frame.delta_us as i64,
            ],
        )?;
        Ok(())
    }

    /// Log multiple frames in a batch (transaction)
    pub fn log_frames(
        &self,
        session_id: i64,
        frames: &[CapturedFrame],
    ) -> Result<(), CaptureError> {
        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO captured_frames (session_id, can_id, data, extended, timestamp_us, delta_us)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;

            for frame in frames {
                stmt.execute(params![
                    session_id,
                    frame.frame.id as i64,
                    &frame.frame.data,
                    frame.frame.extended as i32,
                    frame.timestamp_us as i64,
                    frame.delta_us as i64,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Load all frames from a session
    pub fn load_session(&self, session_id: i64) -> Result<Vec<CapturedFrame>, CaptureError> {
        let mut stmt = self.conn.prepare(
            "SELECT can_id, data, extended, timestamp_us, delta_us
             FROM captured_frames
             WHERE session_id = ?1
             ORDER BY timestamp_us ASC",
        )?;

        let frames = stmt
            .query_map(params![session_id], |row| {
                let can_id: i64 = row.get(0)?;
                let data: Vec<u8> = row.get(1)?;
                let extended: bool = row.get::<_, i32>(2)? != 0;
                let timestamp_us: i64 = row.get(3)?;
                let delta_us: i64 = row.get(4)?;

                let mut frame = canary_hardware::CanFrame::new(can_id as u32, data);
                frame.extended = extended;

                Ok(CapturedFrame {
                    frame,
                    timestamp_us: timestamp_us as u64,
                    delta_us: delta_us as u64,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(frames)
    }

    /// Get frame count for a session
    pub fn frame_count(&self, session_id: i64) -> Result<u64, CaptureError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM captured_frames WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// List all capture sessions
    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>, CaptureError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, start_time_us, end_time_us, frame_count, created_at
             FROM capture_sessions
             ORDER BY id DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    start_time_us: row.get::<_, i64>(2)? as u64,
                    end_time_us: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                    frame_count: row.get::<_, i64>(4)? as u64,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Delete a capture session and its frames
    pub fn delete_session(&self, session_id: i64) -> Result<(), CaptureError> {
        self.conn.execute(
            "DELETE FROM captured_frames WHERE session_id = ?1",
            params![session_id],
        )?;
        self.conn.execute(
            "DELETE FROM capture_sessions WHERE id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    /// Get unique CAN IDs captured in a session
    pub fn unique_can_ids(&self, session_id: i64) -> Result<Vec<u32>, CaptureError> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT can_id FROM captured_frames WHERE session_id = ?1 ORDER BY can_id",
        )?;

        let ids = stmt
            .query_map(params![session_id], |row| {
                let id: i64 = row.get(0)?;
                Ok(id as u32)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }
}

/// Information about a capture session
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session ID
    pub id: i64,
    /// Session description
    pub description: String,
    /// Start time in microseconds since epoch
    pub start_time_us: u64,
    /// End time in microseconds since epoch (None if still active)
    pub end_time_us: Option<u64>,
    /// Number of captured frames
    pub frame_count: u64,
    /// Creation timestamp
    pub created_at: String,
}

impl SessionInfo {
    /// Get session duration in seconds
    pub fn duration_secs(&self) -> Option<f64> {
        self.end_time_us
            .map(|end| (end - self.start_time_us) as f64 / 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canary_hardware::CanFrame;

    fn make_captured_frame(id: u32, data: Vec<u8>, ts: u64, delta: u64) -> CapturedFrame {
        CapturedFrame {
            frame: CanFrame::new(id, data),
            timestamp_us: ts,
            delta_us: delta,
        }
    }

    #[test]
    fn test_create_in_memory_logger() {
        let logger = SqliteLogger::in_memory().unwrap();
        let sessions = logger.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_start_end_session() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("Test capture").unwrap();
        assert!(session_id > 0);

        logger.end_session(session_id).unwrap();

        let sessions = logger.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].description, "Test capture");
    }

    #[test]
    fn test_log_and_load_frames() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("Frame test").unwrap();

        let frame1 = make_captured_frame(0x7E0, vec![0x02, 0x10, 0x01], 1000, 0);
        let frame2 = make_captured_frame(0x7E8, vec![0x06, 0x50, 0x01], 2000, 1000);
        let frame3 = make_captured_frame(0x7E0, vec![0x03, 0x22, 0xF1, 0x90], 3000, 1000);

        logger.log_frame(session_id, &frame1).unwrap();
        logger.log_frame(session_id, &frame2).unwrap();
        logger.log_frame(session_id, &frame3).unwrap();

        let loaded = logger.load_session(session_id).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].frame.id, 0x7E0);
        assert_eq!(loaded[1].frame.id, 0x7E8);
        assert_eq!(loaded[2].frame.id, 0x7E0);
        assert_eq!(loaded[0].timestamp_us, 1000);
        assert_eq!(loaded[1].delta_us, 1000);
    }

    #[test]
    fn test_log_batch_frames() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("Batch test").unwrap();

        let frames: Vec<CapturedFrame> = (0..100)
            .map(|i| make_captured_frame(0x100 + i, vec![i as u8], i as u64 * 100, 100))
            .collect();

        logger.log_frames(session_id, &frames).unwrap();

        let count = logger.frame_count(session_id).unwrap();
        assert_eq!(count, 100);
    }

    #[test]
    fn test_frame_count() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("Count test").unwrap();

        assert_eq!(logger.frame_count(session_id).unwrap(), 0);

        logger
            .log_frame(
                session_id,
                &make_captured_frame(0x100, vec![0x01], 1000, 0),
            )
            .unwrap();

        assert_eq!(logger.frame_count(session_id).unwrap(), 1);
    }

    #[test]
    fn test_unique_can_ids() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("IDs test").unwrap();

        logger
            .log_frame(
                session_id,
                &make_captured_frame(0x7E0, vec![0x01], 1000, 0),
            )
            .unwrap();
        logger
            .log_frame(
                session_id,
                &make_captured_frame(0x7E8, vec![0x02], 2000, 1000),
            )
            .unwrap();
        logger
            .log_frame(
                session_id,
                &make_captured_frame(0x7E0, vec![0x03], 3000, 1000),
            )
            .unwrap();

        let ids = logger.unique_can_ids(session_id).unwrap();
        assert_eq!(ids, vec![0x7E0, 0x7E8]);
    }

    #[test]
    fn test_delete_session() {
        let logger = SqliteLogger::in_memory().unwrap();
        let session_id = logger.start_session("Delete test").unwrap();

        logger
            .log_frame(
                session_id,
                &make_captured_frame(0x100, vec![0x01], 1000, 0),
            )
            .unwrap();

        logger.delete_session(session_id).unwrap();

        let sessions = logger.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_multiple_sessions() {
        let logger = SqliteLogger::in_memory().unwrap();
        let s1 = logger.start_session("Session 1").unwrap();
        let s2 = logger.start_session("Session 2").unwrap();

        logger
            .log_frame(s1, &make_captured_frame(0x100, vec![0x01], 1000, 0))
            .unwrap();
        logger
            .log_frame(s2, &make_captured_frame(0x200, vec![0x02], 1000, 0))
            .unwrap();

        let frames1 = logger.load_session(s1).unwrap();
        let frames2 = logger.load_session(s2).unwrap();
        assert_eq!(frames1.len(), 1);
        assert_eq!(frames2.len(), 1);
        assert_eq!(frames1[0].frame.id, 0x100);
        assert_eq!(frames2[0].frame.id, 0x200);
    }

    #[test]
    fn test_session_info_duration() {
        let info = SessionInfo {
            id: 1,
            description: "test".to_string(),
            start_time_us: 1_000_000,
            end_time_us: Some(3_500_000),
            frame_count: 100,
            created_at: "2024-01-01".to_string(),
        };
        assert!((info.duration_secs().unwrap() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_session_info_no_end() {
        let info = SessionInfo {
            id: 1,
            description: "test".to_string(),
            start_time_us: 1_000_000,
            end_time_us: None,
            frame_count: 0,
            created_at: "2024-01-01".to_string(),
        };
        assert!(info.duration_secs().is_none());
    }
}

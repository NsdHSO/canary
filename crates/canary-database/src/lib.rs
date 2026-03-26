use once_cell::sync::OnceCell;
use sea_orm::{Database, DatabaseConnection, DbErr};

// Single global connection pool for entire library
static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// Initialize the database connection pool
///
/// This must be called once before any database operations
/// All feature crates share this single connection pool
pub async fn initialize(database_url: &str) -> Result<(), DbErr> {
    let db = Database::connect(database_url).await?;
    DB.set(db)
        .map_err(|_| DbErr::Custom("Database already initialized".into()))?;
    Ok(())
}

/// Get reference to the global database connection
///
/// Panics if initialize() has not been called
pub fn get_connection() -> &'static DatabaseConnection {
    DB.get()
        .expect("Database not initialized - call canary_database::initialize() first")
}

/// Check if database has been initialized
pub fn is_initialized() -> bool {
    DB.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_in_memory() {
        // Use in-memory SQLite for testing
        let result = initialize("sqlite::memory:").await;
        assert!(result.is_ok());
        assert!(is_initialized());
    }
}

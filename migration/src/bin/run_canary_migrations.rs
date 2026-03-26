use sea_orm::{Database, DbBackend, DbErr, Statement};
use sea_orm_migration::prelude::*;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let db = Database::connect(&db_url).await?;

    // Ensure search_path is set to canary schema
    println!("Setting search_path to canary, public");
    db.execute_unprepared("SET search_path TO canary, public")
        .await?;

    // Create migration tracking table in canary schema
    println!("Creating migration tracking table in canary schema");
    db.execute_unprepared(
        "CREATE TABLE IF NOT EXISTS canary.seaql_migrations (
            version character varying NOT NULL,
            applied_at bigint NOT NULL,
            CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version)
        )",
    )
    .await?;

    // Get list of already applied migrations (declarative style)
    let applied_migrations = get_applied_migrations(&db).await?;
    println!("Found {} already applied migrations", applied_migrations.len());

    // Get all migrations
    let migrations = migration::Migrator::migrations();
    let manager = SchemaManager::new(&db);

    // Run pending migrations
    println!("Running pending migrations...");
    let pending_migrations: Vec<_> = migrations
        .iter()
        .filter(|m| !applied_migrations.contains(m.name()))
        .collect();

    for migration in pending_migrations {
        let version = migration.name().to_string();
        println!("  Applying migration: {}", version);

        migration.up(&manager).await?;

        let now = chrono::Utc::now().timestamp();
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO canary.seaql_migrations (version, applied_at) VALUES ($1, $2)",
            vec![version.into(), now.into()],
        ))
        .await?;
    }

    println!("Successfully applied {} migrations", migrations.len() - applied_migrations.len());
    Ok(())
}

async fn get_applied_migrations(db: &sea_orm::DatabaseConnection) -> Result<HashSet<String>, DbErr> {
    use sea_orm::FromQueryResult;

    #[derive(FromQueryResult)]
    struct MigrationVersion {
        version: String,
    }

    let results = MigrationVersion::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT version FROM canary.seaql_migrations",
        vec![],
    ))
    .all(db)
    .await?;

    Ok(results.into_iter().map(|r| r.version).collect())
}

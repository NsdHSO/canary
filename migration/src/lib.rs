pub use sea_orm_migration::prelude::*;

mod m20260326_000001_create_canary_schema;
mod m20260326_000002_create_custom_pinouts;
mod m20260326_000003_create_custom_dtc_notes;
mod m20260326_000004_create_service_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260326_000001_create_canary_schema::Migration),
            Box::new(m20260326_000002_create_custom_pinouts::Migration),
            Box::new(m20260326_000003_create_custom_dtc_notes::Migration),
            Box::new(m20260326_000004_create_service_logs::Migration),
        ]
    }
}

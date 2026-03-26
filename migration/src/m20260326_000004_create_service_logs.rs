use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("canary"), ServiceLogs::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServiceLogs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ServiceLogs::UserId).string())
                    .col(
                        ColumnDef::new(ServiceLogs::VehicleInfo)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ServiceLogs::ProcedureId).string())
                    .col(
                        ColumnDef::new(ServiceLogs::PerformedDate)
                            .date()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ServiceLogs::Mileage).integer())
                    .col(ColumnDef::new(ServiceLogs::PartsUsed).json())
                    .col(ColumnDef::new(ServiceLogs::Notes).text())
                    .col(
                        ColumnDef::new(ServiceLogs::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_service_logs_user")
                    .table((Alias::new("canary"), ServiceLogs::Table))
                    .col(ServiceLogs::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("canary"), ServiceLogs::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceLogs {
    Table,
    Id,
    UserId,
    VehicleInfo,
    ProcedureId,
    PerformedDate,
    Mileage,
    PartsUsed,
    Notes,
    CreatedAt,
}

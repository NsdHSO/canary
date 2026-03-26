use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("canary"), CustomDtcNotes::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CustomDtcNotes::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CustomDtcNotes::UserId).string())
                    .col(
                        ColumnDef::new(CustomDtcNotes::DtcCode)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CustomDtcNotes::VehicleInfo).json())
                    .col(
                        ColumnDef::new(CustomDtcNotes::Notes)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CustomDtcNotes::RepairActions).json())
                    .col(
                        ColumnDef::new(CustomDtcNotes::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CustomDtcNotes::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on user_id and dtc_code
        manager
            .create_index(
                Index::create()
                    .name("idx_custom_dtc_user_code")
                    .table((Alias::new("canary"), CustomDtcNotes::Table))
                    .col(CustomDtcNotes::UserId)
                    .col(CustomDtcNotes::DtcCode)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("canary"), CustomDtcNotes::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CustomDtcNotes {
    Table,
    Id,
    UserId,
    DtcCode,
    VehicleInfo,
    Notes,
    RepairActions,
    CreatedAt,
    UpdatedAt,
}

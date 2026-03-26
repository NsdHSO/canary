use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("canary"), CustomPinouts::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CustomPinouts::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CustomPinouts::UserId).string())
                    .col(
                        ColumnDef::new(CustomPinouts::ConnectorType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CustomPinouts::VehicleInfo)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CustomPinouts::PinMappings)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CustomPinouts::Notes).text())
                    .col(
                        ColumnDef::new(CustomPinouts::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CustomPinouts::UpdatedAt)
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
                    .name("idx_custom_pinouts_user")
                    .table((Alias::new("canary"), CustomPinouts::Table))
                    .col(CustomPinouts::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("canary"), CustomPinouts::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CustomPinouts {
    Table,
    Id,
    UserId,
    ConnectorType,
    VehicleInfo,
    PinMappings,
    Notes,
    CreatedAt,
    UpdatedAt,
}

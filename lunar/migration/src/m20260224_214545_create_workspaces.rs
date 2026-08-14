use sea_orm_migration::{prelude::*, schema::*, sea_orm::DbBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_backend = manager.get_database_backend();

        let mut table = Table::create();
        table
            .table(Workspaces::Table)
            .if_not_exists()
            .col(string(Workspaces::Name))
            .col(string(Workspaces::Description))
            .col(timestamp_with_time_zone(Workspaces::CreatedAt))
            .col(timestamp_with_time_zone(Workspaces::UpdatedAt));

        if db_backend == DbBackend::MySql {
            table.col(
                ColumnDef::new(Workspaces::Identifier)
                    .string_len(36)
                    .not_null()
                    .primary_key(),
            );
        } else {
            table.col(pk_uuid(Workspaces::Identifier));
        }

        manager.create_table(table.to_owned()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Workspaces::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Workspaces {
    Table,
    Identifier,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

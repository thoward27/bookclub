use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Circuits::Table)
                    .col(pk_auto(Circuits::Id))
                    .col(string(Circuits::Title))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Circuits::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Circuits {
    Table,
    Id,
    Title,
}

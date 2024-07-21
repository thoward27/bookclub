use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Picks::Table)
                    .col(pk_auto(Picks::Id))
                    .col(string(Picks::Title))
                    .col(string(Picks::Author))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Picks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Picks {
    Table,
    Id,
    Title,
    Author,
    
}



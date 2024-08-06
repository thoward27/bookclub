use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Books::Table)
                    .col(pk_auto(Books::Id))
                    .col(string_uniq(Books::Title))
                    .col(string(Books::Author))
                    .col(integer(Books::UserId))
                    .col(string(Books::CalibreLink))
                    .col(string_uniq(Books::Isbn10))
                    .col(string_uniq(Books::Isbn13))
                    .col(string(Books::CircuitTitle).default("".to_string()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-books-users")
                            .from(Books::Table, Books::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Books::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Books {
    Table,
    Id,
    Title,
    Author,
    CircuitTitle,
    UserId,
    CalibreLink,
    Isbn10,
    Isbn13,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

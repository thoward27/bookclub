use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Meetings::Table)
                    .col(pk_auto(Meetings::Id))
                    .col(integer(Meetings::BookId))
                    .col(timestamp_with_time_zone(Meetings::Date))
                    .col(string(Meetings::Location))
                    .col(integer_null(Meetings::NextMeetingId))
                    .col(json_binary(Meetings::NextMeetingTemplate).default("{}"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-meetings-books")
                            .from(Meetings::Table, Meetings::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-meetings-next")
                            .from(Meetings::Table, Meetings::NextMeetingId)
                            .to(Meetings::Table, Meetings::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Meetings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Meetings {
    Table,
    Id,
    BookId,
    Date,
    Location,
    NextMeetingId,
    NextMeetingTemplate,
}

#[derive(DeriveIden)]
enum Books {
    Table,
    Id,
}

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Meetings {
    Table,
    Id,
    NextMeetingTemplate,
    NextMeetingId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        //
        // add column
        //
        manager
            .alter_table(
                Table::alter()
                    .table(Meetings::Table)
                    .add_column_if_not_exists(json(Meetings::NextMeetingTemplate).default("{}"))
                    .add_column_if_not_exists(integer_null(Meetings::NextMeetingId))
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk-meetings-next")
                            .from(Meetings::Table, Meetings::NextMeetingId)
                            .to(Meetings::Table, Meetings::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .get_foreign_key(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Meetings::Table)
                    .drop_column(Meetings::NextMeetingTemplate)
                    .drop_foreign_key(Alias::new("fk-meetings-next"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

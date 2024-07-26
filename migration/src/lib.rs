#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20231103_114510_notes;

mod m20240721_010532_picks;
mod m20240726_125323_circuits;
mod m20240726_130832_books;
mod m20240726_131830_meetings;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20231103_114510_notes::Migration),
            Box::new(m20240721_010532_picks::Migration),
            Box::new(m20240726_125323_circuits::Migration),
            Box::new(m20240726_130832_books::Migration),
            Box::new(m20240726_131830_meetings::Migration),
        ]
    }
}

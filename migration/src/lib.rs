#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20240726_130832_books;
mod m20240726_131830_meetings;
mod m20240819_125946_test_set_role;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20240726_130832_books::Migration),
            Box::new(m20240726_131830_meetings::Migration),
            Box::new(m20240819_125946_test_set_role::Migration),
        ]
    }
}

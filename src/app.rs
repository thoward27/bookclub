use std::path::Path;

use async_trait::async_trait;
use axum::{routing::get, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    worker::{AppWorker, Processor},
    Result,
};
use migration::Migrator;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    controllers, initializers,
    models::_entities::{books, circuits, meetings, notes, picks, users},
    tasks,
    workers::downloader::DownloadWorker,
};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    // TODO: This removes the default logging configuration, as documented: https://github.com/loco-rs/loco/blob/master/src/logger.rs
    // fn init_logger(config: &config::Config, _env: &Environment) -> Result<bool> {
    //     tracing_subscriber::Registry::default()
    //     .with(sentry::integrations::tracing::layer()).init();
    //     logger::init(&config.logger);
    //     Ok(true)
    // }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::meetings::routes())
            .add_route(controllers::circuits::routes())
            .add_route(controllers::books::routes())
            .add_route(controllers::home::routes())
            .add_route(controllers::picks::routes())
            .add_route(controllers::notes::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::user::routes())
    }

    async fn after_routes(router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(router.route("/", get(controllers::home::redirect)))
    }

    fn connect_workers<'a>(p: &'a mut Processor, ctx: &'a AppContext) {
        p.register(DownloadWorker::build(ctx));
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::SeedData);
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, users::Entity).await?;
        truncate_table(db, notes::Entity).await?;
        truncate_table(db, picks::Entity).await?;
        truncate_table(db, circuits::Entity).await?;
        truncate_table(db, books::Entity).await?;
        truncate_table(db, meetings::Entity).await?;
        Ok(())
    }

    async fn seed(db: &DatabaseConnection, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(db, &base.join("users.yaml").display().to_string()).await?;
        db::seed::<notes::ActiveModel>(db, &base.join("notes.yaml").display().to_string()).await?;
        db::seed::<picks::ActiveModel>(db, &base.join("picks.yaml").display().to_string()).await?;
        db::seed::<circuits::ActiveModel>(db, &base.join("circuits.yaml").display().to_string())
            .await?;
        db::seed::<books::ActiveModel>(db, &base.join("books.yaml").display().to_string()).await?;
        db::seed::<meetings::ActiveModel>(db, &base.join("meetings.yaml").display().to_string())
            .await?;
        // Taken from: https://wiki.postgresql.org/wiki/Fixing_Sequences
        let update_statements = db.query_all(Statement::from_string(db.get_database_backend(), "
        SELECT 
            'SELECT SETVAL(' ||
                quote_literal(quote_ident(sequence_namespace.nspname) || '.' || quote_ident(class_sequence.relname)) ||
                ', COALESCE(MAX(' ||quote_ident(pg_attribute.attname)|| '), 1) ) FROM ' ||
                quote_ident(table_namespace.nspname)|| '.'||quote_ident(class_table.relname)|| ';'
            AS command
        FROM pg_depend 
            INNER JOIN pg_class AS class_sequence
                ON class_sequence.oid = pg_depend.objid 
                    AND class_sequence.relkind = 'S'
            INNER JOIN pg_class AS class_table
                ON class_table.oid = pg_depend.refobjid
            INNER JOIN pg_attribute 
                ON pg_attribute.attrelid = class_table.oid
                    AND pg_depend.refobjsubid = pg_attribute.attnum
            INNER JOIN pg_namespace as table_namespace
                ON table_namespace.oid = class_table.relnamespace
            INNER JOIN pg_namespace AS sequence_namespace
                ON sequence_namespace.oid = class_sequence.relnamespace
        ORDER BY sequence_namespace.nspname, class_sequence.relname;")).await?;
        for update_statement in update_statements {
            let command = update_statement.try_get::<String>("", "command").unwrap();
            db.execute(Statement::from_string(db.get_database_backend(), command))
                .await?;
        }
        Ok(())
    }
}

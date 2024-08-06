use bookclub::app::App;
use loco_rs::cli;
use loco_rs::environment;
use migration::Migrator;
use std::{borrow::Cow, env};

fn main() -> eyre::Result<()> {
    let _guard = if let Ok(dns) = env::var("SENTRY_DSN") {
        let release: Option<Cow<'static, str>> = if let Some(version) = option_env!("VERSION") {
            Some(version.to_string().into())
        } else {
            sentry::release_name!()
        };
        Some(sentry::init((
            dns,
            sentry::ClientOptions {
                release,
                environment: Some(environment::resolve_from_env().into()),
                traces_sample_rate: 5.0,
                ..Default::default()
            },
        )))
    } else {
        None
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli::main::<App, Migrator>().await })
}

#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn echo(req_body: String) -> String {
    req_body
}

#[debug_handler]
pub async fn hello(State(_ctx): State<AppContext>) -> Result<Response> {
    // do something with context (database, etc)
    format::text("Tester")
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("picks")
        .add("/", get(hello))
        .add("/echo", post(echo))
}

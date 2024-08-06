#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;

pub async fn redirect() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/meetings")
        .body(axum::body::Body::empty())
        .unwrap()
}

pub fn routes() -> Routes {
    Routes::new().prefix("home").add("/", get(redirect))
}

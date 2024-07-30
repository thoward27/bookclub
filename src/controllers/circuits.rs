use crate::models::_entities::circuits;
use crate::views::circuits::CircuitsNav;
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn nav(State(ctx): State<AppContext>) -> Result<Response> {
    let circuits = circuits::Entity::find().all(&ctx.db).await.unwrap();
    Ok(CircuitsNav { circuits }.into_response())
}

pub fn routes() -> Routes {
    Routes::new().prefix("circuits").add("/nav", get(nav))
}

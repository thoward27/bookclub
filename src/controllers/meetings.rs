use axum::debug_handler;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;

use crate::{models::_entities::meetings, views::meetings::MeetingsTemplate};

#[debug_handler]
pub async fn get_meetings(State(ctx): State<AppContext>) -> Result<Response> {
    let meetings = meetings::Entity::find()
        .order_by_desc(meetings::Column::Date)
        .all(&ctx.db)
        .await
        .unwrap();
    Ok(MeetingsTemplate::new(meetings, ctx.db)
        .await
        .into_response())
}

pub fn routes() -> Routes {
    Routes::new().prefix("meetings").add("/", get(get_meetings))
}

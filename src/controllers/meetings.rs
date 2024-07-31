use axum::debug_handler;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::{
    common::middlewares::auth::Auth,
    models::_entities::{books, meetings, users},
    views::meetings::{MeetingDetail, MeetingsTemplate},
};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct MeetingUpdateParams {
    pub location: String,
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<MeetingUpdateParams>,
) -> Result<Response> {
    let meeting = meetings::Entity::find()
        .filter(meetings::Column::Id.eq(id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let meeting = meeting
        .into_active_model()
        .set_location(params.location, &ctx.db)
        .await?;
    Ok(MeetingDetail {
        meeting,
        editor: true,
    }
    .into_response())
}

#[debug_handler]
pub async fn get_one(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let meeting = meetings::Entity::find()
        .filter(meetings::Column::Id.eq(id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let book = meeting
        .find_related(books::Entity)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let user = book
        .find_related(users::Entity)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let editor = auth.user.id != user.id;
    println!(
        "editor: {}; auth user {:?}; book user {:?}",
        editor, auth.user, user
    );
    Ok(MeetingDetail { meeting, editor }.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("meetings")
        .add("/", get(get_meetings))
        .add("/:id", get(get_one))
        .add("/:id", post(update))
}

use axum::debug_handler;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono_tz::Tz;
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
    let meetings: Vec<(meetings::Model, books::Model)> = meetings::Entity::find()
        .order_by_desc(meetings::Column::Date)
        .find_also_related(books::Entity)
        .all(&ctx.db)
        .await
        .unwrap()
        .into_iter()
        .map(|(meeting, book)| (meeting, book.unwrap()))
        .collect();
    Ok(MeetingsTemplate { meetings }.into_response())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeetingUpdateParams {
    pub location: String,
    pub date: String,
    pub timezone: String,
}

impl MeetingUpdateParams {
    pub fn get_datetime(&self) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        let tz: Tz = self.timezone.parse().unwrap();
        let date = NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%dT%H:%M")?;
        let datetime = tz.from_local_datetime(&date).single().unwrap();
        Ok(datetime.fixed_offset())
    }
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
        .set_location(params.location.to_string(), &ctx.db)
        .await?
        .into_active_model()
        .set_date(params.get_datetime().unwrap(), &ctx.db)
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
    let (meeting, book) = meetings::Entity::find()
        .find_also_related(books::Entity)
        .filter(meetings::Column::Id.eq(id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let editor = auth.user.id != book.unwrap().user_id;
    Ok(MeetingDetail { meeting, editor }.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("meetings")
        .add("/", get(get_meetings))
        .add("/:id", get(get_one))
        .add("/:id", post(update))
}

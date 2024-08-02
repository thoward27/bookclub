use axum::debug_handler;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono_tz::Tz;
use loco_rs::prelude::*;
use loco_rs::Error;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::models::meetings::MeetingUpdateParams;
use crate::{
    common::middlewares::auth::Auth,
    models::_entities::{books, meetings, users},
    views::meetings::{MeetingDetail, MeetingsTemplate},
};

use axum_extra::extract::Form;

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
pub struct MeetingFormParams {
    pub location: String,
    pub date: String,
    pub timezone: String,
    #[serde(rename = "user_id")]
    pub user_ids: Vec<String>,
}

impl MeetingFormParams {
    pub fn get_datetime(&self) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        let tz: Tz = self.timezone.parse().unwrap();
        let date = NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%dT%H:%M")?;
        let datetime = tz.from_local_datetime(&date).single().unwrap();
        Ok(datetime.fixed_offset())
    }
}

#[debug_handler]
pub async fn update(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<MeetingFormParams>,
) -> Result<Response> {
    let (meeting, book) = meetings::Entity::find()
        .find_also_related(books::Entity)
        .filter(meetings::Column::Id.eq(id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let book = book.unwrap();
    if auth.user.id != book.user_id {
        return Err(Error::Unauthorized(
            "You are not the owner of this book".to_string(),
        ));
    }
    let meeting = meeting
        .into_active_model()
        .update_from_params(
            MeetingUpdateParams {
                location: params.location.to_string(),
                date: params.get_datetime().unwrap(),
                order: params
                    .user_ids
                    .iter()
                    .map(|id| id.parse().unwrap())
                    .collect(),
            },
            &ctx.db,
        )
        .await?;
    Ok(MeetingDetail::new(meeting, book, auth.user, &ctx.db)
        .await
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
    let book = book.unwrap();
    Ok(MeetingDetail::new(meeting, book, auth.user, &ctx.db)
        .await
        .into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("meetings")
        .add("/", get(get_meetings))
        .add("/:id", get(get_one))
        .add("/:id", post(update))
}

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
use crate::views::meetings::MeetingFormTemplate;
use crate::{
    common::middlewares::auth::Auth,
    models::_entities::{books, meetings, users},
    views::meetings::{MeetingTemplate, MeetingsTemplate},
};

use axum_extra::extract::Form;

#[debug_handler]
pub async fn get_many(State(ctx): State<AppContext>) -> Result<Response> {
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

#[debug_handler]
pub async fn get_one(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    Ok(MeetingTemplate::new(id, auth.user, &ctx.db).await)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeetingFormParams {
    pub location: String,
    pub date: String,
    pub timezone: String,
    #[serde(rename = "user_id")]
    pub user_ids: Vec<String>,
    #[serde(rename = "bench_user_id", default = "Vec::new")]
    pub bench_user_ids: Vec<String>,
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
pub async fn update_one(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<MeetingFormParams>,
) -> Result<impl IntoResponse> {
    let (meeting, book) = meetings::Model::get_with_book(&ctx.db, id).await.unwrap();
    if !meeting.is_editable(&auth.user, &book) {
        return Err(Error::Unauthorized(
            "You cannot edit this meeting".to_string(),
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
    Ok(MeetingFormTemplate::new(meeting, book, auth.user, &ctx.db).await)
}

#[debug_handler]
pub async fn create_next(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let (current_meeting, current_book) =
        meetings::Model::get_with_book(&ctx.db, id).await.unwrap();
    meetings::Model::create_next(current_meeting.clone(), current_book.clone(), &ctx.db).await?;
    let current_meeting = meetings::Entity::find()
        .filter(meetings::Column::Id.eq(id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    Ok(MeetingFormTemplate::new(current_meeting, current_book, auth.user, &ctx.db).await)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("meetings")
        .add("/", get(get_many))
        .add("/:id", get(get_one))
        .add("/:id", post(update_one))
        .add("/:id/next", post(create_next))
}

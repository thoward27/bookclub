use axum::debug_handler;
use loco_rs::prelude::*;

use crate::common::middlewares::auth::Auth;
use crate::{models::_entities::users, views::user::CurrentResponse};

#[debug_handler]
async fn current(auth: Auth<users::Model>, State(_ctx): State<AppContext>) -> Result<Response> {
    let user = auth.user;
    format::json(CurrentResponse::new(&user))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/user")
        .add("/current", get(current))
}

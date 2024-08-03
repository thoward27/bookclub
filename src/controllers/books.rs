use crate::common::middlewares::auth::Auth;
use crate::models::_entities::{books, circuits, users};
use crate::models::books::BookUpdateParams;
use crate::views::books::{BookFormTemplate, BookTemplate, BooksTemplate};
use axum::debug_handler;
use loco_rs::prelude::*;
use migration::extension::postgres::PgExpr;
use sea_orm::sea_query::Expr;
use sea_orm::QueryOrder;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[debug_handler]
async fn get_books_by_circuit(
    _auth: Auth<users::Model>,
    State(ctx): State<AppContext>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse> {
    let circuit = circuits::Entity::find()
        .filter(Expr::col(circuits::Column::Title).ilike(name))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let books = books::Entity::find()
        .filter(books::Column::CircuitId.eq(circuit.id))
        .order_by_desc(books::Column::Id)
        .all(&ctx.db)
        .await
        .unwrap();
    Ok(BooksTemplate::new(books, ctx.db).await)
}

#[debug_handler]
async fn get_books(
    _auth: Auth<users::Model>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let books = books::Entity::find()
        .order_by_desc(books::Column::Id)
        .all(&ctx.db)
        .await
        .unwrap();
    Ok(BooksTemplate::new(books, ctx.db).await)
}

#[debug_handler]
async fn get_one(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    Ok(BookTemplate::new(id, auth.user, &ctx.db).await)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookFormParams {
    pub title: String,
    pub author: String,
}

#[debug_handler]
async fn update_one(
    auth: Auth<users::Model>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<BookFormParams>,
) -> Result<impl IntoResponse> {
    let (book, meeting) = books::Model::get_with_meeting(&ctx.db, id).await.unwrap();
    let is_editable = book.is_editable(&auth.user, &meeting);
    if !is_editable {
        return Err(Error::Unauthorized("You cannot edit this book".to_string()));
    }
    let book = book
        .into_active_model()
        .update_from_params(
            BookUpdateParams {
                title: params.title.to_string(),
                author: params.author.to_string(),
            },
            &ctx.db,
        )
        .await
        .unwrap();
    Ok(BookFormTemplate::new(book, meeting, auth.user, &ctx.db).await)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("books")
        .add("/", get(get_books))
        .add("/circuit/:name", get(get_books_by_circuit))
        .add("/:id", get(get_one))
        .add("/:id", post(update_one))
}

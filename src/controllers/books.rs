use crate::common::middlewares::auth::Auth;
use crate::models::_entities::{books, circuits, users};
use crate::views::books::BooksTemplate;
use axum::debug_handler;
use loco_rs::prelude::*;
use migration::extension::postgres::PgExpr;
use sea_orm::sea_query::Expr;
use sea_orm::QueryOrder;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("books")
        .add("/", get(get_books))
        .add("/circuit/:name", get(get_books_by_circuit))
}

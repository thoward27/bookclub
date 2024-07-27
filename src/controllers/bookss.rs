use crate::models::_entities::{books, circuits, users};
use crate::views::books::{BookTemplate, BooksTemplate};
use axum::debug_handler;
use futures::stream::{self, StreamExt};
use loco_rs::prelude::*;

#[debug_handler]
async fn get_books(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let books = books::Entity::find().all(&ctx.db).await.unwrap();
    let books = stream::iter(books)
        .then(|book| async {
            let circuit = book
                .find_related(circuits::Entity)
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
            BookTemplate {
                title: book.title,
                author: book.author,
                circuit: circuit.title,
                username: user.name,
                isbn10: book.isbn10,
                isbn13: book.isbn13,
            }
        })
        .collect::<Vec<BookTemplate>>()
        .await;
    Ok(BooksTemplate { books })
}

pub fn routes() -> Routes {
    Routes::new().prefix("books").add("/", get(get_books))
}

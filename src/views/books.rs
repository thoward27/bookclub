use crate::models::_entities::{books, circuits, users};
use askama_axum::Template;
use futures::stream::{self, StreamExt};
use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/book.html", escape = "none")]
pub struct BookTemplate {
    pub title: String,
    pub author: String,
    pub circuit: String,
    pub username: String,
    pub isbn10: String,
    pub isbn13: String,
}

impl ViewRenderer for BookTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "books.html", escape = "none")]
pub struct BooksTemplate {
    pub books: Vec<BookTemplate>,
}

impl ViewRenderer for BooksTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl BooksTemplate {
    pub async fn new(books: Vec<books::Model>, ctx: DatabaseConnection) -> Self {
        Self {
            books: stream::iter(books)
                .then(|book| async {
                    let circuit = book
                        .find_related(circuits::Entity)
                        .one(&ctx)
                        .await
                        .unwrap()
                        .unwrap();
                    let user = book
                        .find_related(users::Entity)
                        .one(&ctx)
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
                .await,
        }
    }
}

use crate::models::_entities::{books, meetings, users};
use askama_axum::Template;
use futures::stream::{self, StreamExt};
use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "components/book_circuits_nav.html", escape = "none")]
pub struct BooksCircuitNav {
    pub circuits: Vec<String>,
}

impl ViewRenderer for BooksCircuitNav {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "components/book_card.html", escape = "none")]
pub struct BookCardTemplate {
    pub title: String,
    pub author: String,
    pub circuit: String,
    pub username: String,
    pub isbn10: String,
    pub isbn13: String,
}

impl ViewRenderer for BookCardTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "books.html", escape = "none")]
pub struct BooksTemplate {
    pub books: Vec<BookCardTemplate>,
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
                    let user = book
                        .find_related(users::Entity)
                        .one(&ctx)
                        .await
                        .unwrap()
                        .unwrap();
                    BookCardTemplate {
                        title: book.title,
                        author: book.author,
                        circuit: book.circuit_title,
                        username: user.name,
                        isbn10: book.isbn10,
                        isbn13: book.isbn13,
                    }
                })
                .collect::<Vec<BookCardTemplate>>()
                .await,
        }
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "book.html", escape = "none")]
pub struct BookTemplate {
    pub book: books::Model,

    pub form: BookFormTemplate,
}

impl ViewRenderer for BookTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl BookTemplate {
    pub async fn new(id: i32, user: users::Model, db: &DatabaseConnection) -> Self {
        let (book, meeting) = books::Model::get_with_meeting(db, id).await.unwrap();
        let form = BookFormTemplate::new(book.clone(), meeting, user, db).await;
        Self { book, form }
    }
}

#[derive(Template, Debug, Clone)]
#[template(path = "components/book_form.html", escape = "none")]
pub struct BookFormTemplate {
    pub book: books::Model,

    pub editable: bool,
}

impl ViewRenderer for BookFormTemplate {
    fn render<S: Serialize>(&self, _key: &str, _data: S) -> Result<String> {
        Ok(Template::render(self).unwrap())
    }
}

impl BookFormTemplate {
    pub async fn new(
        book: books::Model,
        meeting: meetings::Model,
        user: users::Model,
        _db: &DatabaseConnection,
    ) -> Self {
        let editable = book.is_editable(&user, &meeting);
        Self { book, editable }
    }
}

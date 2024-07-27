use askama_axum::Template;
use loco_rs::prelude::*;
use serde::Serialize;
use std::vec::Vec;

#[derive(Template, Debug, Clone)]
#[template(path = "book.html", escape = "none")]
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

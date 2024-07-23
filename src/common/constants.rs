use std::env;
use std::fmt;
use std::fmt::Display;
use std::sync::LazyLock;

pub const DEFAULT_TITLE: &str = "Inkwell Collective";

#[derive(Debug)]
pub struct Constant<T> {
    pub value: LazyLock<T>,
}

impl<T: std::fmt::Display> Display for Constant<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self.value)
    }
}

pub static TITLE: Constant<String> = Constant {
    value: LazyLock::new(|| {
        env::var("BOOKCLUB_TITLE").unwrap_or_else(|_| DEFAULT_TITLE.to_string())
    }),
};

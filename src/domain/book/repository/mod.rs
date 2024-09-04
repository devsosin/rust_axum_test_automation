pub mod repository_impl;
pub(super) mod save;

use axum::Error;

use super::{dto::request::NewBook, entity::Book};

pub trait BookRepository: Send + Sync {
    async fn get_book(&self, id: i64) -> Result<Option<Book>, Error>;
    async fn save_book(&self, new_book: &NewBook) -> Result<(), Error>;
    async fn delete_book(&self, id: i64) -> Result<(), Error>;
}

pub use repository_impl::BookRepositoryImpl;
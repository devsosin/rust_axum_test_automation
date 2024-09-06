pub mod repository_impl;
pub(super) mod save;

use axum::async_trait;

use super::{dto::request::NewBook, entity::Book};

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_book(&self, id: i32) -> Result<Option<Book>, String>;
    async fn save_book(&self, new_book: &NewBook, type_id: i16) -> Result<i32, String>;
    async fn delete_book(&self, id: i32) -> Result<(), String>;
}

pub use repository_impl::BookRepositoryImpl;

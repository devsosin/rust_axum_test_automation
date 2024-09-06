pub mod repository_impl;

pub(super) mod get_book_type;
pub(super) mod save;

use axum::async_trait;

use super::entity::{Book, BookType};

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_book(&self, id: i32) -> Result<Book, String>;
    async fn save_book(&self, name: &str, type_id: i16) -> Result<i32, String>;
    async fn delete_book(&self, id: i32) -> Result<(), String>;
}

#[async_trait]
pub trait BookTypeRepository: Send + Sync {
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String>;
}

pub use repository_impl::BookRepositoryImpl;
pub use repository_impl::BookTypeRepositoryImpl;

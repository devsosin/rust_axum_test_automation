pub(super) mod create;
pub mod usecase_impl;

use axum::{async_trait, Error};

use super::{dto::request::NewBook, entity::Book};

#[async_trait]
pub trait BookUsecase: Send + Sync {
    async fn create_book(&self, new_book: &NewBook, type_id: i16) -> Result<i32, String>;
    async fn read_book(&self, id: i32) -> Result<Option<Book>, Error>;
    async fn update_book(&self, id: i32) -> Result<Option<Book>, Error>;
    async fn delete_book(&self, id: i32) -> Result<(), Error>;
}

pub use usecase_impl::BookUsecaseImpl;

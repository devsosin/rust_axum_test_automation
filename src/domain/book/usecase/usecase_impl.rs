use std::sync::Arc;

use crate::domain::book::repository::BookRepositoryImpl;

use super::{create::create_book, Book, BookUsecase, NewBook};
use axum::{async_trait, Error};

pub struct BookUsecaseImpl {
    repository: Arc<BookRepositoryImpl>,
}

impl BookUsecaseImpl {
    pub fn new(repository: Arc<BookRepositoryImpl>) -> Self {
        Self { repository }
    }

    pub fn get_repository(&self) -> Arc<BookRepositoryImpl> {
        self.repository.clone()
    }
}

#[async_trait]
impl BookUsecase for BookUsecaseImpl {
    async fn create_book(&self, new_book: &NewBook, type_id: i16) -> Result<i32, String> {
        create_book(self.get_repository(), new_book, type_id).await
    }

    async fn read_book(&self, id: i32) -> Result<Option<Book>, Error> {
        todo!()
    }

    async fn update_book(&self, id: i32) -> Result<Option<Book>, Error> {
        todo!()
    }

    async fn delete_book(&self, id: i32) -> Result<(), Error> {
        todo!()
    }
}

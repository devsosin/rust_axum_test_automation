use std::sync::Arc;

use crate::domain::book::repository::{BookRepository, BookTypeRepository};

use super::{create::create_book, Book, BookUsecase, NewBook};
use axum::{async_trait, Error};

pub struct BookUsecaseImpl<T: BookRepository, U: BookTypeRepository> {
    repository: Arc<T>,
    type_repo: Arc<U>,
}

impl<T, U> BookUsecaseImpl<T, U>
where
    T: BookRepository,
    U: BookTypeRepository,
{
    pub fn new(repository: Arc<T>, type_repo: Arc<U>) -> Self {
        Self {
            repository,
            type_repo,
        }
    }

    pub fn get_repository(&self) -> Arc<T> {
        self.repository.clone()
    }

    pub fn get_type_repo(&self) -> Arc<U> {
        self.type_repo.clone()
    }
}

#[async_trait]
impl<T, U> BookUsecase for BookUsecaseImpl<T, U>
where
    T: BookRepository,
    U: BookTypeRepository,
{
    async fn create_book(&self, new_book: &NewBook) -> Result<i32, String> {
        create_book(self.get_repository(), self.get_type_repo(), new_book).await
    }

    async fn read_book(&self, id: i32) -> Result<Book, Error> {
        todo!()
    }

    async fn update_book(&self, id: i32) -> Result<Book, Error> {
        todo!()
    }

    async fn delete_book(&self, id: i32) -> Result<(), Error> {
        todo!()
    }
}

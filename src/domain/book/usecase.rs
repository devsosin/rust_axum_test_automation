use std::sync::Arc;

use sqlx::PgPool;

use super::{dto::request::NewBook, entity::Book, repository::{self, BookRepository, BookRepositoryImpl}};

#[derive(Clone)]
pub struct BookUsecase {
    repository: Arc<BookRepositoryImpl>,
}

impl BookUsecase {
    pub fn new(repository: Arc<BookRepositoryImpl>) -> Self {
        Self { repository }
    }
    
    // pub fn repository(&self) -> &dyn BookRepository {
    //     &self.repository
    // }

    pub async fn create_book(&self, pool: &PgPool, new_book: &NewBook, type_id: i16) -> Result<bool, String> {
        match self.repository.save_book(new_book).await {
            Ok(result) => {
                // Custom Exception
                // Error::BookAlreadyExists
                Ok(true)
            },
            Err(e) => {
                Err(e.to_string())
            }
        }

        // repository::save::save_book(pool, new_book, type_id).await
    }
}
use std::sync::Arc;

use axum::Error;
use sqlx::PgPool;
use super::{Book, BookRepository, NewBook};

pub struct BookRepositoryImpl {
    pool: Arc<PgPool>
}

impl BookRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl BookRepository for BookRepositoryImpl {    
    async fn get_book(&self, id: i64) -> Result<Option<Book>, Error> {
        // todo!()
        Err(Error::new(""))
    }

    async fn save_book(&self, new_book: &NewBook, type_id: i16) -> Result<(), Error> {
        // todo!()
        Err(Error::new(""))
    }

    async fn delete_book(&self, id: i64) -> Result<(), Error> {
        // todo!()
        Err(Error::new(""))
    }
}
use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::domain::book::{dto::request::NewBook, entity::Book};

use super::{save::save_book, BookRepository};

pub struct BookRepositoryImpl {
    pool: Arc<PgPool>,
}

impl BookRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn get_book(&self, id: i32) -> Result<Option<Book>, String> {
        todo!()
    }

    // String 자리에 Error -> CUSTOM Error 전달
    async fn save_book(&self, new_book: &NewBook, type_id: i16) -> Result<i32, String> {
        save_book(&self.pool, new_book, type_id).await
    }

    async fn delete_book(&self, id: i32) -> Result<(), String> {
        todo!()
    }
}

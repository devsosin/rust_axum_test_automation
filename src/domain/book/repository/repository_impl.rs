use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::domain::book::entity::{Book, BookType};

use super::{
    get_book_type::get_book_type_by_name, save::save_book, BookRepository, BookTypeRepository,
};

pub struct BookRepositoryImpl {
    pool: Arc<PgPool>,
}

pub struct BookTypeRepositoryImpl {
    pool: Arc<PgPool>,
}

impl BookRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl BookTypeRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn get_book(&self, id: i32) -> Result<Book, String> {
        todo!()
    }

    async fn get_books(&self) -> Result<Vec<Book>, String> {
        todo!()
    }

    // String 자리에 Error -> CUSTOM Error 전달
    async fn save_book(&self, name: &str, type_id: i16) -> Result<i32, String> {
        save_book(&self.pool, name, type_id).await
    }

    async fn update_book(&self, id: i32, name: &str) -> Result<(), String> {
        todo!()
    }

    async fn delete_book(&self, id: i32) -> Result<(), String> {
        todo!()
    }
}

#[async_trait]
impl BookTypeRepository for BookTypeRepositoryImpl {
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String> {
        get_book_type_by_name(&self.pool, name).await
    }

    async fn get_book_types(&self) -> Result<Vec<BookType>, String> {
        todo!()
    }
}

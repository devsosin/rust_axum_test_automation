use sqlx::PgPool;

use super::{dto::request::NewBook, entity::Book, repository};

pub async fn create_book(pool: &PgPool, new_book: &NewBook, type_id: i16) -> Result<bool, String> {
    repository::create_book(pool, new_book, type_id).await
}
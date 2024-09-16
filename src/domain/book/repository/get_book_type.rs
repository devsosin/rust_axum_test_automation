use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::book::entity::BookType, global::errors::CustomError};

pub struct GetBookTypeRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetBookTypeRepo: Send + Sync {
    async fn get_book_types(&self) -> Result<Vec<BookType>, Arc<CustomError>>;
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, Arc<CustomError>>;
}

impl GetBookTypeRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetBookTypeRepo for GetBookTypeRepoImpl {
    async fn get_book_types(&self) -> Result<Vec<BookType>, Arc<CustomError>> {
        get_book_types(&self.pool).await
    }
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, Arc<CustomError>> {
        get_book_type_by_name(&self.pool, name).await
    }
}

async fn get_book_types(pool: &PgPool) -> Result<Vec<BookType>, Arc<CustomError>> {
    let rows: Vec<BookType> = sqlx::query_as::<_, BookType>(r"SELECT * FROM tb_book_type")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetBookTypes): {:?}", &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("Book Type".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    Ok(rows)
}

pub async fn get_book_type_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<BookType, Arc<CustomError>> {
    let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            // row not found error
            let err_msg = format!("Error(GetBookTypeByName {}): {:?}", name, e);
            tracing::error!("{:?}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("Book Type".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    Ok(row)
}

#[cfg(test)]
mod tests {

    use crate::{
        config::database::create_connection_pool,
        domain::book::{entity::BookType, repository::get_book_type::get_book_types},
    };

    use super::get_book_type_by_name;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false);
    }

    #[tokio::test]
    async fn check_get_book_types_success() {
        // Arrange
        let pool = create_connection_pool().await;

        // Act
        let result = get_book_types(&pool).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), 3); // 개인, 커플 기업
    }

    #[tokio::test]
    async fn check_get_book_type_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "개인";

        // Act
        let result = get_book_type_by_name(&pool, name).await;
        // 결과 제대로 받아왔는지 체크
        assert!(result.is_ok());
        let book_type = result.unwrap();
        let type_id = book_type.get_id();

        // Assert
        let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
            .bind(name)
            .fetch_one(&pool)
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert_eq!(row.get_id(), type_id);
    }

    #[tokio::test]
    async fn check_book_type_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "없는 이름";

        // Act
        let result = get_book_type_by_name(&pool, name).await;

        // Assert: 존재하지 않는 것은 Err(RowNotFound) 반환
        assert!(result.is_err());
    }
}

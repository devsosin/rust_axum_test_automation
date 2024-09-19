use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::category::entity::BaseCategory, global::errors::CustomError};

pub struct GetCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetCategoryRepo: Send + Sync {
    async fn get_list(
        &self,
        user_id: i32,
        book_id: i32,
        is_record: bool,
    ) -> Result<Vec<BaseCategory>, Box<CustomError>>;
}

impl GetCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl GetCategoryRepo for GetCategoryRepoImpl {
    async fn get_list(
        &self,
        user_id: i32,
        book_id: i32,
        is_record: bool,
    ) -> Result<Vec<BaseCategory>, Box<CustomError>> {
        get_list(&self.pool, user_id, book_id, is_record).await
    }
}

pub async fn get_list(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    is_record: bool,
) -> Result<Vec<BaseCategory>, Box<CustomError>> {
    let result = sqlx::query_as::<_, BaseCategory>(
        "
        WITH AuthorityCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM tb_user_book_role AS br
                WHERE br.user_id = $1 AND br.book_id = $2
            ) AS is_authorized
        )
        SELECT * 
        FROM tb_base_category 
        WHERE (book_id = $2 OR book_id IS NULL) 
            AND (SELECT is_authorized FROM AuthorityCheck) = true;
        ",
    )
    .bind(user_id)
    .bind(book_id)
    .bind(is_record)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetList {}): {:?}", book_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Box::new(err)
    })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{config::database::create_connection_pool, global::errors::CustomError};

    use super::get_list;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_base_list_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = 1;
        let is_record = true;

        // Act
        let result = get_list(&pool, user_id, book_id, is_record).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert!(result.unwrap().len() >= 4)
    }

    // testing?
    async fn check_book_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = -32;
        let is_record = true;

        // Act
        let result = get_list(&pool, user_id, book_id, is_record).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    // testing?
    async fn check_unauthorized() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 3;
        let book_id = 1;
        let is_record = true;

        // Act
        let result = get_list(&pool, user_id, book_id, is_record).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}

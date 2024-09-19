use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::category::entity::SubCategory, global::errors::CustomError};

pub struct GetCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetCategoryRepo: Send + Sync {
    async fn get_list(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<Vec<SubCategory>, Box<CustomError>>;
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
        base_id: i16,
    ) -> Result<Vec<SubCategory>, Box<CustomError>> {
        get_list(&self.pool, user_id, base_id).await
    }
}

pub async fn get_list(
    pool: &PgPool,
    user_id: i32,
    base_id: i16,
) -> Result<Vec<SubCategory>, Box<CustomError>> {
    let result = sqlx::query_as::<_, SubCategory>(
        "
        WITH BaseExists AS (
            SELECT id, book_id
            FROM tb_base_category
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM BaseExists as be
                LEFT JOIN tb_book AS b ON b.id = be.book_id
                LEFT JOIN tb_user_book_role AS br ON b.id = br.book_id
                WHERE br.user_id = $1 OR be.book_id IS NULL
            ) AS is_authorized
        )
        SELECT *
        FROM tb_sub_category
        WHERE base_id = $2 AND
            (SELECT is_authorized FROM AuthorityCheck) = true;
        ",
    )
    .bind(user_id)
    .bind(base_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetSubList {}): {:?}", base_id, &e);
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
    use crate::config::database::create_connection_pool;

    use super::get_list;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_sub_list_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = 8;

        // Act
        let result = get_list(&pool, user_id, base_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert!(result.unwrap().len() >= 5)
    }
}

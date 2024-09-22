use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::{constants::DeleteResult, errors::CustomError};

pub struct DeleteCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait DeleteCategoryRepo: Send + Sync {
    async fn delete_base_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<(), Box<CustomError>>;
}

impl DeleteCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl DeleteCategoryRepo for DeleteCategoryRepoImpl {
    async fn delete_base_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<(), Box<CustomError>> {
        _delete_base_category(&self.pool, user_id, base_id).await
    }
}

async fn _delete_base_category(
    pool: &PgPool,
    user_id: i32,
    base_id: i16,
) -> Result<(), Box<CustomError>> {
    let result = sqlx::query_as::<_, DeleteResult>(
        "
        WITH BaseCategoryExists AS (
            SELECT id, book_id
            FROM tb_base_category
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM BaseCategoryExists AS be
                JOIN tb_book AS b ON b.id = be.book_id
                JOIN tb_user_book_role AS br ON b.id = br.book_id
                WHERE br.user_id = $1 AND br.role = 'owner'
            ) AS is_authorized
        ),
        DeleteBaseCategory AS (
            DELETE FROM tb_base_category
            WHERE id = $2
                AND (SELECT is_authorized FROM AuthorityCheck) = true
            RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM BaseCategoryExists) AS is_exist,
            (SELECT is_authorized FROM AuthorityCheck) AS is_authorized,
            (SELECT COUNT(*) FROM DeleteBaseCategory) AS delete_count;
        ",
    )
    .bind(user_id)
    .bind(base_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(DeleteBaseCategory {}), {:?}", base_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if !result.get_exist() {
        return Err(Box::new(CustomError::NotFound("BaseCategory".to_string())));
    } else if !result.get_authorized() {
        return Err(Box::new(CustomError::Unauthorized(
            "BaseCategoryRole".to_string(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::category::{entity::BaseCategory, repository::save_base::save_base_category},
        global::errors::CustomError,
    };

    use super::_delete_base_category;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_delete_base_category_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_category = BaseCategory::new(
            1,
            1,
            true,
            true,
            "사업수입".to_string(),
            "123456".to_string(),
        );
        let base_id = save_base_category(&pool, user_id, base_category)
            .await
            .unwrap();

        // Act
        let result = _delete_base_category(&pool, user_id, base_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = sqlx::query("SELECT * FROM tb_base_category WHERE id = $1")
            .bind(base_id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(row.is_none())
    }

    #[tokio::test]
    async fn check_base_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let user_id = 1;
        let base_id = -32;

        // Act
        let result = _delete_base_category(&pool, user_id, base_id).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let pool = create_connection_pool().await;
        // ref) init.sql
        let user_id = 2;
        let base_id = 11;

        // Act
        let result = _delete_base_category(&pool, user_id, base_id).await;

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

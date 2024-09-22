use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::{constants::DeleteResult, errors::CustomError};

pub struct DeleteCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait DeleteCategoryRepo: Send + Sync {
    async fn delete_sub_category(&self, user_id: i32, sub_id: i32) -> Result<(), Box<CustomError>>;
}

impl DeleteCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl DeleteCategoryRepo for DeleteCategoryRepoImpl {
    async fn delete_sub_category(&self, user_id: i32, sub_id: i32) -> Result<(), Box<CustomError>> {
        _delete_sub_category(&self.pool, user_id, sub_id).await
    }
}

async fn _delete_sub_category(
    pool: &PgPool,
    user_id: i32,
    sub_id: i32,
) -> Result<(), Box<CustomError>> {
    let result = sqlx::query_as::<_, DeleteResult>(
        "
        WITH SubCategoryExists AS (
            SELECT id, base_id
            FROM tb_sub_category
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM SubCategoryExists AS s
                JOIN tb_base_category AS bc ON bc.id = s.base_id
                JOIN tb_book AS b ON b.id = bc.book_id
                JOIN tb_user_book_role AS br ON b.id = br.user_id
                WHERE br.user_id = $1 AND br.role = 'owner'
            ) AS is_authorized
        ),
        DeleteSubCategory AS (
            DELETE FROM tb_sub_category
            WHERE id = $2
                AND (SELECT is_authorized FROM AuthorityCheck) = true
            RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM SubCategoryExists) AS is_exist,
            (SELECT is_authorized FROM AuthorityCheck),
            (SELECT COUNT(*) FROM DeleteSubCategory) AS delete_count;
        ",
    )
    .bind(user_id)
    .bind(sub_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(DeleteSubCategory {}): {:?}", sub_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Box::new(err)
    })?;

    if !result.get_exist() {
        return Err(Box::new(CustomError::NotFound("SubCategory".to_string())));
    } else if !result.get_authorized() {
        return Err(Box::new(CustomError::Unauthorized(
            "SubCategoryRole".to_string(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::category::{entity::SubCategory, repository::save_sub::save_sub_category},
        global::errors::CustomError,
    };

    use super::_delete_sub_category;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_delete_sub_category_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = 11;
        let sub_category = SubCategory::new(base_id, "삭제용 서브 카테고리".to_string());

        let sub_id = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();

        // Act
        let result = _delete_sub_category(&pool, user_id, sub_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = sqlx::query("SELECT * FROM tb_sub_category WHERE id = $1")
            .bind(sub_id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(row.is_none())
    }

    #[tokio::test]
    async fn check_sub_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let sub_id = -32;

        // Act
        let result = _delete_sub_category(&pool, user_id, sub_id).await;

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
    async fn check_unahorized() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = 11;
        let sub_category = SubCategory::new(base_id, "삭제 권한 없는 서브 카테고리".to_string());

        let sub_id = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();
        let user_id = 2;

        // Act
        let result = _delete_sub_category(&pool, user_id, sub_id).await;

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

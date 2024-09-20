use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::{constants::UpdateResult, errors::CustomError};

pub struct UpdateCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait UpdateCategoryRepo: Send + Sync {
    async fn update_sub_category(
        &self,
        user_id: i32,
        sub_id: i32,
        name: String,
    ) -> Result<(), Box<CustomError>>;
}

impl UpdateCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl UpdateCategoryRepo for UpdateCategoryRepoImpl {
    async fn update_sub_category(
        &self,
        user_id: i32,
        sub_id: i32,
        name: String,
    ) -> Result<(), Box<CustomError>> {
        _update_sub_category(&self.pool, user_id, sub_id, name).await
    }
}

async fn _update_sub_category(
    pool: &PgPool,
    user_id: i32,
    sub_id: i32,
    name: String,
) -> Result<(), Box<CustomError>> {
    let result = sqlx::query_as::<_, UpdateResult>(
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
                JOIN tb_book AS b ON bc.book_id = b.id
                JOIN tb_user_book_role AS br ON b.id = br.book_id
                WHERE br.user_id = $1 AND br.role != 'viewer'
            ) AS is_authorized
        ),
        DuplicateCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM tb_sub_category AS s
                WHERE s.id != $2 AND s.name = $3
                    AND s.base_id = (SELECT base_id FROM SubCategoryExists)
            ) AS is_duplicated
        ),
        UpdateSubCategory AS (
            UPDATE tb_sub_category SET name = $3
            WHERE id = $2
                AND (SELECT is_authorized FROM AuthorityCheck) = true
                AND (SELECT is_duplicated FROM DuplicateCheck) = false
            RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM SubCategoryExists) AS is_exist,
            (SELECT is_authorized FROM AuthorityCheck) AS is_authorized,
            (SELECT is_duplicated FROM DuplicateCheck) AS is_duplicated,
            (SELECT COUNT(*) FROM UpdateSubCategory) AS update_count;
        ",
    )
    .bind(user_id)
    .bind(sub_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(UpdateSubCategory {}): {:?}", sub_id, &e);
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
    } else if result.get_duplicated() {
        return Err(Box::new(CustomError::Duplicated("SubCategory".to_string())));
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

    use super::_update_sub_category;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_update_sub_category_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = 11;
        let sub_category = SubCategory::new(base_id, "수정용 서브 카테고리".to_string());
        let sub_id = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();

        let target_name = "수정한 서브 카테고리 이름";

        // Act
        let result = _update_sub_category(&pool, user_id, sub_id, target_name.to_string()).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = sqlx::query_as::<_, SubCategory>("SELECT * FROM tb_sub_category WHERE id = $1")
            .bind(sub_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(target_name, row.get_name())
    }

    #[tokio::test]
    async fn check_sub_category_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let sub_id = -32;

        let target_name = "수정불가 서브 카테고리";

        // Act
        let result = _update_sub_category(&pool, user_id, sub_id, target_name.to_string()).await;

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

        let user_id = 1;
        let base_id = 11;
        let sub_category = SubCategory::new(base_id, "권한체크용 서브 카테고리".to_string());
        let sub_id = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();

        let user_id = 2;

        let target_name = "변경되지 않을 서브 카테고리";

        // Act
        let result = _update_sub_category(&pool, user_id, sub_id, target_name.to_string()).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = 11;
        let sub_category = SubCategory::new(base_id, "수정 중복체크 서브 카테고리".to_string());
        let _ = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();

        let sub_category = SubCategory::new(base_id, "수정 대상".to_string());
        let sub_id = save_sub_category(&pool, user_id, sub_category)
            .await
            .unwrap();

        let target_name = "수정 중복체크 서브 카테고리";

        // Act
        let result = _update_sub_category(&pool, user_id, sub_id, target_name.to_string()).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}

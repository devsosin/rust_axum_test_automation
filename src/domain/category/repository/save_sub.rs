use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::category::entity::SubCategory, global::errors::CustomError};

pub struct SaveCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveCategoryRepo: Send + Sync {
    async fn save_sub_category(
        &self,
        user_id: i32,
        sub_category: SubCategory,
    ) -> Result<i32, Box<CustomError>>;
}

impl SaveCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveCategoryRepo for SaveCategoryRepoImpl {
    async fn save_sub_category(
        &self,
        user_id: i32,
        sub_category: SubCategory,
    ) -> Result<i32, Box<CustomError>> {
        save_sub_category(&self.pool, user_id, sub_category).await
    }
}

#[derive(Debug, sqlx::FromRow)]
struct InsertResult {
    id: Option<i32>,
    is_exist: bool,
    is_authorized: bool,
    is_duplicated: bool,
}

pub async fn save_sub_category(
    pool: &PgPool,
    user_id: i32,
    sub_category: SubCategory,
) -> Result<i32, Box<CustomError>> {
    let result = sqlx::query_as::<_, InsertResult>(
        r"
        WITH BaseExists AS (
            SELECT id, book_id
            FROM tb_base_category
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT be.id AS base_id
            FROM BaseExists AS be
            JOIN tb_book AS b ON be.book_id = b.id
            JOIN tb_user_book_role AS br ON br.book_id = b.id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ),
        DuplicateCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM AuthorityCheck AS a
                JOIN tb_sub_category AS s ON a.base_id = s.base_id
                WHERE s.name = $3
            ) AS is_duplicated
        ),
        InsertSubCategory AS (
            INSERT INTO tb_sub_category (base_id, name)
            SELECT base_id, $3
            FROM AuthorityCheck
            WHERE (SELECT is_duplicated FROM DuplicateCheck) = false
            RETURNING id
        )
        SELECT 
            EXISTS (SELECT 1 FROM BaseExists) AS is_exist,
            EXISTS (SELECT 1 FROM AuthorityCheck) AS is_authorized,
            (SELECT is_duplicated FROM DuplicateCheck),
            (SELECT id FROM InsertSubCategory);
        ",
    )
    .bind(user_id)
    .bind(sub_category.get_base_id())
    .bind(sub_category.get_name())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveSubCategory): {:?}", &e);
        tracing::error!(err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Box::new(err)
    })?;

    if !result.is_exist {
        return Err(Box::new(CustomError::NotFound("BaseCategory".to_string())));
    } else if !result.is_authorized {
        return Err(Box::new(CustomError::Unauthorized("BookRole".to_string())));
    } else if result.is_duplicated {
        return Err(Box::new(CustomError::Duplicated("SubCategory".to_string())));
    }

    Ok(result.id.unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::category::{
            entity::{BaseCategory, SubCategory},
            repository::save_base::save_base_category,
        },
        global::errors::CustomError,
    };

    use super::save_sub_category;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_save_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = 1;
        let base_category = BaseCategory::new(
            1,
            book_id,
            true,
            true,
            "서브카테고리용".to_string(),
            "FF0012".to_string(),
        );
        let base_id = save_base_category(&pool, user_id, base_category)
            .await
            .unwrap();

        let sub_category = SubCategory::new(base_id, "테스트 서브 카테고리".to_string());

        // Act
        let result = save_sub_category(&pool, user_id, sub_category).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, SubCategory>("SELECT * FROM tb_sub_category WHERE id = $1")
            .bind(result)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result, row.get_id())
    }

    #[tokio::test]
    async fn check_base_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = -32;

        let sub_category = SubCategory::new(base_id, "없는 베이스".to_string());

        // Act
        let result = save_sub_category(&pool, user_id, sub_category).await;

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
        let base_id = 1;
        let sub_category = SubCategory::new(base_id, "권한없는 카테고리".to_string());

        // Act
        let result = save_sub_category(&pool, user_id, sub_category).await;

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

        // ref) init.sql
        let user_id = 1;
        let base_id = 11; // 1-10 (book = null)
        let duplicate_name = "중복체크 서브 카테고리";
        let sub_category = SubCategory::new(base_id, duplicate_name.to_string());
        let _ = save_sub_category(&pool, user_id, sub_category).await;
        let sub_category = SubCategory::new(base_id, duplicate_name.to_string());

        // Act
        let result = save_sub_category(&pool, user_id, sub_category).await;

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

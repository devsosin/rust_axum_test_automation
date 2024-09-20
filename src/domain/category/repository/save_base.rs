use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::category::entity::BaseCategory, global::errors::CustomError};

pub struct SaveCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveCategoryRepo: Send + Sync {
    async fn save_base_category(
        &self,
        user_id: i32,
        base_category: BaseCategory,
    ) -> Result<i16, Box<CustomError>>;
}

impl SaveCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveCategoryRepo for SaveCategoryRepoImpl {
    async fn save_base_category(
        &self,
        user_id: i32,
        base_category: BaseCategory,
    ) -> Result<i16, Box<CustomError>> {
        save_base_category(&self.pool, user_id, base_category).await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct InsertResult {
    id: Option<i16>,
    is_exist: bool,
    is_authorized: bool,
    is_duplicated: bool,
}

pub async fn save_base_category(
    pool: &PgPool,
    user_id: i32,
    base_category: BaseCategory,
) -> Result<i16, Box<CustomError>> {
    let result = sqlx::query_as::<_, InsertResult>(
        r"
        WITH BookExists AS (
            SELECT id FROM tb_book
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT br.book_id
            FROM BookExists AS b
            JOIN tb_user_book_role AS br ON b.id = br.book_id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ),
        DuplicateCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM tb_base_category AS c
                WHERE c.name = $6 
                    AND (c.book_id = $2 OR c.book_id IS NULL)
            ) AS is_duplicate
        ),
        InsertBaseCategory AS (
            INSERT INTO tb_base_category (type_id, book_id, is_record, is_income, name, color)
            SELECT $3, book_id, $4, $5, $6, $7
            FROM AuthorityCheck
            WHERE (SELECT is_duplicate FROM DuplicateCheck) = false
            RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM BookExists) AS is_exist,
            EXISTS (SELECT 1 FROM AuthorityCheck) AS is_authorized,
            (SELECT is_duplicate FROM DuplicateCheck) AS is_duplicated,
            (SELECT id FROM InsertBaseCategory) AS id;
        ",
    )
    .bind(user_id)
    .bind(base_category.get_book_id())
    .bind(base_category.get_type_id())
    .bind(base_category.get_is_record())
    .bind(base_category.get_is_income())
    .bind(base_category.get_name())
    .bind(base_category.get_color())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveBaseCategory): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if !result.is_exist {
        return Err(Box::new(CustomError::NotFound("Book".to_string())));
    } else if !result.is_authorized {
        return Err(Box::new(CustomError::Unauthorized("BookRole".to_string())));
    } else if result.is_duplicated {
        return Err(Box::new(CustomError::Duplicated(
            "BaseCategory".to_string(),
        )));
    }

    Ok(result.id.unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool, domain::category::entity::BaseCategory,
        global::errors::CustomError,
    };

    use super::save_base_category;

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
            "테스트 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );

        // Act
        let result = save_base_category(&pool, user_id, base_category).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, BaseCategory>("SELECT * FROM tb_base_category WHERE id = $1")
            .bind(result)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result, row.get_id())
    }

    #[tokio::test]
    async fn check_book_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = -32;
        let base_category = BaseCategory::new(
            1,
            book_id,
            true,
            true,
            "테스트 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );
        // Act
        let result = save_base_category(&pool, user_id, base_category).await;

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
        let book_id = 1;
        let base_category = BaseCategory::new(
            1,
            book_id,
            true,
            true,
            "권한 없는 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );
        // Act
        let result = save_base_category(&pool, user_id, base_category).await;

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
    async fn check_type_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        // ref) init.sql
        let user_id = 1;
        let book_id = 1;
        let base_category = BaseCategory::new(
            -3,
            book_id,
            true,
            true,
            "잘못된 typeid 카테고리".to_string(),
            "FF0012".to_string(),
        );
        // Act
        let result = save_base_category(&pool, user_id, base_category).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::DatabaseError(_) => true,
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
        let book_id = 1;
        let duplicate_name = "중복 베이스 이름";
        let base_category = BaseCategory::new(
            1,
            book_id,
            true,
            true,
            duplicate_name.to_string(),
            "FF0012".to_string(),
        );
        let _ = save_base_category(&pool, user_id, base_category.clone()).await;

        // Act
        let result = save_base_category(&pool, user_id, base_category).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_duplicated_base() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = 1;
        let duplicate_name = "수입";
        let base_category = BaseCategory::new(
            1,
            book_id,
            true,
            true,
            duplicate_name.to_string(),
            "FF0012".to_string(),
        );

        // Act
        let result = save_base_category(&pool, user_id, base_category).await;

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

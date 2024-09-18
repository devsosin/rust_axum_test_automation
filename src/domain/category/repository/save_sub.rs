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

pub async fn save_sub_category(
    pool: &PgPool,
    user_id: i32,
    sub_category: SubCategory,
) -> Result<i32, Box<CustomError>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::category::{
            entity::{BaseCategory, SubCategory},
            repository::save_base::save_base_category,
        },
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

        // Act

        // Assert
        assert!(false)
    }

    #[tokio::test]
    async fn check_base_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;

        // Act

        // Assert
        assert!(false)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 2;
        // base_id가 특정 유저가 생성한 가계부이고 권한이 없을 경우 에러
        let base_id = 32;
        let sub_category = SubCategory::new(base_id, "테스트 서브 카테고리".to_string());

        // Act
        // Assert
        assert!(false)
    }
}

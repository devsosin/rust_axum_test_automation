use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::category::entity::SubCategory, global::errors::CustomError};

pub struct GetCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetCategoryRepo {
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
    todo!()
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn func_name() {
        // Arrange
        todo!()

        // Act

        // Assert
    }
}

use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::image::entity::Image, global::errors::CustomError};

pub struct GetImageByIdRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetImageByIdRepo: Send + Sync {
    async fn get_image_by_id(&self, id: i32) -> Result<Image, Box<CustomError>>;
}

impl GetImageByIdRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl GetImageByIdRepo for GetImageByIdRepoImpl {
    async fn get_image_by_id(&self, id: i32) -> Result<Image, Box<CustomError>> {
        get_image_by_id(&self.pool, id).await
    }
}

pub async fn get_image_by_id(pool: &PgPool, id: i32) -> Result<Image, Box<CustomError>> {
    sqlx::query_as::<_, Image>("SELECT * FROM tb_image WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("(GetImage {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("Image".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };

            Box::new(err)
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::image::{entity::Image, repository::save::save_images},
        global::utils::get_uuid,
    };

    use super::get_image_by_id;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_image_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let content_types = vec!["jpg"].repeat(1);
        let images: Vec<Image> = content_types
            .into_iter()
            .map(|content_type| {
                Image::new(
                    "test_file_name.jpg".to_string(),
                    format!("{:?}.{}", get_uuid(), content_type),
                )
            })
            .collect();

        let id = save_images(&pool, images)
            .await
            .unwrap()
            .first()
            .unwrap()
            .to_owned();

        // Act
        let result = get_image_by_id(&pool, id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), id)
    }

    #[tokio::test]
    async fn check_image_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let id = -32;

        // Act
        let result = get_image_by_id(&pool, id).await;

        // Assert
        assert!(result.is_err())
    }
}

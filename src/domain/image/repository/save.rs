use std::sync::Arc;

use axum::async_trait;
use sqlx::{PgPool, Row};

use crate::{domain::image::entity::Image, global::errors::CustomError};

pub struct SaveImageRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveImageRepo: Send + Sync {
    async fn save_images(&self, images: Vec<Image>) -> Result<Vec<i32>, Box<CustomError>>;
}

impl SaveImageRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveImageRepo for SaveImageRepoImpl {
    async fn save_images(&self, images: Vec<Image>) -> Result<Vec<i32>, Box<CustomError>> {
        save_images(&self.pool, images).await
    }
}

pub async fn save_images(pool: &PgPool, images: Vec<Image>) -> Result<Vec<i32>, Box<CustomError>> {
    let mut query = "INSERT INTO tb_image(original_name, image_key) VALUES ".to_string();
    let mut values = Vec::with_capacity(images.len() * 2);

    for (i, _) in images.iter().enumerate() {
        values.push(format!("(${}, ${})", i * 2 + 1, i * 2 + 2));
    }
    query.push_str(&values.join(", "));
    query.push_str(" RETURNING id");

    let mut query_builder = sqlx::query(&query);
    // let mut query_args = Vec::with_capacity(images.len() * 2);
    for image in &images {
        query_builder = query_builder
            .bind(image.get_original_name())
            .bind(image.get_image_key());
    }

    let rows = query_builder.fetch_all(pool).await.map_err(|e| {
        let err_msg = format!("Error(SaveImage): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Box::new(err)
    })?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool, domain::image::entity::Image,
        global::utils::get_uuid,
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    use super::save_images;

    #[tokio::test]
    async fn check_save_images_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let content_types = vec!["jpg"].repeat(5);
        let images: Vec<Image> = content_types
            .into_iter()
            .map(|content_type| {
                Image::new(
                    "test_file_name.jpg".to_string(),
                    format!("{:?}.{}", get_uuid(), content_type),
                )
            })
            .collect();

        // Act
        let result = save_images(&pool, images.clone()).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        println!("{:?}", result.clone());
        assert_eq!(result.len(), images.len())
    }
}

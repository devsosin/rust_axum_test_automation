use std::sync::Arc;

use axum::async_trait;
use s3::Bucket;

use crate::{
    domain::image::{
        dto::{request::NewImages, response::PresignedUrl},
        entity::Image,
        repository::save::SaveImageRepo,
    },
    global::{errors::CustomError, utils::get_uuid},
};

pub struct CreateImageUsecaseImpl<T>
where
    T: SaveImageRepo,
{
    repository: T,
    bucket: Arc<Bucket>,
}

#[async_trait]
pub trait CreateImageUsecase: Send + Sync {
    async fn create_images(&self, images: NewImages)
        -> Result<Vec<PresignedUrl>, Box<CustomError>>;
}

impl<T> CreateImageUsecaseImpl<T>
where
    T: SaveImageRepo,
{
    pub fn new(repository: T, bucket: &Arc<Bucket>) -> Self {
        Self {
            repository,
            bucket: bucket.clone(),
        }
    }
}

#[async_trait]
impl<T> CreateImageUsecase for CreateImageUsecaseImpl<T>
where
    T: SaveImageRepo,
{
    async fn create_images(
        &self,
        images: NewImages,
    ) -> Result<Vec<PresignedUrl>, Box<CustomError>> {
        create_images(&self.repository, &self.bucket, images).await
    }
}

pub async fn create_images<T>(
    repository: &T,
    bucket: &Bucket,
    new_images: NewImages,
) -> Result<Vec<PresignedUrl>, Box<CustomError>>
where
    T: SaveImageRepo,
{
    let mut urls = Vec::with_capacity(new_images.len());
    let mut images = Vec::with_capacity(new_images.len());

    for image in new_images.get_file_names() {
        let ext = image.split(".").last().unwrap();
        let image_key = format!("{:?}.{}", get_uuid(), ext);

        let url = bucket
            .presign_put(&format!("raw/{}", image_key), 600, None, None)
            .await
            .map_err(|e| {
                let err_msg = format!("{:?}", &e);
                tracing::error!("{}", err_msg);

                Box::new(CustomError::Unexpected(e.into()))
            })?;
        urls.push(url);
        images.push(Image::new(image.to_string(), image_key));
    }

    match repository.save_images(images).await {
        Ok(ids) => {
            let datas = ids
                .into_iter()
                .enumerate()
                .map(|(i, id)| PresignedUrl::new(id, urls[i].to_string()))
                .collect();
            Ok(datas)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};
    use s3::{creds::Credentials, Bucket};

    use crate::{
        config::aws::get_bucket,
        domain::image::{dto::request::NewImages, entity::Image, repository::save::SaveImageRepo},
        global::{errors::CustomError, utils::get_uuid},
    };

    use super::create_images;

    mock! {
        SaveImageRepoImpl {}

        #[async_trait]
        impl SaveImageRepo for SaveImageRepoImpl {
            async fn save_images(&self, images: Vec<Image>) -> Result<Vec<i32>, Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_create_image_success() {
        // Arrange
        let new_images = NewImages::new(vec![
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
        ]);

        let content_types = vec!["jpg"].repeat(new_images.len());
        let images: Vec<Image> = content_types
            .into_iter()
            .map(|content_type| {
                Image::new(
                    "test_images.jpg".to_string(),
                    format!("{:?}.{}", get_uuid(), content_type),
                )
            })
            .collect();

        let mut mock_repo = MockSaveImageRepoImpl::new();
        mock_repo
            .expect_save_images()
            .with(predicate::eq(images.clone()))
            .returning(|imgs| Ok((1..=(imgs.len() as i32)).collect()));

        let bucket = get_bucket();

        // Act
        let result = create_images(&mock_repo, &bucket, new_images.clone()).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), new_images.len())
    }

    // cannot fail
    #[tokio::test]
    async fn check_s3_credential_fail() {
        // Arrange
        let new_images = NewImages::new(vec!["test_image.jpg".to_string()]);

        let content_types = vec!["jpg"].repeat(new_images.len());
        let images: Vec<Image> = content_types
            .into_iter()
            .map(|content_type| {
                Image::new(
                    "test_image.jpg".to_string(),
                    format!("{:?}.{}", get_uuid(), content_type),
                )
            })
            .collect();

        let bucket_name = "rust-s3-test";
        let region = "us-east-1".parse().unwrap();
        let credentials =
            Credentials::new(Some("for test key"), Some("for fail key"), None, None, None).unwrap();

        let bucket = Bucket::new(bucket_name, region, credentials).unwrap();

        let mut mock_repo = MockSaveImageRepoImpl::new();
        mock_repo
            .expect_save_images()
            .with(predicate::eq(images.clone()))
            .returning(|imgs| Ok((1..=(imgs.len() as i32)).collect()));

        // Act
        let result = create_images(&mock_repo, &bucket, new_images.clone()).await;

        // Assert
        assert!(result.is_ok()); // infailable -> must be credential fail
    }
}

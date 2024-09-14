use axum::async_trait;

use crate::{
    domain::image::{entity::Image, repository::get_by_id::GetImageByIdRepo},
    global::errors::CustomError,
};

pub struct ReadImageUsecaseImpl<T>
where
    T: GetImageByIdRepo,
{
    repository: T,
}

#[async_trait]
pub trait ReadImageUsecase: Send + Sync {
    async fn read_image(&self, id: i32) -> Result<Image, Box<CustomError>>;
}

impl<T> ReadImageUsecaseImpl<T>
where
    T: GetImageByIdRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadImageUsecase for ReadImageUsecaseImpl<T>
where
    T: GetImageByIdRepo,
{
    async fn read_image(&self, id: i32) -> Result<Image, Box<CustomError>> {
        read_image(&self.repository, id).await
    }
}

pub async fn read_image<T>(repository: &T, id: i32) -> Result<Image, Box<CustomError>>
where
    T: GetImageByIdRepo,
{
    repository.get_image_by_id(id).await
}

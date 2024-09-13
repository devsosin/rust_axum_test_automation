use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::user::{dto::response::UserInfo, repository::get_by_id::GetUserByIdRepo},
    global::errors::CustomError,
};

pub struct RefreshTokenUsecaseImpl<T>
where
    T: GetUserByIdRepo,
{
    repository: T,
}

#[async_trait]
pub trait RefreshTokenUsecase: Send + Sync {
    async fn refresh(&self, id: i32) -> Result<UserInfo, Arc<CustomError>>;
}

impl<T> RefreshTokenUsecaseImpl<T>
where
    T: GetUserByIdRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> RefreshTokenUsecase for RefreshTokenUsecaseImpl<T>
where
    T: GetUserByIdRepo,
{
    async fn refresh(&self, id: i32) -> Result<UserInfo, Arc<CustomError>> {
        _refresh(&self.repository, id).await
    }
}
async fn _refresh<T>(repository: &T, id: i32) -> Result<UserInfo, Arc<CustomError>>
where
    T: GetUserByIdRepo,
{
    Ok(repository.get_by_id(id).await?.to_info())
}

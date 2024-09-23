use axum::async_trait;

use crate::{
    domain::connect::{entity::Connect, repository::get::GetConnectRepo},
    global::errors::CustomError,
};

pub struct ReadConnectUsecaseImpl<T>
where
    T: GetConnectRepo,
{
    repository: T,
}

#[async_trait]
pub trait ReadConnectUsecase: Send + Sync {
    async fn read_connect(&self, name: String) -> Result<Connect, Box<CustomError>>;
}

impl<T> ReadConnectUsecaseImpl<T>
where
    T: GetConnectRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadConnectUsecase for ReadConnectUsecaseImpl<T>
where
    T: GetConnectRepo,
{
    async fn read_connect(&self, name: String) -> Result<Connect, Box<CustomError>> {
        _read_connect(&self.repository, name).await
    }
}

async fn _read_connect<T>(repository: &T, name: String) -> Result<Connect, Box<CustomError>>
where
    T: GetConnectRepo,
{
    repository.get_connect_by_name(name).await
}

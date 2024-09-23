use axum::async_trait;

use crate::{
    domain::connect::{dto::request::NewConnect, repository::save::SaveConnectRepo},
    global::errors::CustomError,
};

pub struct CreateConnectUsecaseImpl<T>
where
    T: SaveConnectRepo,
{
    repository: T,
}

#[async_trait]
pub trait CreateConnectUsecase: Send + Sync {
    async fn create_connect(&self, new_connect: NewConnect) -> Result<i32, Box<CustomError>>;
}

impl<T> CreateConnectUsecaseImpl<T>
where
    T: SaveConnectRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateConnectUsecase for CreateConnectUsecaseImpl<T>
where
    T: SaveConnectRepo,
{
    async fn create_connect(&self, new_connect: NewConnect) -> Result<i32, Box<CustomError>> {
        _create_connect(&self.repository, new_connect).await
    }
}

async fn _create_connect<T>(
    repository: &T,
    new_connect: NewConnect,
) -> Result<i32, Box<CustomError>>
where
    T: SaveConnectRepo,
{
    repository
        .save_connect(new_connect.get_name().to_string())
        .await
}

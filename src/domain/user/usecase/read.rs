use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::user::{dto::response::UserInfo, repository::get_user::GetUserRepo},
    global::errors::CustomError,
};

pub(crate) struct ReadUserUsecaseImpl<T>
where
    T: GetUserRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub(crate) trait ReadUserUsecase: Send + Sync {
    async fn read_user(&self, id: i32) -> Result<UserInfo, Arc<CustomError>>;
}

impl<T> ReadUserUsecaseImpl<T>
where
    T: GetUserRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadUserUsecase for ReadUserUsecaseImpl<T>
where
    T: GetUserRepo,
{
    async fn read_user(&self, id: i32) -> Result<UserInfo, Arc<CustomError>> {
        read_user(&*self.repository, id).await
    }
}

pub(crate) async fn read_user<T>(repository: &T, id: i32) -> Result<UserInfo, Arc<CustomError>>
where
    T: GetUserRepo,
{
    let user = repository.get_by_id(id).await?;

    Ok(user.to_info())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::user::{entity::User, repository::get_user::GetUserRepo},
        global::errors::CustomError,
    };

    use super::read_user;

    mock! {
        GetUserRepoImpl {}

        #[async_trait]
        impl GetUserRepo for GetUserRepoImpl {
            async fn get_by_id(&self, id: i32) -> Result<User, Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_get_user_success() {
        // Arrange
        let id = 1;

        let mut mock_repo = MockGetUserRepoImpl::new();
        mock_repo
            .expect_get_by_id()
            .with(predicate::eq(id))
            .returning(|i| {
                Ok(User::new(
                    "test1234@test.test".to_string(),
                    "test_password".to_string(),
                    "nickname".to_string(),
                    "email".to_string(),
                )
                .id(i)
                .build())
            });

        // Act
        let result = read_user(&mock_repo, id).await;

        // Assert
        assert!(result.is_ok())
    }
}

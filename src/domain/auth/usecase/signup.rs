use axum::async_trait;
use std::sync::Arc;

use crate::{
    domain::user::{
        dto::request::NewUser,
        repository::save::SaveUserRepo,
        utils::password_hash::{hash_password, hash_password_fixed},
    },
    global::errors::CustomError,
};

pub struct SignupUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    repository: T,
}

#[async_trait]
pub trait SignupUserUsecase: Send + Sync {
    async fn signup_user(&self, new_user: NewUser) -> Result<i32, Arc<CustomError>>;
}

impl<T> SignupUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    pub(crate) fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> SignupUserUsecase for SignupUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    async fn signup_user(&self, new_user: NewUser) -> Result<i32, Arc<CustomError>> {
        _signup_user(&self.repository, new_user).await
    }
}

#[cfg(not(test))]
fn _hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    hash_password(password.as_bytes())
}

#[cfg(test)]
fn _hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    hash_password_fixed(password.as_bytes(), "fixedsaltfortest") // valid base64 string it's crazy
}

async fn _signup_user<T>(repository: &T, mut new_user: NewUser) -> Result<i32, Arc<CustomError>>
where
    T: SaveUserRepo,
{
    let hashed_password = _hash_password(new_user.password()).map_err(|e| {
        let err_msg = format!("Error(CreateUser-hashing): {:?}", &e);
        tracing::error!("{}", err_msg);

        Arc::new(CustomError::Unexpected(anyhow::Error::msg(
            "failed to hashing password",
        )))
    })?;

    new_user.set_password(hashed_password);

    repository.save_user(new_user.to_entity()).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::user::{
            dto::request::{LoginType, NewUser},
            entity::User,
            repository::save::SaveUserRepo,
        },
        global::errors::CustomError,
    };

    use super::{_hash_password, _signup_user};

    mock! {
        SaveUserRepoImpl {}

        #[async_trait]
        impl SaveUserRepo for SaveUserRepoImpl {
            async fn save_user(&self, user: User) -> Result<i32, Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_create_user_success() {
        // Arrange
        let new_user = NewUser::new(
            LoginType::Email,
            "test1234@test.test".to_string(),
            "test_password".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "test1234@test.test".to_string(),
            None,
            None,
        );
        let user = User::new(
            "test1234@test.test".to_string(),
            _hash_password("test_password").unwrap(),
            "nickname".to_string(),
            "test1234@test.test".to_string(),
            "email".to_string(),
        );

        let mut mock_repo = MockSaveUserRepoImpl::new();
        mock_repo
            .expect_save_user()
            .with(predicate::eq(user.clone()))
            .returning(|_| Ok(1));

        // Act
        let result = _signup_user(&mock_repo, new_user).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap(), 1);
    }
}

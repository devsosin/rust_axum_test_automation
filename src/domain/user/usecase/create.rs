use axum::async_trait;
use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};

use crate::{
    domain::user::{dto::request::NewUser, repository::save::SaveUserRepo, util::hash_password},
    global::errors::CustomError,
};

pub(crate) struct CreateUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub(crate) trait CreateUserUsecase: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<i32, Arc<CustomError>>;
}

impl<T> CreateUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateUserUsecase for CreateUserUsecaseImpl<T>
where
    T: SaveUserRepo,
{
    async fn create_user(&self, new_user: NewUser) -> Result<i32, Arc<CustomError>> {
        _create_user(&*self.repository, new_user).await
    }
}

fn _hash_password_fixed(password: &[u8]) -> Result<String, argon2::password_hash::Error> {
    // 고정된 솔트를 사용하여 일관된 해싱 결과를 얻기 위해
    let salt = SaltString::from_b64("fixedsaltfortest").unwrap(); // 고정된 22자 솔트
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
}

async fn _create_user<T>(repository: &T, mut new_user: NewUser) -> Result<i32, Arc<CustomError>>
where
    T: SaveUserRepo,
{
    let hashed_password = hash_password(new_user.password().as_bytes()).map_err(|e| {
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

    use super::{_create_user, _hash_password_fixed};

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
            "test1234@test.test".to_string(),
            "test_password".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );
        let user = User::new(
            "test1234@test.test".to_string(),
            _hash_password_fixed("test_password".as_bytes()).unwrap(),
            "nickname".to_string(),
            "email".to_string(),
        );

        let mut mock_repo = MockSaveUserRepoImpl::new();
        mock_repo
            .expect_save_user()
            .with(predicate::eq(user.clone()))
            .returning(|_| Ok(1));

        // Act
        let result = _create_user(&mock_repo, new_user).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap(), 1);
    }
}

use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::user::{
        dto::request::EditUser,
        repository::update::UpdateUserRepo,
        utils::password_hash::{hash_password, hash_password_fixed},
    },
    global::errors::CustomError,
};

pub(crate) struct UpdateUserUsecaseImpl<T>
where
    T: UpdateUserRepo,
{
    repository: T,
}

#[async_trait]
pub(crate) trait UpdateUserUsecase: Send + Sync {
    async fn update_user(&self, id: i32, edit_user: EditUser) -> Result<(), Arc<CustomError>>;
}

impl<T> UpdateUserUsecaseImpl<T>
where
    T: UpdateUserRepo,
{
    pub(crate) fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> UpdateUserUsecase for UpdateUserUsecaseImpl<T>
where
    T: UpdateUserRepo,
{
    async fn update_user(&self, id: i32, edit_user: EditUser) -> Result<(), Arc<CustomError>> {
        _update_user(&self.repository, id, edit_user).await
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

async fn _update_user<T>(
    repository: &T,
    id: i32,
    edit_user: EditUser,
) -> Result<(), Arc<CustomError>>
where
    T: UpdateUserRepo,
{
    if let Some(password) = edit_user.get_password() {
        let hashed_password = _hash_password(password.get_password()).map_err(|_| {
            let err = CustomError::Unexpected(anyhow::Error::msg("password hashing error"));
            Arc::new(err)
        })?;
        repository.verify_password(id, hashed_password).await?
    }

    repository.update_user(id, edit_user.to_update()).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::user::{
            dto::request::{EditPassword, EditUser},
            entity::UpdateUser,
            repository::update::UpdateUserRepo,
        },
        global::{constants::FieldUpdate, errors::CustomError},
    };

    use super::{_hash_password, _update_user};

    mock! {
        UpdateUserRepoImpl {}

        #[async_trait]
        impl UpdateUserRepo for UpdateUserRepoImpl {
            async fn update_user(&self, id: i32, edit_user: UpdateUser) -> Result<(), Arc<CustomError>>;
            async fn verify_password(&self, id: i32, password: String) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_update_user_success() {
        // Arrange
        let id = 1;

        let original_password = "original_password";
        let update_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set("new_password".to_string()),
            FieldUpdate::Set("010-2345-6789".to_string()),
            FieldUpdate::NoChange,
        );

        let mut mock_repo = MockUpdateUserRepoImpl::new();
        mock_repo
            .expect_update_user()
            .with(predicate::eq(id), predicate::eq(update_user.clone()))
            .returning(|_, _| Ok(()));
        mock_repo
            .expect_verify_password()
            .with(
                predicate::eq(id),
                predicate::eq(_hash_password(original_password).unwrap()),
            )
            .returning(|_, _| Ok(()));

        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                original_password.to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        // Act
        let result = _update_user(&mock_repo, id, edit_user).await;

        // Assert
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn check_password_not_equal() {
        // Arrange
        let id = 1;
        let incorrect_password = "incorrect_password";
        let update_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set("new_password".to_string()),
            FieldUpdate::Set("010-2345-6789".to_string()),
            FieldUpdate::NoChange,
        );

        let mut mock_repo = MockUpdateUserRepoImpl::new();
        mock_repo
            .expect_update_user()
            .with(predicate::eq(id), predicate::eq(update_user.clone()))
            .returning(|_, _| Ok(()));
        mock_repo
            .expect_verify_password()
            .with(
                predicate::eq(id),
                predicate::eq(_hash_password(incorrect_password).unwrap()),
            )
            .returning(|_, _| Err(Arc::new(CustomError::ValidationError("User".to_string()))));

        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                incorrect_password.to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        // Act
        let result = _update_user(&mock_repo, id, edit_user).await;

        // Assert
        assert!(result.is_err())
    }
}

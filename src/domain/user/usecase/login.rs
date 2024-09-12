use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::user::{
        dto::{
            request::{LoginInfo, LoginType},
            response::UserInfo,
        },
        entity::User,
        repository::{
            login::LoginUserRepo,
            save::{save_user, SaveUserRepo},
        },
        utils::password_hash::{hash_password, hash_password_fixed, verify_password},
    },
    global::errors::CustomError,
};

pub(crate) struct LoginUserUsecaseImpl<T, U>
where
    T: LoginUserRepo,
    U: SaveUserRepo,
{
    login_repo: T,
    save_repo: U,
}

#[async_trait]
pub(crate) trait LoginUserUsecase: Send + Sync {
    async fn login(&self, login_info: LoginInfo) -> Result<UserInfo, Arc<CustomError>>;
}

impl<T, U> LoginUserUsecaseImpl<T, U>
where
    T: LoginUserRepo,
    U: SaveUserRepo,
{
    pub(crate) fn new(login_repo: T, save_repo: U) -> Self {
        Self {
            login_repo,
            save_repo,
        }
    }
}

#[async_trait]
impl<T, U> LoginUserUsecase for LoginUserUsecaseImpl<T, U>
where
    T: LoginUserRepo,
    U: SaveUserRepo,
{
    async fn login(&self, login_info: LoginInfo) -> Result<UserInfo, Arc<CustomError>> {
        _login(&self.login_repo, &self.save_repo, login_info).await
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

async fn _login<T, U>(
    login_repo: &T,
    save_repo: &U,
    login_info: LoginInfo,
) -> Result<UserInfo, Arc<CustomError>>
where
    T: LoginUserRepo,
    U: SaveUserRepo,
{
    let result = login_repo.get_by_username(login_info.get_username()).await;
    let user;
    if let Err(e) = result {
        user = match login_info.get_login_type() {
            LoginType::Email => Err(e),
            _ => {
                let oauth_user = login_info.to_entity();
                let new_id = save_repo.save_user(oauth_user.clone()).await?;
                Ok(oauth_user.id(new_id).build())
            }
        }?;
    } else {
        user = result.unwrap();
    }

    // 비밀번호 체크
    if !verify_password(user.get_password(), login_info.get_password().as_bytes()).unwrap() {
        return Err(Arc::new(CustomError::ValidationError(
            "Password".to_string(),
        )));
    }

    Ok(user.to_info())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::user::{
            dto::request::{LoginInfo, LoginType},
            entity::User,
            repository::{login::LoginUserRepo, save::SaveUserRepo},
        },
        global::errors::CustomError,
    };

    use super::{_hash_password, _login};

    mock! {
        LoginUserRepoImpl {}

        #[async_trait]
        impl LoginUserRepo for LoginUserRepoImpl {
            async fn get_by_username(&self, username: &str) -> Result<User, Arc<CustomError>>;
        }
    }

    mock! {
        SaveUserRepoImpl {}

        #[async_trait]
        impl SaveUserRepo for SaveUserRepoImpl {
            async fn save_user(&self, user: User) -> Result<i32, Arc<CustomError>>;
        }
    }

    fn _get_save_repo(id: Option<i32>) -> MockSaveUserRepoImpl {
        let mut repo = MockSaveUserRepoImpl::new();

        if let Some(user_id) = id {
            repo.expect_save_user().returning(move |_| Ok(user_id));
        };

        repo
    }

    fn _get_login_repo(
        username: &str,
        pw: String,
        login_type: String,
        user_id: i32,
    ) -> MockLoginUserRepoImpl {
        let mut mock_login_repo = MockLoginUserRepoImpl::new();
        mock_login_repo
            .expect_get_by_username()
            .with(predicate::eq(username.to_string()))
            .returning(move |un| {
                Ok(User::new(
                    un.to_string(),
                    _hash_password(pw.as_str()).unwrap(),
                    "testnick".to_string(),
                    un.to_string(),
                    login_type.clone(),
                )
                .id(user_id)
                .build())
            });

        mock_login_repo
    }

    #[tokio::test]
    async fn check_login_success() {
        // Arrange
        let username = "login_usecase@test.test";
        let user_id = 1;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw".to_string(),
            "email".to_string(),
            user_id,
        );
        let mock_save_repo = _get_save_repo(Some(1));

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), user_id)
    }

    #[tokio::test]
    async fn check_naver_signup_login_success() {
        // Arrange
        let username = "navertest_unique_id";
        let user_id = 2;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw_naver".to_string(),
            LoginType::Naver,
            Some("naver_test_email@test.test".to_string()),
            Some("testnick".to_string()),
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_naver".to_string(),
            "naver".to_string(),
            user_id,
        );
        let mock_save_repo = _get_save_repo(Some(user_id));

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        // 생성 여부 체크
        assert_eq!(result.get_id(), user_id)
    }

    #[tokio::test]
    async fn check_google_signup_login_success() {
        // Arrange
        let username = "googletest_unique_id";
        let user_id = 3;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw_google".to_string(),
            LoginType::Google,
            Some("google_test_email@test.test".to_string()),
            Some("testnick".to_string()),
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_google".to_string(),
            "google".to_string(),
            user_id,
        );
        let mock_save_repo = _get_save_repo(Some(user_id));

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), user_id)
    }

    #[tokio::test]
    async fn check_kakao_signup_login_success() {
        // Arrange
        let username = "kakaotest_unique_id";
        let user_id = 4;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw_kakao".to_string(),
            LoginType::Kakao,
            Some("kakao_test_email@test.test".to_string()),
            Some("testnick".to_string()),
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_kakao".to_string(),
            "kakao".to_string(),
            user_id,
        );
        let mock_save_repo = _get_save_repo(Some(user_id));

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), user_id)
    }

    #[tokio::test]
    async fn check_meta_signup_login_success() {
        // Arrange
        let username = "metatest_unique_id";
        let user_id = 5;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw_meta".to_string(),
            LoginType::Meta,
            Some("meta_test_email@test.test".to_string()),
            Some("testnick".to_string()),
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_meta".to_string(),
            "meta".to_string(),
            user_id,
        );
        let mock_save_repo = _get_save_repo(Some(user_id));

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), user_id)
    }

    #[tokio::test]
    async fn check_username_not_found() {
        // Arrange
        let username = "not_found_email@test.test";
        let login_info = LoginInfo::new(
            username.to_string(),
            "invalid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mut mock_repo = MockLoginUserRepoImpl::new();
        mock_repo
            .expect_get_by_username()
            .with(predicate::eq(username))
            .returning(|_| Err(Arc::new(CustomError::NotFound("User".to_string()))));

        let mock_save_repo = _get_save_repo(None);

        // Act
        let result = _login(&mock_repo, &mock_save_repo, login_info).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_pw_incorrect() {
        // Arrange
        let username = "pw_incorrect@test.test";
        let login_info = LoginInfo::new(
            username.to_string(),
            "invalid_pw_email".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_email".to_string(),
            "email".to_string(),
            394,
        );
        let mock_save_repo = _get_save_repo(None);

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_pw_incorrect_oauth() {
        // Arrange
        let username = "pw_incorrect_oauth@test.test";
        let login_info = LoginInfo::new(
            username.to_string(),
            "invalid_pw_meta".to_string(),
            LoginType::Naver,
            None,
            None,
            None,
        );

        let mock_login_repo = _get_login_repo(
            username,
            "valid_pw_naver".to_string(),
            "naver".to_string(),
            3766,
        );
        let mock_save_repo = _get_save_repo(None);

        // Act
        let result = _login(&mock_login_repo, &mock_save_repo, login_info).await;

        // Assert
        assert!(result.is_err())
    }
}

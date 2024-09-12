use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{
    domain::user::entity::{UpdateUser, User},
    global::{constants::FieldUpdate, errors::CustomError},
};

pub(crate) struct UpdateUserRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait UpdateUserRepo: Send + Sync {
    async fn update_user(&self, id: i32, edit_user: UpdateUser) -> Result<(), Arc<CustomError>>;
    async fn verify_password(&self, id: i32, password: String) -> Result<(), Arc<CustomError>>;
}

impl UpdateUserRepoImpl {
    pub(crate) fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl UpdateUserRepo for UpdateUserRepoImpl {
    async fn update_user(&self, id: i32, edit_user: UpdateUser) -> Result<(), Arc<CustomError>> {
        _update_user(&self.pool, id, edit_user).await
    }
    async fn verify_password(&self, id: i32, password: String) -> Result<(), Arc<CustomError>> {
        _verify_password(&self.pool, id, password).await
    }
}

async fn _verify_password(
    pool: &PgPool,
    id: i32,
    password: String,
) -> Result<(), Arc<CustomError>> {
    let row = sqlx::query_as::<_, User>("SELECT * FROM tb_user WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(UpdateUser-Verify {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("User".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    if row.get_password() != password {
        return Err(Arc::new(CustomError::ValidationError("User".to_string())));
    }

    Ok(())
}

async fn _update_user(
    pool: &PgPool,
    id: i32,
    edit_user: UpdateUser,
) -> Result<(), Arc<CustomError>> {
    let mut query: String = "UPDATE tb_user SET ".to_string();
    let mut index = 0;

    match edit_user.get_profile_id() {
        FieldUpdate::NoChange => {}
        _ => {
            index += 1;
            query.push_str(format!("profile_id = ${}, ", index).as_str());
        }
    }
    match edit_user.get_password() {
        FieldUpdate::Set(_) => {
            index += 1;
            query.push_str(format!("password = ${}, ", index).as_str());
        }
        _ => {}
    }
    match edit_user.get_nickname() {
        FieldUpdate::Set(_) => {
            index += 1;
            query.push_str(format!("nickname = ${}, ", index).as_str());
        }
        _ => {}
    }
    match edit_user.get_phone() {
        FieldUpdate::Set(_) => {
            index += 1;
            query.push_str(format!("phone = ${}, ", index).as_str());
        }
        _ => {}
    }

    if index == 0 {
        return Err(Arc::new(CustomError::NoFieldUpdate("User".to_string())));
    }

    query.push_str("updated_at = NOW()");
    query.push_str(format!(" WHERE id = ${}", index + 1).as_str());

    let mut query_builder = sqlx::query(&query);

    match edit_user.get_profile_id() {
        FieldUpdate::Set(profile_id) => {
            query_builder = query_builder.bind(profile_id);
        }
        FieldUpdate::SetNone => {
            query_builder = query_builder.bind(&None::<i32>);
        }
        _ => {}
    }
    match edit_user.get_password() {
        FieldUpdate::Set(password) => {
            query_builder = query_builder.bind(password);
        }
        _ => {}
    }
    match edit_user.get_nickname() {
        FieldUpdate::Set(nickname) => {
            query_builder = query_builder.bind(nickname);
        }
        _ => {}
    }
    match edit_user.get_phone() {
        FieldUpdate::Set(phone) => {
            query_builder = query_builder.bind(phone);
        }
        _ => {}
    }

    let result = query_builder.bind(id).execute(pool).await.map_err(|e| {
        let err_msg = format!("Error(UpdateUser {}): {:?}", id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Arc::new(err)
    })?;

    if result.rows_affected() == 0 {
        return Err(Arc::new(CustomError::NotFound("User".to_string())));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        config::database::create_connection_pool,
        domain::user::{
            entity::{UpdateUser, User},
            repository::{get_user::get_by_id, save::save_user},
        },
        global::constants::FieldUpdate,
    };

    use super::{_update_user, _verify_password};

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_update_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "updatetest1@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "updatetest1@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();
        let edit_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set("new_password".to_string()),
            FieldUpdate::Set("010-2345-6789".to_string()),
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_user(&pool, new_id, edit_user).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let updated_user = get_by_id(&pool, new_id).await.unwrap();

        assert_eq!(updated_user.get_password(), "new_password");
        assert_eq!(
            updated_user
                .get_phone()
                .as_ref()
                .expect("phone must be set"),
            "010-2345-6789"
        );
    }

    #[tokio::test]
    async fn check_no_field_to_update() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "updatetest2@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "updatetest2@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        let edit_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_user(&pool, new_id, edit_user).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_user_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let no_id = -32;

        let edit_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::Set("newname".to_string()),
        );

        // Act
        let result = _update_user(&pool, no_id, edit_user).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_profile_id_valid() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "updatetest3@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "updatetest3@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        let edit_user = UpdateUser::new(
            FieldUpdate::Set(-32),
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_user(&pool, new_id, edit_user).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_password_valid() {
        // Arrange
        let pool = create_connection_pool().await;

        let hashed_password = "test_password".to_string();

        let user = User::new(
            "test1234pw@test.test".to_string(),
            hashed_password.clone(),
            "nickname".to_string(),
            "test1234pw@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        // Act
        let result = _verify_password(&pool, new_id, hashed_password).await;

        // Assert
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn check_updated_at_changed() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "test1234up@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "test1234up@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        let edit_user = UpdateUser::new(
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::Set("010-1234-5678".to_string()),
            FieldUpdate::NoChange,
        );

        let last_time = Utc::now().naive_utc();

        // Act
        let result = _update_user(&pool, new_id, edit_user).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        let edited_user = get_by_id(&pool, new_id).await.unwrap();

        // Assert
        assert!(last_time < edited_user.get_updated_at().unwrap())
    }
}

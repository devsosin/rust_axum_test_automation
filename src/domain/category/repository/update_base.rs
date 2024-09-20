use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{
    domain::category::entity::UpdateBaseCategory,
    global::{
        constants::{FieldUpdate, UpdateResult},
        errors::CustomError,
    },
};

pub struct UpdateCategoryRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait UpdateCategoryRepo: Send + Sync {
    async fn update_base_category(
        &self,
        user_id: i32,
        base_id: i16,
        update_base: UpdateBaseCategory,
    ) -> Result<(), Box<CustomError>>;
}

impl UpdateCategoryRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl UpdateCategoryRepo for UpdateCategoryRepoImpl {
    async fn update_base_category(
        &self,
        user_id: i32,
        base_id: i16,
        update_base: UpdateBaseCategory,
    ) -> Result<(), Box<CustomError>> {
        _update_base_category(&self.pool, user_id, base_id, update_base).await
    }
}

fn make_query(index: &mut i32, field_name: &str) -> String {
    *index += 1;
    format!("{} = ${}, ", field_name, index)
}

async fn _update_base_category(
    pool: &PgPool,
    user_id: i32,
    base_id: i16,
    update_base: UpdateBaseCategory,
) -> Result<(), Box<CustomError>> {
    let mut query = "
    WITH BaseCategoryExists AS (
        SELECT id, book_id
        FROM tb_base_category
        WHERE id = $2
    ),
    AuthorityCheck AS (
        SELECT EXISTS (
            SELECT 1
            FROM BaseCategoryExists AS c
            JOIN tb_book AS b ON b.id = c.book_id
            JOIN tb_user_book_role AS br ON b.id = br.book_id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ) AS is_authorized
    ),
    "
    .to_string();

    let mut index = 2;
    let mut name_check = false;

    let mut update_query = r"
        UpdateBaseCategory AS (
            UPDATE tb_base_category SET "
        .to_string();

    if let FieldUpdate::Set(_) = update_base.get_name() {
        update_query.push_str(&make_query(&mut index, "name"));
        query.push_str(&format!(
            r"
            DuplicateCheck AS (
                SELECT EXISTS (
                    SELECT 1
                    FROM tb_base_category AS c
                    LEFT JOIN tb_book AS b ON b.id = c.book_id
                    WHERE c.id != $2 AND c.name = ${}
                        AND (b.id = $2 OR b.id IS NULL)
                ) AS is_duplicated
            ),",
            index
        ));
        name_check = true;
    }

    if let FieldUpdate::Set(_) = update_base.get_color() {
        update_query.push_str(&make_query(&mut index, "color"));
    }

    if index == 2 {
        return Err(Box::new(CustomError::NoFieldUpdate(
            "BaseCategory".to_string(),
        )));
    }

    update_query.pop();
    update_query.pop();

    update_query.push_str(
        &("
        WHERE id = $2
            AND (SELECT is_authorized FROM AuthorityCheck) = true
        "
        .to_string()
            + if name_check {
                "AND (SELECT is_duplicated FROM DuplicateCheck) = false\n"
            } else {
                ""
            }),
    );

    query.push_str(
        &(update_query
            + "RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM BaseCategoryExists) AS is_exist,
            (SELECT COUNT(*) FROM UpdateBaseCategory) AS update_count,
            (SELECT is_authorized FROM AuthorityCheck) AS is_authorized,
    " + if name_check {
            "(SELECT is_duplicated FROM DuplicateCheck) AS is_duplicated"
        } else {
            "false AS is_duplicated"
        } + ";"),
    );

    let mut query_builder = sqlx::query_as::<_, UpdateResult>(&query)
        .bind(user_id)
        .bind(base_id);

    if let FieldUpdate::Set(v) = update_base.get_name() {
        query_builder = query_builder.bind(v);
    }
    if let FieldUpdate::Set(v) = update_base.get_color() {
        query_builder = query_builder.bind(v);
    }

    println!("{}", query);

    let result = query_builder.fetch_one(pool).await.map_err(|e| {
        let err_msg = format!("Error(UpdateBaseCategory {}): {:?}", base_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };

        Box::new(err)
    })?;

    if !result.get_exist() {
        return Err(Box::new(CustomError::NotFound("Record".to_string())));
    } else if !result.get_authorized() {
        return Err(Box::new(CustomError::Unauthorized(
            "BaseCategoryRole".to_string(),
        )));
    } else if result.get_duplicated() {
        return Err(Box::new(CustomError::Duplicated(
            "BaseCategory".to_string(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::category::{
            entity::{BaseCategory, UpdateBaseCategory},
            repository::save_base::save_base_category,
        },
        global::{constants::FieldUpdate, errors::CustomError},
    };

    use super::_update_base_category;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_base_category_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_category = BaseCategory::new(
            1,
            1,
            true,
            true,
            "수정용 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );

        let base_id = save_base_category(&pool, user_id, base_category)
            .await
            .unwrap();

        let new_name = "수정한 카테고리명";

        let update_base = UpdateBaseCategory::new(
            FieldUpdate::Set(new_name.to_string()),
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_base_category(&pool, user_id, base_id, update_base).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = sqlx::query_as::<_, BaseCategory>("SELECT * FROM tb_base_category WHERE id = $1")
            .bind(base_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.get_name(), new_name);
    }

    #[tokio::test]
    async fn check_no_update_field() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_category = BaseCategory::new(
            1,
            1,
            true,
            true,
            "필드 없데이트 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );

        let base_id = save_base_category(&pool, user_id, base_category)
            .await
            .unwrap();

        let update_base = UpdateBaseCategory::new(FieldUpdate::NoChange, FieldUpdate::NoChange);

        // Act
        let result = _update_base_category(&pool, user_id, base_id, update_base).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NoFieldUpdate(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_base_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let base_id = -32;

        let new_name = "수정한 카테고리명";

        let update_base = UpdateBaseCategory::new(
            FieldUpdate::Set(new_name.to_string()),
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_base_category(&pool, user_id, base_id, update_base).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let pool = create_connection_pool().await;

        // ref) init.sql
        let user_id = 2;
        let base_id = 11; // 공통, 다른 사용자 모두 확인

        let new_color = "123456";

        let update_base = UpdateBaseCategory::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(new_color.to_string()),
        );

        // Act
        let result = _update_base_category(&pool, user_id, base_id, update_base).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // 공통 카테고리 이름과도 겹치는지 확인
        // Arrange
        let pool = create_connection_pool().await;

        // ref) init.sql
        let user_id = 1;
        let base_category = BaseCategory::new(
            1,
            1,
            true,
            true,
            "중복체크용 베이스 카테고리".to_string(),
            "FF0012".to_string(),
        );

        let base_id = save_base_category(&pool, user_id, base_category)
            .await
            .unwrap();

        let new_name = "수입"; // 공통 카테고리 이름

        let update_base = UpdateBaseCategory::new(
            FieldUpdate::Set(new_name.to_string()),
            FieldUpdate::NoChange,
        );

        // Act
        let result = _update_base_category(&pool, user_id, base_id, update_base).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}

use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{
    domain::record::entity::UpdateRecord,
    global::{constants::FieldUpdate, errors::CustomError},
};

pub struct UpdateRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait UpdateRecordRepo: Send + Sync {
    async fn update_record(
        &self,
        user_id: i32,
        record_id: i64,
        edit_record: UpdateRecord,
    ) -> Result<(), Arc<CustomError>>;
}

impl UpdateRecordRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl UpdateRecordRepo for UpdateRecordRepoImpl {
    async fn update_record(
        &self,
        user_id: i32,
        record_id: i64,
        edit_record: UpdateRecord,
    ) -> Result<(), Arc<CustomError>> {
        update_record(&self.pool, user_id, record_id, edit_record).await
    }
}

fn make_query(index: &mut i32, field_name: &str) -> String {
    *index += 1;
    format!("{} = ${}, ", field_name, index)
}

#[derive(Debug, sqlx::FromRow)]
struct UpdateRecordResult {
    is_exist: bool,
    is_authorized: bool,
    update_count: i64,
    is_asset_exist: bool,
    is_category_exist: bool,
}

impl UpdateRecordResult {
    fn get_exist(&self) -> bool {
        self.is_exist
    }
    fn get_authorized(&self) -> bool {
        self.is_authorized
    }
    fn get_asset_exist(&self) -> bool {
        self.is_asset_exist
    }
    fn get_category_exist(&self) -> bool {
        self.is_category_exist
    }
}

async fn update_record(
    pool: &PgPool,
    user_id: i32,
    record_id: i64,
    edit_record: UpdateRecord,
) -> Result<(), Arc<CustomError>> {
    let mut query = r"
        WITH RecordExists AS (
            SELECT book_id
            FROM tb_record
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT r.book_id
            FROM RecordExists AS r
            JOIN tb_book AS b ON b.id = r.book_id
            JOIN tb_user_book_role AS br ON b.id = br.book_id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ),"
    .to_string();

    let mut index = 2;
    let mut ctg_check = false;
    let mut asset_check = false;

    let mut update_query = r"
        UpdateRecord AS (
            UPDATE tb_record SET "
        .to_string();

    if let FieldUpdate::Set(_) = edit_record.get_sub_category_id() {
        update_query.push_str(&make_query(&mut index, "sub_category_id"));
        query.push_str(&format!(
            r"
            CategoryCheck AS (
                SELECT EXISTS (
                    SELECT 1
                    FROM tb_sub_category AS sc
                    JOIN tb_base_category AS bc ON bc.id = sc.base_id
                    LEFT JOIN RecordExists AS r ON bc.book_id = r.book_id
                    WHERE sc.id = ${}
                        AND (r.book_id IS NOT NULL OR bc.book_id IS NULL)
                ) AS is_category_exist
            ),",
            index
        ));
        ctg_check = true;
    };
    if let FieldUpdate::Set(_) = edit_record.get_amount() {
        update_query.push_str(&make_query(&mut index, "amount"));
    };
    if let FieldUpdate::Set(_) = edit_record.get_memo() {
        update_query.push_str(&make_query(&mut index, "memo"));
    };
    if let FieldUpdate::Set(_) = edit_record.get_target_dt() {
        update_query.push_str(&make_query(&mut index, "target_dt"));
    };
    match edit_record.get_asset_id() {
        FieldUpdate::Set(_) => {
            update_query.push_str(&make_query(&mut index, "asset_id"));
            query.push_str(&format!(
                r"
                AssetCheck AS (
                    SELECT EXISTS (
                        SELECT 1
                        FROM tb_asset AS a
                        JOIN RecordExists AS r ON r.book_id = a.book_id
                        WHERE a.id = ${}
                        ) AS is_asset_exist
                        ),",
                index
            ));
            asset_check = true;
        }
        FieldUpdate::SetNone => {
            update_query.push_str(&make_query(&mut index, "asset_id"));
        }
        _ => {}
    }

    if index == 2 {
        return Err(Arc::new(CustomError::NoFieldUpdate("Record".to_string())));
    }

    update_query.push_str(
        &("
            updated_at = NOW() 
            WHERE id = $2
                AND EXISTS (SELECT book_id FROM AuthorityCheck) = true
            "
        .to_string()
            + if ctg_check {
                "AND (SELECT is_category_exist FROM CategoryCheck) = true\n"
            } else {
                ""
            }
            + if asset_check {
                "AND (SELECT is_asset_exist FROM AssetCheck) = true\n"
            } else {
                ""
            }),
    );

    query.push_str(
        &(update_query
            + "RETURNING id
        )" + "
            SELECT
                EXISTS (SELECT 1 FROM RecordExists) AS is_exist,
                EXISTS (SELECT 1 FROM AuthorityCheck) AS is_authorized,
                (SELECT COUNT(*) FROM UpdateRecord) AS update_count\n"
            + if ctg_check {
                ",(SELECT is_category_exist FROM CategoryCheck) AS is_category_exist\n"
            } else {
                ",true AS is_category_exist"
            }
            + if asset_check {
                ",(SELECT is_asset_exist FROM AssetCheck) AS is_asset_exist\n"
            } else {
                ",true AS is_asset_exist"
            }
            + ";"),
    );

    println!("{}", query);

    let mut query_builder = sqlx::query_as::<_, UpdateRecordResult>(&query)
        .bind(user_id)
        .bind(record_id);

    if let FieldUpdate::Set(v) = edit_record.get_sub_category_id() {
        query_builder = query_builder.bind(v);
    }
    if let FieldUpdate::Set(v) = edit_record.get_amount() {
        query_builder = query_builder.bind(v);
    }
    if let FieldUpdate::Set(v) = edit_record.get_memo() {
        query_builder = query_builder.bind(v);
    }
    if let FieldUpdate::Set(v) = edit_record.get_target_dt() {
        query_builder = query_builder.bind(v);
    }
    match edit_record.get_asset_id() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        FieldUpdate::SetNone => {
            query_builder = query_builder.bind(None::<i32>);
        }
        _ => {}
    }

    let result = query_builder.fetch_one(pool).await.map_err(|e| {
        let err_msg = format!("Update(Record {}): {}", record_id, e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    if !result.get_exist() {
        return Err(Arc::new(CustomError::NotFound("Record".to_string())));
    } else if !result.get_authorized() {
        return Err(Arc::new(CustomError::Unauthorized(
            "RecordRole".to_string(),
        )));
    } else if !result.get_category_exist() {
        return Err(Arc::new(CustomError::NotFound("Category".to_string())));
    } else if !result.get_asset_exist() {
        return Err(Arc::new(CustomError::NotFound("Asset".to_string())));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, Utc};
    use sqlx::PgPool;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{
            entity::{Record, UpdateRecord},
            repository::{get_record::get_by_id, save::save_record, update::update_record},
        },
        global::{constants::FieldUpdate, errors::CustomError},
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    async fn _save_sample(pool: &PgPool, user_id: i32) -> i64 {
        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );
        let new_id = save_record(&pool, user_id, record, None).await.unwrap();
        new_id
    }

    #[tokio::test]
    async fn check_update_record_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let change_amount: i32 = 15000;

        let new_id = _save_sample(&pool, user_id).await;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(change_amount),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, user_id, new_id, edit_record).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = get_by_id(&pool, user_id, new_id).await.unwrap();

        assert_eq!(row.get_amount(), change_amount);
        assert_eq!(row.get_memo(), &None);
    }

    #[tokio::test]
    async fn check_no_field_to_update() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;

        let new_id = _save_sample(&pool, user_id).await;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, user_id, new_id, edit_record).await;

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
    async fn check_record_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let no_id = -32;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(15000),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, user_id, no_id, edit_record).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_category_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;

        let new_id = _save_sample(&pool, user_id).await;
        let edit_record = UpdateRecord::new(
            FieldUpdate::Set(-32), // 없는 카테고리, 내 카테고리가 아닌 경우
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, user_id, new_id, edit_record).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_asset_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;

        let new_id = _save_sample(&pool, user_id).await;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(15000),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::Set(-32), // 없는 자산, 내 자산이 아닌 경우
        );

        // Act
        let result = update_record(&pool, user_id, new_id, edit_record).await;

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
    async fn check_updated_at_changed() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;

        let new_id = _save_sample(&pool, user_id).await;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(15000),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );
        let last_time = Utc::now().naive_utc();

        // Act
        let result = update_record(&pool, user_id, new_id, edit_record).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let updated_user = get_by_id(&pool, user_id, new_id).await.unwrap();
        assert!(last_time < updated_user.get_updated_at().unwrap())
    }
}

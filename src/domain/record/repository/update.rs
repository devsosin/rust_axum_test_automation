use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::domain::record::entity::{FieldUpdate, UpdateRecord};

pub(crate) struct UpdateRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait UpdateRecordRepo: Send + Sync {
    async fn update_record(&self, id: i64, edit_record: UpdateRecord) -> Result<(), String>;
}

impl UpdateRecordRepoImpl {
    pub(crate) fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UpdateRecordRepo for UpdateRecordRepoImpl {
    async fn update_record(&self, id: i64, edit_record: UpdateRecord) -> Result<(), String> {
        update_record(&self.pool, id, edit_record).await
    }
}

fn make_query(index: &mut i32, field_name: &str) -> String {
    *index += 1;
    format!("{} = ${}, ", field_name, index)
}

async fn update_record(pool: &PgPool, id: i64, edit_record: UpdateRecord) -> Result<(), String> {
    // check_validation

    let mut query: String = "UPDATE tb_record SET ".to_string();
    let mut index = 0;

    match edit_record.get_sub_category_id() {
        FieldUpdate::Set(_) => {
            query.push_str(&make_query(&mut index, "sub_category_id"));
        }
        _ => {}
    };
    match edit_record.get_amount() {
        FieldUpdate::Set(_) => {
            query.push_str(&make_query(&mut index, "amount"));
        }
        _ => {}
    };
    match edit_record.get_memo() {
        FieldUpdate::Set(_) => {
            query.push_str(&make_query(&mut index, "memo"));
        }
        _ => {}
    };
    match edit_record.get_target_dt() {
        FieldUpdate::Set(_) => {
            query.push_str(&make_query(&mut index, "target_dt"));
        }
        _ => {}
    }
    match edit_record.get_asset_id() {
        FieldUpdate::Set(_) => {
            query.push_str(&make_query(&mut index, "asset_id"));
        }
        FieldUpdate::SetNone => {
            query.push_str(&make_query(&mut index, "asset_id"));
        }
        FieldUpdate::NoChange => {}
    }

    if index == 0 {
        return Err("no fields to update".to_string());
    }

    query.pop();
    query.pop();
    index += 1;
    query.push_str(&format!(" WHERE id = ${}", index));

    let mut query_builder = sqlx::query(&query);

    match edit_record.get_sub_category_id() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        _ => (),
    };
    match edit_record.get_amount() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        _ => {}
    };
    match edit_record.get_memo() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        _ => {}
    };
    match edit_record.get_target_dt() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        _ => {}
    }
    match edit_record.get_asset_id() {
        FieldUpdate::Set(v) => {
            query_builder = query_builder.bind(v);
        }
        FieldUpdate::SetNone => {
            query_builder = query_builder.bind(None::<i32>);
        }
        FieldUpdate::NoChange => {}
    }

    let result = query_builder.bind(id).execute(pool).await.map_err(|e| {
        let err_msg = format!("Update(Record){}: {}", id, e);
        tracing::error!("{}", err_msg);
        err_msg
    })?;

    if result.rows_affected() == 0 {
        return Err(format!("Update(Record){}: not found", id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{
            entity::{FieldUpdate, Record, UpdateRecord},
            repository::{get_record::get_by_id, save::save_record, update::update_record},
        },
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let change_amount: i32 = 15000;

        let new_id = save_record(&pool, record, None).await.unwrap();
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(change_amount),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, new_id, edit_record).await;
        assert!(result.clone().map_err(|e| println!("{}", e)).is_ok());

        // Assert
        let row = get_by_id(&pool, new_id).await.unwrap();

        assert_eq!(row.get_amount(), change_amount);
        assert_eq!(row.get_memo(), &None);
    }

    #[tokio::test]
    async fn check_no_field_to_update() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let new_id = save_record(&pool, record, None).await.unwrap();
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, new_id, edit_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let id = -32;
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(15000),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, id, edit_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_category_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let new_id = save_record(&pool, record, None).await.unwrap();
        let edit_record = UpdateRecord::new(
            FieldUpdate::Set(-32),
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
            FieldUpdate::NoChange,
        );

        // Act
        let result = update_record(&pool, new_id, edit_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_asset_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let new_id = save_record(&pool, record, None).await.unwrap();
        let edit_record = UpdateRecord::new(
            FieldUpdate::NoChange,
            FieldUpdate::Set(15000),
            FieldUpdate::SetNone,
            FieldUpdate::NoChange,
            FieldUpdate::Set(-32),
        );

        // Act
        let result = update_record(&pool, new_id, edit_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_category_valid() {
        // 해당 카테고리 권한
        todo!()

        // Arrange

        // Act

        // Assert
    }

    #[tokio::test]
    async fn check_asset_valid() {
        // 해당 자산 권한
        todo!()

        // Arrange

        // Act

        // Assert
    }
}

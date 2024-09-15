use std::sync::Arc;

use axum::async_trait;
use sqlx::{Error, PgPool};

use crate::{domain::record::entity::Record, global::errors::CustomError};

pub struct SaveRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveRecordRepo: Send + Sync {
    async fn save_record(
        &self,
        record: Record,
        connect_ids: Option<Vec<i32>>,
    ) -> Result<i64, Arc<CustomError>>;

    async fn validate_connect_ids(
        &self, 
        connect_ids: &Option<Vec<i32>>,
    ) -> Result<(), Arc<CustomError>>;
}

impl SaveRecordRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveRecordRepo for SaveRecordRepoImpl {
    async fn save_record(
        &self,
        record: Record,
        connect_ids: Option<Vec<i32>>,
    ) -> Result<i64, Arc<CustomError>> {
        save_record(&self.pool, record, connect_ids).await
    }

    async fn validate_connect_ids(
        &self, 
        connect_ids: &Option<Vec<i32>>,
    ) -> Result<(), Arc<CustomError>> {
        validate_connect_ids(&self.pool, &connect_ids).await
    }
    
}

async fn validate_connect_ids(
    pool: &PgPool,
    connect_ids: &Option<Vec<i32>>,
) -> Result<(), Arc<CustomError>> {
    if let Some(ids) = connect_ids {
        let invalid_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM (
                SELECT unnest($1::int[]) AS input_ids
            ) AS un
            WHERE un.input_ids NOT IN (SELECT id FROM tb_connect);
            "#,
        )
        .bind(ids)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(SaveRecord-validation): {:?}", &e); 
            tracing::error!("{}", err_msg);

            let err = match e {
                Error::Database(_) => CustomError::DatabaseError(e),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

        if invalid_count > 0 {
            return Err(Arc::new(CustomError::ValidationError("Connect".to_string())));
        }
    }

    Ok(())
}

pub async fn save_record(
    pool: &PgPool,
    record: Record,
    connect_ids: Option<Vec<i32>>,
) -> Result<i64, Arc<CustomError>> {
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        -- tb_record에 삽입
        WITH inserted_record AS (
            INSERT INTO tb_record (book_id, sub_category_id, amount, memo, target_dt, created_at, asset_id) 
            VALUES ($1, $2, $3, $4, $5, NOW(), $6)
            RETURNING id
        ),
        inserted_connect AS (
            -- 조건부로 연결 작업 수행
            INSERT INTO tb_record_connect (record_id, connect_id)
                SELECT inserted_record.id, connect.id
                FROM inserted_record
                JOIN tb_connect AS connect ON connect.id = ANY($7::int[])
                WHERE $7 IS NOT NULL -- connect_ids가 NULL이 아닌 경우에만 수행
            RETURNING record_id
        )
        SELECT COALESCE((SELECT record_id FROM inserted_connect), (SELECT id FROM inserted_record)) AS id;
    "#,
    )
    .bind(record.get_book_id())
    .bind(record.get_sub_category_id())
    .bind(record.get_amount())
    .bind(record.get_memo())
    .bind(record.get_target_dt())
    .bind(record.get_asset_id())
    .bind(connect_ids)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Save(Record): {:?}", e);
        tracing::error!("{}", err_msg);
        
        let err = match e {
            Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    Ok(id)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{entity::Record, repository::save::{save_record, validate_connect_ids}},
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_save_record_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, record, Some(vec![1])).await;
        let inserted_id = result.map_err(|e| println!("{:?}", e)).unwrap();

        // Assert
        let row = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(inserted_id, row.get_id());
        assert_eq!(&None, row.get_memo());
    }

    #[tokio::test]
    async fn check_save_record_success_without_connect() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, record, None).await;
        let inserted_id = result.map_err(|e| println!("{:?}", e)).unwrap();

        // Assert
        let row = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(inserted_id, row.get_id());
    }

    #[tokio::test]
    async fn check_fail_no_category() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            1,
            -32, // 없는 카테고리
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, record, None).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_fail_no_book() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            -32, // 없는 가계부
            18,
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, record, None).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_fail_no_asset() {
        // Arrange
        let pool = create_connection_pool().await;

        let record = Record::new(
            -32, // 없는 가계부
            18,
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            Some(-32),
        );

        // Act
        let result = save_record(&pool, record, None).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_no_connect() {
        // Arrange
        let pool = create_connection_pool().await;
        
        let new_connects = Some(vec![1, -32]);

        // Act
        let result = validate_connect_ids(&pool, &new_connects).await;
        assert!(result.is_err());
    }
}

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
        user_id: i32,
        record: Record,
        connect_ids: Option<Vec<i32>>,
    ) -> Result<i64, Box<CustomError>>;
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
        user_id: i32,
        record: Record,
        connect_ids: Option<Vec<i32>>,
    ) -> Result<i64, Box<CustomError>> {
        save_record(&self.pool, user_id, record, connect_ids).await
    }
    
}

#[derive(Debug, sqlx::FromRow)]
struct InsertRecord {
    is_authroized: bool,
    record_id: Option<i64>,
    connect_ids: Vec<i32>
}

impl InsertRecord {
    pub fn get_authroized(&self) -> bool {
        self.is_authroized
    }
    pub fn get_record_id(&self) -> Option<i64> {
        self.record_id
    }
    pub fn get_connects(&self) -> &Vec<i32> {
        &self.connect_ids
    }
}

pub async fn save_record(
    pool: &PgPool,
    user_id: i32,
    record: Record,
    connect_ids: Option<Vec<i32>>,
) -> Result<i64, Box<CustomError>> {
    let result = sqlx::query_as::<_, InsertRecord>(
        r#"
        WITH AuthorityCheck AS (
            SELECT book_id
            FROM tb_user_book_role
            WHERE user_id = $1 AND book_id = $2 AND role != 'viewer'
        ),
        InsertRecord AS (
            INSERT INTO tb_record (book_id, sub_category_id, amount, memo, target_dt, created_at, asset_id) 
                SELECT (book_id, $3, $4, $5, $6, NOW(), $7)
                    FROM AuthorityCheck
                    WHERE book_id IS NOT NULL
            RETURNING id
        ),
        ValidConnects AS (
            SELECT id
            FROM tb_connect
            WHERE id = ANY($8::int[])
        ),
        InsertConnect AS (
            INSERT INTO tb_record_connect (record_id, connect_id)
                SELECT r.id, vc.id
                FROM InsertRecord AS r
                CROSS JOIN ValidConnects AS vc
            RETURNING connect_id
        )
        SELECT 
            (SELECT id FROM InsertRecord) AS record_id,
            EXISTS (SELECT 1 FROM AuthorityCheck) AS is_authorized,
            ARRAY(SELECT connect_id FROM InsertConnect) AS connect_ids;
    "#,
    )
    .bind(user_id)
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
        Box::new(err)
    })?;
    
    if !result.get_authroized() {
        return Err(Box::new(CustomError::Unauthorized("RecordRole".to_string())))
    } // 카테고리 체크

    // 커넥트 반환
    result.get_connects();

    Ok(result.get_record_id().unwrap())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{entity::Record, repository::save::save_record}, global::errors::CustomError,
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
        
        let user_id = 1;
        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, user_id, record, Some(vec![1])).await;
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
    async fn check_success_without_connect() {
        // Arrange
        let pool = create_connection_pool().await;
        
        let user_id = 1;
        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, user_id, record, None).await;
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
    async fn check_no_category() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let record = Record::new(
            1,
            -32, // 없는 카테고리, 다른 사람 카테고리
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, user_id, record, None).await;

        // Assert
        // Not Found -> 권한 없는 카테고리 접근 제한
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
        
    }

    #[tokio::test]
    async fn check_book_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let record = Record::new(
            -32, // 존재하지 않는 가계부
            18,
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, user_id, record, None).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_no_asset() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let record = Record::new(
            -32, // 없는 가계부
            18,
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            Some(-32),
        );

        // Act
        let result = save_record(&pool, user_id, record, None).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::DatabaseError(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_no_role() {
        // Arrange
        let pool = create_connection_pool().await;

        // ref) init.sql
        let user_id = 2;
        let record = Record::new(
            1, // 읽기전용 가계부
            18,
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        // Act
        let result = save_record(&pool, user_id, record, None).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}

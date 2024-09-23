use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{
    domain::record::entity::{Record, Search},
    global::errors::CustomError,
};

pub struct GetRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetRecordRepo: Send + Sync {
    async fn get_list(
        &self,
        user_id: i32,
        book_id: i32,
        search_query: Search,
    ) -> Result<Vec<Record>, Box<CustomError>>;
    async fn get_by_id(&self, user_id: i32, record_id: i64) -> Result<Record, Box<CustomError>>;
}

impl GetRecordRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl GetRecordRepo for GetRecordRepoImpl {
    async fn get_list(
        &self,
        user_id: i32,
        book_id: i32,
        search_query: Search,
    ) -> Result<Vec<Record>, Box<CustomError>> {
        get_list(&self.pool, user_id, book_id, search_query).await
    }
    async fn get_by_id(&self, user_id: i32, record_id: i64) -> Result<Record, Box<CustomError>> {
        get_by_id(&self.pool, user_id, record_id).await
    }
}

async fn get_list(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    search_query: Search,
) -> Result<Vec<Record>, Box<CustomError>> {
    let mut query = "
        SELECT r.* 
        FROM tb_record AS r
        JOIN tb_book AS b ON b.id = r.book_id
        JOIN tb_user_book_role AS br ON b.id = br.book_id
        JOIN tb_sub_category AS sc ON r.sub_category_id = sc.id
        WHERE br.user_id = $1 AND b.id = $2
            AND r.target_dt BETWEEN $3 AND $4
    "
    .to_string();

    let mut bind_idx = 5;
    if let Some(_) = search_query.get_sub_id() {
        query.push_str(format!("AND sc.id = ${} ", bind_idx).as_str());
        bind_idx += 1;
    }
    if let Some(_) = search_query.get_base_id() {
        query.push_str(format!("AND sc.base_id = ${}", bind_idx).as_str());
    }

    let mut query_builder = sqlx::query_as::<_, Record>(&query)
        .bind(user_id)
        .bind(book_id)
        .bind(search_query.get_start_dt())
        .bind(search_query.get_end_dt());

    if let Some(sub_id) = search_query.get_sub_id() {
        query_builder = query_builder.bind(sub_id);
    }
    if let Some(base_id) = search_query.get_base_id() {
        query_builder = query_builder.bind(base_id);
    }

    let rows = query_builder.fetch_all(pool).await.map_err(|e| {
        let err_msg = format!("Error(GetRecords): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    Ok(rows)
}

pub async fn get_by_id(
    pool: &PgPool,
    user_id: i32,
    record_id: i64,
) -> Result<Record, Box<CustomError>> {
    let row = sqlx::query_as::<_, Record>(
        "
        SELECT r.* FROM tb_record AS r
        JOIN tb_book AS b ON b.id = r.book_id
        JOIN tb_user_book_role AS br ON b.id = br.book_id
        WHERE br.user_id = $1 AND r.id = $2
    ",
    )
    .bind(user_id)
    .bind(record_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetRecord {}): {:?}", record_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            sqlx::Error::RowNotFound => CustomError::NotFound("Record".to_string()),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    Ok(row)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{
            entity::{Record, Search},
            repository::get_record::{get_by_id, get_list},
        },
        global::errors::CustomError,
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_list_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let book_id = 1;
        let start_dt = NaiveDate::parse_from_str("2024-09-01", "%Y-%m-%d").unwrap();
        let end_dt = NaiveDate::parse_from_str("2024-10-01", "%Y-%m-%d").unwrap();
        let search_query = Search::new(start_dt, end_dt, None, None);

        // Act
        let result = get_list(&pool, user_id, book_id, search_query).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let rows = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(result.len(), rows.len());
    }

    #[tokio::test]
    async fn check_target_dt() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 3;
        let book_id = 2;
        let start_dt = NaiveDate::parse_from_str("2024-09-01", "%Y-%m-%d").unwrap();
        let end_dt = NaiveDate::parse_from_str("2024-10-01", "%Y-%m-%d").unwrap();
        let search_query = Search::new(start_dt, end_dt, None, None);

        // Act
        let result = get_list(&pool, user_id, book_id, search_query).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        // ref) init.sql
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn check_base_category() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 3;
        let book_id = 2;
        let start_dt = NaiveDate::parse_from_str("2024-09-01", "%Y-%m-%d").unwrap();
        let end_dt = NaiveDate::parse_from_str("2024-10-01", "%Y-%m-%d").unwrap();
        let search_query = Search::new(start_dt, end_dt, Some(8), None);

        // Act
        let result = get_list(&pool, user_id, book_id, search_query).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        // ref) init.sql
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn check_sub_category() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 3;
        let book_id = 2;
        let start_dt = NaiveDate::parse_from_str("2024-09-01", "%Y-%m-%d").unwrap();
        let end_dt = NaiveDate::parse_from_str("2024-10-01", "%Y-%m-%d").unwrap();
        let search_query = Search::new(start_dt, end_dt, None, Some(16));

        // Act
        let result = get_list(&pool, user_id, book_id, search_query).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        // ref) init.sql
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn check_get_by_id_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        // save_record
        let record_id = 1i64;

        // Act
        let result = get_by_id(&pool, user_id, record_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE id = $1")
            .bind(record_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.get_amount(), row.get_amount())
    }

    #[tokio::test]
    async fn check_get_by_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let no_id = -32i64;

        // Act
        let result = get_by_id(&pool, user_id, no_id).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}

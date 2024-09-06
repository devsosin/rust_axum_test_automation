use sqlx::PgPool;

use crate::domain::book::entity::BookType;

pub async fn get_book_type_by_name(pool: &PgPool, name: &str) -> Result<BookType, String> {
    let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            // row not found error
            let err_msg = format!("Error(GetBookTypeByName): {}", e);
            tracing::error!("{:?}", err_msg);
            // err_msg
            "없는 카테고리입니다.".to_string()
        })?;

    Ok(row)
}

#[cfg(test)]
mod tests {

    use sqlx::Acquire;

    use crate::{config::database::create_connection_pool, domain::book::entity::BookType};

    use super::get_book_type_by_name;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false);
    }

    #[tokio::test]
    async fn check_get_book_type_success() {
        // Arrange
        let pool = create_connection_pool().await;
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();

        let name = "개인";

        // Act
        let result = get_book_type_by_name(&pool, name).await;
        // 결과 제대로 받아왔는지 체크
        assert!(result.is_ok());
        let book_type = result.unwrap();
        let type_id = book_type.get_id();

        // Assert
        let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
            .bind(name)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                // Error: RowNotFound
                let err_message = format!("Error: {:?}", e);
                tracing::error!("{:?}", err_message);
                err_message
            })
            .unwrap();

        assert_eq!(row.get_id(), type_id);

        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn check_book_type_not_exist() {
        // Arrange
        let pool = create_connection_pool().await;
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();

        let name = "없는 이름";

        // Act
        let result = get_book_type_by_name(&pool, name).await;

        // Assert: 존재하지 않는 것은 Err(RowNotFound) 반환
        assert!(result.is_err());

        transaction.rollback().await.unwrap();
    }
}

use sqlx::PgPool;

use crate::domain::book::entity::Book;

pub async fn get_books(pool: &PgPool) -> Result<Vec<Book>, String> {
    todo!()
}

pub async fn get_book(pool: &PgPool, id: i32) -> Result<Book, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::book::{
            entity::Book,
            repository::get_book::{get_book, get_books},
        },
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false);
    }

    // test snippet
    #[tokio::test]
    async fn check_get_books_success() {
        // Arrange
        let pool = create_connection_pool().await;

        // Act
        let result = get_books(&pool).await;
        assert!(result.is_ok());

        // Assert (나중에 user_id로 -> book_role 체크)
        let result = result.unwrap();
        let rows = sqlx::query("SELECT * FROM tb_book")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(rows.len(), result.len())
    }

    #[tokio::test]
    async fn check_get_books_failure() {
        // 실패 케이스 ?
        // 해당 role user_id가 없으면 빈 벡터 반환 -30 같은거
        todo!()
    }

    #[tokio::test]
    async fn check_get_book_success() {
        // Arange
        let pool = create_connection_pool().await;
        let id: i32 = 1;

        // Act
        let result = get_book(&pool, id).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, Book>("SELECT * FROM tb_book WHERE ID = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e)
            .unwrap();

        assert_eq!(result.get_name(), row.get_name());
    }

    #[tokio::test]
    async fn check_get_book_failure() {
        // Arange
        let pool = create_connection_pool().await;
        let id: i32 = -32;

        // Act
        let result = get_book(&pool, id).await;

        // Assert -> row not found error
        assert!(result.is_err());
    }
}

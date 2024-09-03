use sqlx::PgPool;

use super::dto::request::NewBook;

// 에러 핸들링 미들웨어
pub async fn create_book(pool: &PgPool, new_book: &NewBook, type_id: i16) -> Result<bool, String> {
    sqlx::query(
        r#"
        INSERT INTO tb_book (name, type_id) VALUES
        ($1, $2)
        "#,
    )
    .bind(new_book.get_name())
    .bind(type_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error: Inserting new book: {:?}", e);
        let err_message = format!("가계부 생성 중 오류가 발생했습니다.");
        err_message
    })?;

    Ok(true)
}
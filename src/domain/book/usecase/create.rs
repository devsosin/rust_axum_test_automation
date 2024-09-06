use std::sync::Arc;

use super::{
    super::repository::{BookRepository, BookRepositoryImpl},
    NewBook,
};

pub async fn create_book(
    repository: Arc<BookRepositoryImpl>,
    new_book: &NewBook,
    type_id: i16,
) -> Result<i32, String> {
    repository.save_book(new_book, type_id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::mock;

    use crate::domain::book::{dto::request::NewBook, entity::Book, repository::BookRepository};

    mock! {
        pub BookRepositoryImpl {}

        #[async_trait]
        impl BookRepository for BookRepositoryImpl {
            async fn get_book(&self, id: i32) -> Result<Option<Book>, String>;
            async fn save_book(&self, new_book: &NewBook, type_id: i16) -> Result<i32, String>;
            async fn delete_book(&self, id: i32) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn check_repository() {}
}

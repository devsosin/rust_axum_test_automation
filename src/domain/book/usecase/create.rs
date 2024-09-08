use std::sync::Arc;

use axum::async_trait;

use crate::domain::book::{dto::request::NewBook, repository::save::SaveBookRepo};

pub struct CreateBookUsecaseImpl<T: SaveBookRepo> {
    repository: Arc<T>,
}

#[async_trait]
pub trait CreateBookUsecase: Send + Sync {
    async fn create_book(&self, new_book: &NewBook) -> Result<i32, String>;
}

impl<T> CreateBookUsecaseImpl<T>
where
    T: SaveBookRepo,
{
    pub fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateBookUsecase for CreateBookUsecaseImpl<T>
where
    T: SaveBookRepo,
{
    async fn create_book(&self, new_book: &NewBook) -> Result<i32, String> {
        create_book(self.repository.clone(), new_book).await
    }
}

pub async fn create_book<T: SaveBookRepo>(
    repository: Arc<T>,
    new_book: &NewBook,
) -> Result<i32, String> {
    repository
        .save_book(new_book.get_name(), new_book.get_book_type())
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{
        dto::request::NewBook,
        entity::BookType,
        repository::{save::SaveBookRepo, GetBookTypeRepo},
    };

    use super::{CreateBookUsecase, CreateBookUsecaseImpl};

    mock! {
        pub SaveBookRepoImpl {}

        #[async_trait]
        impl SaveBookRepo for SaveBookRepoImpl {
            async fn save_book(&self, name: &str, book_type: &str) -> Result<i32, String>;
        }
    }
    mock! {
        pub GetBookTypeRepoImpl {}

        #[async_trait]
        impl GetBookTypeRepo for GetBookTypeRepoImpl {
            async fn get_book_types(&self) -> Result<Vec<BookType>, String>;
            async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String>;
        }
    }

    #[tokio::test]
    async fn check_create_book_success() {
        // Arrange
        let mut mock_repo = MockSaveBookRepoImpl::new();

        let new_book = NewBook::new("새 가계부".to_string(), "일반".to_string());

        // 모킹 동작 설정
        mock_repo
            .expect_save_book()
            .with(
                predicate::eq(new_book.get_name().to_owned()),
                predicate::eq(new_book.get_book_type().to_owned()),
            )
            .returning(|_, _| Ok(1)); // 성공 시 id 1반환

        let usecase = CreateBookUsecaseImpl::new(Arc::new(mock_repo));

        // Act
        let book_id = usecase.create_book(&new_book).await;

        // Assert
        assert_eq!(book_id, Ok(1));
    }

    /*
     * "비즈니스 로직 처리 상 book_type이 잘못되었을 경우"
     */
    #[tokio::test]
    async fn check_create_book_failure() {
        // Arrnge
        let mut mock_repo = MockSaveBookRepoImpl::new();

        let new_book = NewBook::new("새 가계부".to_string(), "개인".to_string());

        mock_repo
            .expect_save_book()
            .with(
                predicate::eq(new_book.get_name().to_owned()),
                predicate::eq(new_book.get_book_type().to_owned()),
            )
            .returning(|_, _| Err("에러가 발생했습니다.".to_string())); // repo단위 에러 반환

        let usecase = CreateBookUsecaseImpl::new(Arc::new(mock_repo));

        // Act
        let result = usecase.create_book(&new_book).await;

        // Assert
        assert_eq!(result, Err("에러가 발생했습니다.".to_string()));
    }
}

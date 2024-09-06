use std::sync::Arc;

use super::{super::repository::BookRepository, NewBook};
use crate::domain::book::repository::BookTypeRepository;

pub async fn create_book<T: BookRepository, U: BookTypeRepository>(
    book_repo: Arc<T>,
    type_repo: Arc<U>,
    new_book: &NewBook,
) -> Result<i32, String> {
    let type_id = type_repo
        .get_book_type_by_name(new_book.get_name())
        .await
        // map_err -> 잘못된 카테고리 이름 입력 시 no row 에러 반환
        .map_err(|e| e)?
        .get_id();

    book_repo.save_book(new_book.get_name(), type_id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{
        dto::request::NewBook,
        entity::{Book, BookType},
        repository::{BookRepository, BookTypeRepository},
        usecase::{BookUsecase, BookUsecaseImpl},
    };

    mock! {
        pub BookRepositoryImpl {}

        #[async_trait]
        impl BookRepository for BookRepositoryImpl {
            async fn get_book(&self, id: i32) -> Result<Book, String>;
            async fn save_book(&self, name: &str, type_id: i16) -> Result<i32, String>;
            async fn delete_book(&self, id: i32) -> Result<(), String>;
        }
    }
    mock! {
        pub BookTypeRepositoryImpl {}

        #[async_trait]
        impl BookTypeRepository for BookTypeRepositoryImpl {
            async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String>;
        }
    }

    #[tokio::test]
    async fn check_create_book_success() {
        // Arrange
        let mut mock_book_repo = MockBookRepositoryImpl::new();
        let mut mock_type_repo = MockBookTypeRepositoryImpl::new();

        let new_book = NewBook::new("새 가계부".to_string(), "일반".to_string());

        let type_id: i16 = 1;

        // 모킹 동작 설정
        mock_book_repo
            .expect_save_book()
            .with(
                predicate::eq(new_book.get_name().to_owned()),
                predicate::eq(type_id),
            )
            .returning(|_, _| Ok(1)); // 성공 시 id 1반환

        mock_type_repo
            .expect_get_book_type_by_name()
            .with(predicate::eq(new_book.get_name().to_owned()))
            .returning(|_| Ok(BookType::test_new()));

        // Act
        let usecase = BookUsecaseImpl::new(Arc::new(mock_book_repo), Arc::new(mock_type_repo));

        // Assert
        let book_id = usecase.create_book(&new_book).await;
        assert_eq!(book_id, Ok(1));
    }

    /*
     * "비즈니스 로직 처리 상 book_type이 잘못되었을 경우"
     */
    #[tokio::test]
    async fn check_create_book_failure() {
        // Arrnge
        let mut mock_book_repo = MockBookRepositoryImpl::new();
        let mut mock_type_repo = MockBookTypeRepositoryImpl::new();

        let new_book = NewBook::new("새 가계부".to_string(), "개인".to_string());
        let type_id: i16 = 1;

        mock_book_repo
            .expect_save_book()
            .with(
                predicate::eq(new_book.get_name().to_owned()),
                predicate::eq(type_id),
            )
            .returning(|_, _| Err("에러가 발생했습니다.".to_string())); // repo단위 에러 반환

        mock_type_repo
            .expect_get_book_type_by_name()
            .with(predicate::eq(new_book.get_name().to_owned()))
            .returning(|_| Ok(BookType::test_new()));

        // Act
        let usecase = BookUsecaseImpl::new(Arc::new(mock_book_repo), Arc::new(mock_type_repo));

        // Assert
        let result = usecase.create_book(&new_book).await;
        assert_eq!(result, Err("에러가 발생했습니다.".to_string()));
    }
}

use std::sync::Arc;

use axum::async_trait;

use crate::domain::book::{
    dto::request::NewBook,
    repository::{save::SaveBookRepo, GetBookTypeRepo},
};

pub struct CreateBookUsecaseImpl<T: SaveBookRepo, U: GetBookTypeRepo> {
    book_repo: Arc<T>,
    type_repo: Arc<U>,
}

#[async_trait]
pub trait CreateBookUsecase: Send + Sync {
    async fn create_book(&self, new_book: &NewBook) -> Result<i32, String>;
}

impl<T, U> CreateBookUsecaseImpl<T, U>
where
    T: SaveBookRepo,
    U: GetBookTypeRepo,
{
    pub fn new(book_repo: Arc<T>, type_repo: Arc<U>) -> Self {
        Self {
            book_repo,
            type_repo,
        }
    }
}

#[async_trait]
impl<T, U> CreateBookUsecase for CreateBookUsecaseImpl<T, U>
where
    T: SaveBookRepo,
    U: GetBookTypeRepo,
{
    async fn create_book(&self, new_book: &NewBook) -> Result<i32, String> {
        create_book(self.book_repo.clone(), self.type_repo.clone(), new_book).await
    }
}

pub async fn create_book<T: SaveBookRepo, U: GetBookTypeRepo>(
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
        entity::BookType,
        repository::{save::SaveBookRepo, GetBookTypeRepo},
    };

    use super::{CreateBookUsecase, CreateBookUsecaseImpl};

    mock! {
        pub SaveBookRepoImpl {}

        #[async_trait]
        impl SaveBookRepo for SaveBookRepoImpl {
            async fn save_book(&self, name: &str, type_id: i16) -> Result<i32, String>;
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
        let mut mock_book_repo = MockSaveBookRepoImpl::new();
        let mut mock_type_repo = MockGetBookTypeRepoImpl::new();

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

        let usecase =
            CreateBookUsecaseImpl::new(Arc::new(mock_book_repo), Arc::new(mock_type_repo));

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
        let mut mock_book_repo = MockSaveBookRepoImpl::new();
        let mut mock_type_repo = MockGetBookTypeRepoImpl::new();

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

        let usecase =
            CreateBookUsecaseImpl::new(Arc::new(mock_book_repo), Arc::new(mock_type_repo));

        // Act
        let result = usecase.create_book(&new_book).await;

        // Assert
        assert_eq!(result, Err("에러가 발생했습니다.".to_string()));
    }
}

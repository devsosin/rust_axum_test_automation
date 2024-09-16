use axum::async_trait;

use crate::{
    domain::book::{entity::Book, repository::get_book::GetBookRepo},
    global::errors::CustomError,
};

pub struct ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    repository: T,
}

#[async_trait]
pub trait ReadBookUsecase: Send + Sync {
    async fn read_books(&self, user_id: i32) -> Result<Vec<Book>, Box<CustomError>>;
    async fn read_book(&self, user_id: i32, book_id: i32) -> Result<Book, Box<CustomError>>;
}

impl<T> ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadBookUsecase for ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    async fn read_books(&self, user_id: i32) -> Result<Vec<Book>, Box<CustomError>> {
        // Dereferencing Arc to get to the inner T
        read_books(&self.repository, user_id).await
    }
    async fn read_book(&self, user_id: i32, book_id: i32) -> Result<Book, Box<CustomError>> {
        read_book(&self.repository, user_id, book_id).await
    }
}

async fn read_books<T: GetBookRepo>(
    repository: &T,
    user_id: i32,
) -> Result<Vec<Book>, Box<CustomError>> {
    match repository.get_books(user_id).await {
        Ok(books) => {
            if books.len() == 0 {
                return Err(Box::new(CustomError::NotFound("Book".to_string())));
            }
            Ok(books)
        }
        Err(e) => Err(e),
    }
}

async fn read_book<T: GetBookRepo>(
    repository: &T,
    user_id: i32,
    book_id: i32,
) -> Result<Book, Box<CustomError>> {
    repository.get_book(user_id, book_id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{entity::Book, repository::get_book::GetBookRepo};
    use crate::global::errors::CustomError;

    use super::{ReadBookUsecase, ReadBookUsecaseImpl};

    mock! {
        GetBookRepoImpl {}

        #[async_trait]
        impl GetBookRepo for GetBookRepoImpl {
            async fn get_books(&self, user_id: i32) -> Result<Vec<Book>, Box<CustomError>>;
            async fn get_book(&self, user_id: i32, book_id: i32) -> Result<Book, Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_read_books_success() {
        // Arrange
        let user_id = 1;
        let books = vec![Book::new("읽기용 가계부".to_string(), 1).id(1)];
        let ret_books = books.clone();

        let mut mock_repo = MockGetBookRepoImpl::new();
        // 모킹 동작 설정
        mock_repo
            .expect_get_books()
            .with(predicate::eq(user_id))
            .returning(move |i| Ok(ret_books.clone())); // 성공 시 books 반환

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_books(user_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(books.len(), result.len());
    }

    #[tokio::test]
    async fn check_read_book_success() {
        // Arrange
        let user_id = 1;
        let book_id = 1;

        let mut mock_repo = MockGetBookRepoImpl::new();
        mock_repo
            .expect_get_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(|_, i| Ok(Book::new("새 가계부".to_string(), 1).id(i)));

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_book(user_id, book_id).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id().unwrap(), book_id);
    }

    #[tokio::test]
    async fn check_read_book_not_found() {
        // Arrange
        let book_id = 1;
        let user_id = -32;

        let mut mock_repo = MockGetBookRepoImpl::new();
        mock_repo
            .expect_get_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Book".to_string()))));

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_book(user_id, book_id).await;

        // Assert
        assert!(result.is_err())
    }
}

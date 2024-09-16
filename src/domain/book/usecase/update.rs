use axum::async_trait;

use crate::{
    domain::book::{
        dto::request::EditBook, entity::BookUpdate, repository::update::UpdateBookRepo,
    },
    global::errors::CustomError,
};

pub struct UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    repository: T,
}

#[async_trait]
pub trait UpdateBookUsecase: Send + Sync {
    async fn update_book(
        &self,
        user_id: i32,
        book_id: i32,
        edit_book: EditBook,
    ) -> Result<(), Box<CustomError>>;
}

impl<T> UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> UpdateBookUsecase for UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    async fn update_book(
        &self,
        user_id: i32,
        book_id: i32,
        edit_book: EditBook,
    ) -> Result<(), Box<CustomError>> {
        update_book(&self.repository, user_id, book_id, edit_book).await
    }
}

pub async fn update_book<T: UpdateBookRepo>(
    repository: &T,
    user_id: i32,
    book_id: i32,
    edit_book: EditBook,
) -> Result<(), Box<CustomError>> {
    let book_update: BookUpdate = edit_book.id(book_id).to_entity(user_id);
    repository.update_book(book_update).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::entity::BookUpdate;
    use crate::domain::book::{
        dto::request::EditBook, repository::update::UpdateBookRepo, usecase::update::update_book,
    };

    use crate::global::errors::CustomError;

    mock! {
        UpdateBookRepoImpl {}

        #[async_trait]
        impl UpdateBookRepo for UpdateBookRepoImpl {
            async fn update_book(&self, book_update: BookUpdate) -> Result<(), Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_book_update_success() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let edit_book = EditBook::new("수정 가계부".to_string());
        let book_update = edit_book.clone().id(book_id).to_entity(user_id);

        let mut mock_repo = MockUpdateBookRepoImpl::new();
        mock_repo
            .expect_update_book()
            .with(predicate::eq(book_update.clone()))
            .returning(|_| Ok(()));

        // Act
        let result = update_book(&mock_repo, user_id, book_id, edit_book).await;

        // Assert
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn check_book_update_failure() {
        // Arrange
        let user_id = 1;
        let no_id = -32;
        let edit_book = EditBook::new("수정 가계부".to_string());
        let book_update = edit_book.clone().id(no_id).to_entity(user_id);

        let mut mock_repo = MockUpdateBookRepoImpl::new();
        mock_repo
            .expect_update_book()
            .with(predicate::eq(book_update.clone()))
            .returning(|_| Err(Box::new(CustomError::NotFound("Book".to_string())))); // duplicate, authroized

        // Act
        let result = update_book(&mock_repo, user_id, no_id, edit_book).await;

        // Assert
        assert!(result.is_err())
    }
}

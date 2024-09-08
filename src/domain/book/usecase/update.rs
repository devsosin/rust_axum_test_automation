use std::sync::Arc;

use axum::async_trait;

use crate::domain::book::{dto::request::EditBook, repository::update::UpdateBookRepo};

pub struct UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub trait UpdateBookUsecase: Send + Sync {
    async fn update_book(&self, id: i32, edit_book: &EditBook) -> Result<(), String>;
}

impl<T> UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    pub fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> UpdateBookUsecase for UpdateBookUsecaseImpl<T>
where
    T: UpdateBookRepo,
{
    async fn update_book(&self, id: i32, edit_book: &EditBook) -> Result<(), String> {
        update_book(&*self.repository, id, edit_book).await
    }
}

pub async fn update_book(
    repository: &impl UpdateBookRepo,
    id: i32,
    edit_book: &EditBook,
) -> Result<(), String> {
    repository.update_book(id, edit_book.get_name()).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{
        dto::request::EditBook, repository::update::UpdateBookRepo, usecase::update::update_book,
    };

    mock! {
        UpdateBookRepoImpl {}

        #[async_trait]
        impl UpdateBookRepo for UpdateBookRepoImpl {
            async fn update_book(&self, id: i32, name: &str) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn check_book_update_success() {
        // Arrange
        let id = 1;
        let target_book = EditBook::new("수정 가계부".to_string());

        let mut mock_repo = MockUpdateBookRepoImpl::new();
        mock_repo
            .expect_update_book()
            .with(
                predicate::eq(id),
                predicate::eq(target_book.get_name().to_owned()),
            )
            .returning(|_, _| Ok(()));

        // Act
        let result = update_book(&mock_repo, id, &target_book).await;

        // Assert
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn check_book_update_failure() {
        // Arrange
        let no_id = 15000;
        let target_book = EditBook::new("수정 가계부".to_string());

        let mut mock_repo = MockUpdateBookRepoImpl::new();
        mock_repo
            .expect_update_book()
            .with(
                predicate::eq(no_id),
                predicate::eq(target_book.get_name().to_owned()),
            )
            .returning(|_, _| Err("존재하지 않는 id입니다.".to_string()));

        // Act
        let result = update_book(&mock_repo, no_id, &target_book).await;

        // Assert
        assert!(result.is_err())
    }
}

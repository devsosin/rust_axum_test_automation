use axum::Error;
use super::{Book, BookRepository, NewBook};

pub struct BookRepositoryImpl {

}

impl BookRepositoryImpl {

}

impl BookRepository for BookRepositoryImpl {
    async fn get_book(&self, id: i64) -> Result<Option<Book>, Error> {
        todo!()
    }

    async fn save_book(&self, new_book: &NewBook) -> Result<(), Error> {
        todo!()
    }

    async fn delete_book(&self, id: i64) -> Result<(), Error> {
        todo!()
    }
}
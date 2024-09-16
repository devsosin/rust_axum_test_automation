use serde::{Deserialize, Serialize};

use crate::domain::book::entity::{Book, BookUpdate};

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct NewBook {
    name: String,
    type_id: i16,
}

impl NewBook {
    pub fn new(name: String, type_id: i16) -> Self {
        Self { name, type_id }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type_id(&self) -> i16 {
        self.type_id
    }

    pub fn to_entity(&self) -> Book {
        Book::new(self.name.to_owned(), self.type_id)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct EditBook {
    book_id: Option<i32>,
    name: String,
}

impl EditBook {
    pub fn new(name: String) -> Self {
        Self {
            book_id: None,
            name,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn id(mut self, id: i32) -> Self {
        self.book_id = Some(id);
        self
    }

    pub fn to_entity(&self, user_id: i32) -> BookUpdate {
        BookUpdate::new(user_id, self.book_id.unwrap(), self.name.to_string())
    }
}

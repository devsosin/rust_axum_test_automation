use serde::{Deserialize, Serialize};

use crate::domain::book::entity::Book;

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
    name: String,
}

impl EditBook {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

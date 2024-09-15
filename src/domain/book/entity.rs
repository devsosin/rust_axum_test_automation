use serde::{Deserialize, Serialize};

#[derive(Deserialize, sqlx::FromRow, Serialize, Clone, Debug, PartialEq)]
pub struct Book {
    id: Option<i32>,
    name: String,
    type_id: i16,
}

impl Book {
    pub fn new(name: String, type_id: i16) -> Self {
        Self {
            id: None,
            name,
            type_id,
        }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn get_id(&self) -> Option<i32> {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type_id(&self) -> i16 {
        self.type_id
    }
}

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub struct BookType {
    id: i16,
    name: String,
}

impl BookType {
    pub fn new(id: i16, name: String) -> Self {
        Self { id, name }
    }
    pub fn test_new() -> Self {
        Self {
            id: 1,
            name: "개인".to_string(),
        }
    }

    pub fn get_id(&self) -> i16 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub struct BookRole {
    user_id: i32,
    book_id: i32,
    role: String,
}

impl BookRole {
    pub fn new(user_id: i32, book_id: i32, role: String) -> Self {
        Self {
            user_id,
            book_id,
            role,
        }
    }
}

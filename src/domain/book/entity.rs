use serde::{Deserialize, Serialize};

#[derive(Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Book {
    id: i32,
    name: String,
    type_id: i16,
}

impl Book {
    pub fn test_new(id: i32) -> Self {
        Self {
            id,
            name: "테스트 가계부".to_string(),
            type_id: 1,
        }
    }

    pub fn get_id(&self) -> i32 {
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

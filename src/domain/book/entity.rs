use serde::{Deserialize, Serialize};

#[derive(Deserialize, sqlx::FromRow, Serialize, Clone, Debug, PartialEq)]
pub(crate) struct Book {
    id: Option<i32>,
    name: String,
    type_id: i16,
}

impl Book {
    pub(crate) fn new(id: Option<i32>, name: String, type_id: i16) -> Self {
        Self { id, name, type_id }
    }

    pub(crate) fn get_id(&self) -> Option<i32> {
        self.id
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_type_id(&self) -> i16 {
        self.type_id
    }
}

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub(crate) struct BookType {
    id: i16,
    name: String,
}

impl BookType {
    pub(crate) fn new(id: i16, name: String) -> Self {
        Self { id, name }
    }
    pub(crate) fn test_new() -> Self {
        Self {
            id: 1,
            name: "개인".to_string(),
        }
    }

    pub(crate) fn get_id(&self) -> i16 {
        self.id
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

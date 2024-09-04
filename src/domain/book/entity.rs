use serde::{Deserialize, Serialize};

#[derive(Deserialize, sqlx::FromRow, Serialize)]
pub struct Book {
    id: i32,
    name: String, 
    type_id: i16, 
}

impl Book {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type_id(&self) -> i16 {
        self.type_id
    }
}

#[derive(Deserialize)]
pub struct BookType {
    id: i16,
    name: String,
}
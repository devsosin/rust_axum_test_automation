use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone, PartialEq)]
pub struct Image {
    id: Option<i32>,
    original_name: String,
    image_key: String,
}

impl Image {
    pub fn new(original_name: String, image_key: String) -> Self {
        Self {
            id: None,
            original_name,
            image_key,
        }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn get_id(&self) -> i32 {
        self.id.unwrap()
    }

    pub fn get_original_name(&self) -> &str {
        &self.original_name
    }

    pub fn get_image_key(&self) -> &str {
        &self.image_key
    }
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewBook {
    name: String,
    book_type: String,
}

impl NewBook {
    pub fn new(name: String, book_type: String) -> Self {
        Self { name, book_type }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_book_type(&self) -> &str {
        &self.book_type
    }
}

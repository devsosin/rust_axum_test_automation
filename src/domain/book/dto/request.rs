use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewBook {
    name: String,
    book_type: String,
}

impl NewBook {
    pub fn new(name: &str, book_type: &str) -> Self {
        Self {
            name: name.to_owned(),
            book_type: book_type.to_owned(),
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_book_type(&self) -> &str {
        &self.book_type
    }
}
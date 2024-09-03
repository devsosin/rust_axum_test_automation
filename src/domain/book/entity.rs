use serde::Deserialize;

#[derive(Deserialize)]
pub struct Book {
    id: i64,
    name: String
}

#[derive(Deserialize)]
pub struct BookType {
    id: i16,
    name: String,
}
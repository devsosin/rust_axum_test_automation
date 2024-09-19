#[derive(Debug, sqlx::FromRow, PartialEq, Clone)]
pub struct BaseCategory {
    id: Option<i16>,
    type_id: i16,
    book_id: Option<i32>,
    is_record: bool,
    is_income: bool,
    name: String,
    color: String,
}

impl BaseCategory {
    pub fn new(
        type_id: i16,
        book_id: i32,
        is_record: bool,
        is_income: bool,
        name: String,
        color: String,
    ) -> Self {
        Self {
            id: None,
            type_id,
            book_id: Some(book_id),
            is_record,
            is_income,
            name,
            color,
        }
    }

    pub fn id(mut self, id: i16) -> Self {
        self.id = Some(id);
        self
    }

    pub fn get_id(&self) -> i16 {
        self.id.unwrap()
    }
    pub fn get_type_id(&self) -> i16 {
        self.type_id
    }
    pub fn get_book_id(&self) -> i32 {
        self.book_id.unwrap()
    }
    pub fn get_is_record(&self) -> bool {
        self.is_record
    }
    pub fn get_is_income(&self) -> bool {
        self.is_income
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_color(&self) -> &str {
        &self.color
    }
}

#[derive(Debug, sqlx::FromRow, PartialEq)]
pub struct SubCategory {
    id: Option<i32>,
    base_id: i16,
    name: String,
}

impl SubCategory {
    pub fn new(base_id: i16, name: String) -> Self {
        Self {
            id: None,
            base_id,
            name,
        }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn get_id(&self) -> i32 {
        self.id.unwrap()
    }

    pub fn get_base_id(&self) -> i16 {
        self.base_id
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

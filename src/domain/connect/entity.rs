use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow, PartialEq, Clone)]
pub struct Connect {
    id: Option<i32>,
    name: String,
}

impl Connect {
    pub fn new(name: String) -> Self {
        Self { id: None, name }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn get_id(&self) -> i32 {
        self.id.unwrap()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

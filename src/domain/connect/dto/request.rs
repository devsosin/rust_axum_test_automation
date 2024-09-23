use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct NewConnect {
    name: String,
}

impl NewConnect {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

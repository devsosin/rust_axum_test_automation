use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct NewImages {
    file_names: Vec<String>,
}

impl NewImages {
    pub fn new(file_names: Vec<String>) -> Self {
        Self { file_names }
    }

    pub fn len(&self) -> usize {
        self.file_names.len()
    }

    pub fn get_file_names(&self) -> &Vec<String> {
        &self.file_names
    }
}

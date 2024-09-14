use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PresignedUrl {
    id: i32,
    url: String,
}

impl PresignedUrl {
    pub fn new(id: i32, url: String) -> Self {
        Self { id, url }
    }
}

use serde::{Deserialize, Serialize};

use super::request::LoginType;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserInfo {
    id: i32,
    login_type: LoginType,
    username: String,
    email: String, // masking
    nickname: String,
    phone: Option<String>, // masking
    profile_id: Option<i32>,
}

impl UserInfo {
    pub fn new(
        id: i32,
        username: String,
        email: String, // masking
        nickname: String,
        login_type: LoginType,
        phone: Option<String>, // masking
        profile_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            nickname,
            login_type,
            phone,
            profile_id,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}

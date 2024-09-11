use serde::Serialize;

use super::request::LoginType;

#[derive(Serialize, Clone)]
pub struct UserInfo {
    id: i32,
    email: String, // masking
    nickname: String,
    login_type: LoginType,
    phone: Option<String>, // masking
    profile_id: Option<i32>,
}

impl UserInfo {
    pub fn new(
        id: i32,
        email: String, // masking
        nickname: String,
        login_type: LoginType,
        phone: Option<String>, // masking
        profile_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            email,
            nickname,
            login_type,
            phone,
            profile_id,
        }
    }
}

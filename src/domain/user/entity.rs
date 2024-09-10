use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Clone)]
pub(super) struct User {
    id: Option<i32>,
    user_email: String,
    password: String,
    nickname: String,
    login_type: String,
    phone: Option<String>,
    unique_id: Option<String>,
    access_token: Option<String>,

    profile_id: Option<i32>,
    is_active: bool,
    is_admin: bool,

    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl User {
    pub(super) fn new(
        user_email: String,
        password: String,
        nickname: String,
        login_type: String,
    ) -> Self {
        Self {
            id: None,
            user_email,
            password,
            nickname,
            login_type,
            phone: None,
            unique_id: None,
            access_token: None,
            profile_id: None,
            is_active: true,
            is_admin: false,
            created_at: None,
            updated_at: None,
        }
    }

    pub(crate) fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }
    pub(crate) fn phone(mut self, phone: Option<String>) -> Self {
        self.phone = phone;
        self
    }
    pub(crate) fn unique_id(mut self, unique_id: Option<String>) -> Self {
        self.unique_id = unique_id;
        self
    }
    pub(crate) fn access_token(mut self, access_token: Option<String>) -> Self {
        self.access_token = access_token;
        self
    }
    pub(crate) fn profile_id(mut self, profile_id: Option<i32>) -> Self {
        self.profile_id = profile_id;
        self
    }
    pub(crate) fn build(self) -> Self {
        Self {
            id: self.id,
            user_email: self.user_email,
            password: self.password,
            nickname: self.nickname,
            login_type: self.login_type,
            phone: self.phone,
            unique_id: self.unique_id,
            access_token: self.access_token,

            profile_id: self.profile_id,
            is_active: self.is_active,
            is_admin: self.is_admin,

            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn get_id(&self) -> &Option<i32> {
        &self.id
    }
    pub fn get_user_email(&self) -> &str {
        &self.user_email
    }
    pub(crate) fn get_password(&self) -> &str {
        &self.password
    }
    pub fn get_nickname(&self) -> &str {
        &self.nickname
    }
    pub fn get_login_type(&self) -> &str {
        &self.login_type
    }
    pub fn get_phone(&self) -> &Option<String> {
        &self.phone
    }
    pub fn get_unique_id(&self) -> &Option<String> {
        &self.unique_id
    }
    pub fn get_access_token(&self) -> &Option<String> {
        &self.access_token
    }
    pub fn get_profile_id(&self) -> &Option<i32> {
        &self.profile_id
    }
}

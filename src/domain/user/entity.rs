use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::global::constants::FieldUpdate;

use super::dto::response::UserInfo;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Clone)]
pub struct User {
    id: Option<i32>,
    login_type: String,
    username: String,
    password: String,
    access_token: Option<String>,

    nickname: String,
    email: String,
    profile_id: Option<i32>,
    phone: Option<String>,

    is_active: bool,
    is_admin: bool,

    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl User {
    pub fn new(
        username: String,
        password: String,
        nickname: String,
        email: String,
        login_type: String,
    ) -> Self {
        Self {
            id: None,
            login_type,
            username,
            password,
            access_token: None,

            nickname,
            email,
            profile_id: None,
            phone: None,

            is_active: true,
            is_admin: false,

            created_at: None,
            updated_at: None,
        }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }
    pub fn phone(mut self, phone: Option<String>) -> Self {
        self.phone = phone;
        self
    }
    pub fn access_token(mut self, access_token: Option<String>) -> Self {
        self.access_token = access_token;
        self
    }
    pub fn profile_id(mut self, profile_id: Option<i32>) -> Self {
        self.profile_id = profile_id;
        self
    }

    pub fn build(self) -> Self {
        Self {
            id: self.id,
            login_type: self.login_type,
            username: self.username,
            password: self.password,
            access_token: self.access_token,

            nickname: self.nickname,
            email: self.email,
            profile_id: self.profile_id,
            phone: self.phone,

            is_active: self.is_active,
            is_admin: self.is_admin,

            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn get_id(&self) -> &Option<i32> {
        &self.id
    }
    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn get_password(&self) -> &str {
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
    pub fn get_email(&self) -> &str {
        &self.email
    }
    pub fn get_access_token(&self) -> &Option<String> {
        &self.access_token
    }
    pub fn get_profile_id(&self) -> &Option<i32> {
        &self.profile_id
    }
    pub fn get_is_active(&self) -> bool {
        self.is_active
    }
    pub fn get_updated_at(&self) -> &Option<NaiveDateTime> {
        &self.updated_at
    }

    pub fn to_info(&self) -> UserInfo {
        UserInfo::new(
            self.id.unwrap(),
            self.username.to_string(),
            self.email.to_string(),
            self.nickname.to_string(),
            self.login_type.parse().unwrap(),
            self.phone.clone(),
            self.profile_id.clone(),
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(super) struct UpdateUser {
    profile_id: FieldUpdate<i32>,
    password: FieldUpdate<String>,
    phone: FieldUpdate<String>,
    nickname: FieldUpdate<String>,
}

impl UpdateUser {
    pub fn new(
        profile_id: FieldUpdate<i32>,
        password: FieldUpdate<String>,
        phone: FieldUpdate<String>,
        nickname: FieldUpdate<String>,
    ) -> Self {
        Self {
            profile_id,
            password,
            phone,
            nickname,
        }
    }

    pub fn get_profile_id(&self) -> &FieldUpdate<i32> {
        &self.profile_id
    }
    pub fn get_password(&self) -> &FieldUpdate<String> {
        &self.password
    }
    pub fn get_phone(&self) -> &FieldUpdate<String> {
        &self.phone
    }
    pub fn get_nickname(&self) -> &FieldUpdate<String> {
        &self.nickname
    }
}

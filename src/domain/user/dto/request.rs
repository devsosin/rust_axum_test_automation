use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    domain::user::entity::{UpdateUser, User},
    global::{constants::FieldUpdate, errors::CustomError},
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoginType {
    Email,
    Google,
    Naver,
    Kakao,
    Meta,
}

impl FromStr for LoginType {
    type Err = Arc<CustomError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(LoginType::Google),
            "meta" => Ok(LoginType::Meta),
            "email" => Ok(LoginType::Email),
            "kakao" => Ok(LoginType::Kakao),
            "naver" => Ok(LoginType::Naver),
            _ => Err(Arc::new(CustomError::ValidationError(
                "LoginType".to_string(),
            ))),
        }
    }
}

impl ToString for LoginType {
    fn to_string(&self) -> String {
        match self {
            LoginType::Email => "email".to_string(),
            LoginType::Google => "google".to_string(),
            LoginType::Naver => "naver".to_string(),
            LoginType::Kakao => "kakao".to_string(),
            LoginType::Meta => "meta".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct NewUser {
    user_email: String,
    password: String,
    password_confirm: String, // handler check
    nickname: String,
    login_type: LoginType,
    phone: Option<String>,
    unique_id: Option<String>,
    access_token: Option<String>,
}

impl NewUser {
    pub(crate) fn new(
        user_email: String,
        password: String,
        password_confirm: String,
        nickname: String,
        login_type: LoginType,
        phone: Option<String>,
        unique_id: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        Self {
            user_email,
            password,
            password_confirm,
            nickname,
            login_type,
            phone,
            unique_id,
            access_token,
        }
    }

    pub(crate) fn set_password(&mut self, password: String) {
        self.password = password
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }

    pub(crate) fn is_password_matching(&self) -> bool {
        &self.password == &self.password_confirm
    }

    pub(crate) fn to_entity(&self) -> User {
        let login_type = self.login_type.to_string();
        User::new(
            self.user_email.to_string(),
            self.password.to_string(),
            self.nickname.to_string(),
            login_type,
        )
        .phone(self.phone.clone())
        .unique_id(self.unique_id.clone())
        .access_token(self.access_token.clone())
        .build()
    }

    pub(crate) fn get_email(&self) -> &str {
        &self.user_email
    }
    pub(crate) fn get_phone(&self) -> &Option<String> {
        &self.phone
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct EditPassword {
    new: String,
    original: String, // password confirm
}

impl EditPassword {
    pub(crate) fn new(new: String, original: String) -> Self {
        Self { new, original }
    }

    pub(crate) fn get_password(&self) -> &str {
        &self.original
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct EditUser {
    profile_id: Option<i32>,
    password: Option<EditPassword>,
    phone: Option<String>,
    nickname: Option<String>,
}

impl EditUser {
    pub(crate) fn new(
        profile_id: Option<i32>,
        password: Option<EditPassword>,
        phone: Option<String>,
        nickname: Option<String>,
    ) -> Self {
        Self {
            profile_id,
            password,
            phone,
            nickname,
        }
    }
    pub(crate) fn get_password(&self) -> &Option<EditPassword> {
        &self.password
    }
    pub(crate) fn get_phone(&self) -> &Option<String> {
        &self.phone
    }

    pub(crate) fn to_update(self) -> UpdateUser {
        let profile_id = match self.profile_id {
            // 프로필 내리기 -> 0이면
            Some(v) if v == 0 => FieldUpdate::SetNone,
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        let password: FieldUpdate<String> = match self.password {
            Some(v) => FieldUpdate::Set(v.new),
            None => FieldUpdate::NoChange,
        };
        let phone = match self.phone {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        let nickname = match self.nickname {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        UpdateUser::new(profile_id, password, phone, nickname)
    }
}

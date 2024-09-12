pub struct AuthConfig {
    jwt_access: String,
    jwt_refresh: String,
}

impl AuthConfig {
    pub fn get_access(&self) -> &str {
        &self.jwt_access
    }
    pub fn get_refresh(&self) -> &str {
        &self.jwt_refresh
    }
}

pub fn get_config() -> AuthConfig {
    let jwt_access = std::env::var("JWT_ACCESS").expect("set JWT_ACCESS env variable");
    let jwt_refresh = std::env::var("JWT_REFRESH").expect("set JWT_REFRESH env variable");

    AuthConfig {
        jwt_access,
        jwt_refresh,
    }
}

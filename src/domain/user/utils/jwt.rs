use serde::{Deserialize, Serialize};

use jsonwebtoken::{
    decode, encode, errors::Result as JWT_Result, DecodingKey, EncodingKey, Header, Validation,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: i32,
    pub iat: usize,
    pub exp: usize,
    pub username: Option<String>,
}

pub fn create_jwt(
    user_id: i32,
    username: Option<String>,
    jwt_secret: &str,
    exp_time: i64,
) -> JWT_Result<String> {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(exp_time)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user_id,
        iat,
        exp,
        username,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

pub fn decode_jwt(token: &str, jwt_secret: &str) -> JWT_Result<TokenClaims> {
    decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|d| d.claims)
}

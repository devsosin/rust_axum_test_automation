use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use regex::Regex;

pub(crate) enum PasswordValidationError {
    TooShort,
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialChar,
    TooSimple,
}

pub(crate) fn validation_password_strength(
    password: &str,
) -> Result<(), Vec<PasswordValidationError>> {
    let mut errors = Vec::with_capacity(5);

    if password.len() < 8 {
        errors.push(PasswordValidationError::TooShort);
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push(PasswordValidationError::MissingUppercase);
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push(PasswordValidationError::MissingLowercase);
    }

    if !password.chars().any(|c| c.is_digit(10)) {
        errors.push(PasswordValidationError::MissingNumber);
    }

    let special_char_regex = Regex::new(r#"[!@#$%^&*(),.?":{}|<>]"#).unwrap();
    if !special_char_regex.is_match(password) {
        errors.push(PasswordValidationError::MissingSpecialChar);
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

pub(crate) fn hash_password(password: &[u8]) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
}

pub(crate) fn verify_password(
    stored_hash: &str,
    password: &[u8],
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(stored_hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password, &parsed_hash).is_ok())
}

use regex::Regex;

pub enum PasswordValidationError {
    TooShort,
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialChar,
    TooSimple,
}

pub fn validation_password_strength(password: &str) -> Result<(), Vec<PasswordValidationError>> {
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

pub fn validation_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    email_regex.is_match(email)
}

pub fn validation_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r#"^[\d\s\-\(\)]+$"#).unwrap();
    let sanitized_input: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    phone_regex.is_match(phone) && sanitized_input.len() >= 10
}

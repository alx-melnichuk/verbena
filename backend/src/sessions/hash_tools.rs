use std::fmt;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub const CD_HASHING_PASSWD: &str = "HashingPassword";

const MAX_PASSWORD_LENGTH: usize = 64;

#[derive(Debug, PartialEq)]
pub enum HashError {
    /// Parameter is empty.
    PasswordIsEmpty,
    /// The maximum parameter length has been exceeded.
    PasswordExceedMaxLen(usize),
    /// An error occurred while creating the hash.
    Hashing(String),
    /// An error occurred while parsing the hash.
    InvalidHashFormat(String),
}

impl Into<String> for HashError {
    fn into(self) -> String {
        self.to_string()
    }
}

impl std::error::Error for HashError {}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            HashError::PasswordIsEmpty => write!(f, "Password is empty."),
            HashError::PasswordExceedMaxLen(max_length) => {
                write!(
                    f,
                    "The password exceeds the max length of {} characters.",
                    max_length
                )
            }
            HashError::Hashing(info) => {
                write!(f, "Error creating hash: {}", info)
            }
            HashError::InvalidHashFormat(info) => {
                write!(f, "Invalid password hash format: {}", info)
            }
        }
    }
}

pub fn hash(password: impl Into<String>) -> Result<String, HashError> {
    let password = password.into();

    if password.is_empty() {
        return Err(HashError::PasswordIsEmpty);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(HashError::PasswordExceedMaxLen(MAX_PASSWORD_LENGTH));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        // .map_err(|_| ErrorMessage::HashingError)?;
        .map_err(|e| HashError::Hashing(e.to_string()))?
        .to_string();

    // Ok(password_hash.hash.to_owned())
    Ok(password_hash)
}

pub fn compare(password: &str, hashed_password: &str) -> Result<bool, HashError> {
    if password.is_empty() {
        return Err(HashError::PasswordIsEmpty);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(HashError::PasswordExceedMaxLen(MAX_PASSWORD_LENGTH));
    }

    let parsed_hash = PasswordHash::new(hashed_password)
        // .map_err(|_| ErrorMessage::InvalidHashFormat)?;
        .map_err(|e| HashError::InvalidHashFormat(e.to_string()))?;

    let password_matches = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true);

    Ok(password_matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test() -> (String, String) {
        let password = "password123";
        let hashed_password = hash(password).unwrap();
        (password.to_string(), hashed_password)
    }

    #[test]
    fn test_compare_hashed_passwords_should_return_true() {
        let (password, hashed_password) = setup_test();

        assert_eq!(compare(&password, &hashed_password).unwrap(), true);
    }

    #[test]
    fn test_compare_hashed_passwords_should_return_false() {
        let (_, hashed_password) = setup_test();

        assert_eq!(compare("wrongpassword", &hashed_password).unwrap(), false);
    }

    #[test]
    fn test_compare_empty_password_should_return_fail() {
        let (_, hashed_password) = setup_test();

        assert_eq!(
            compare("", &hashed_password).unwrap_err(),
            // ErrorMessage::EmptyPassword
            HashError::PasswordIsEmpty
        )
    }

    #[test]
    fn test_compare_long_password_should_return_fail() {
        let (_, hashed_password) = setup_test();

        let long_password = "a".repeat(1000);
        assert_eq!(
            compare(&long_password, &hashed_password).unwrap_err(),
            // # ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH)
            HashError::PasswordExceedMaxLen(MAX_PASSWORD_LENGTH)
        );
    }

    #[test]
    fn test_compare_invalid_hash_should_fail() {
        let invalid_hash = "invalid-hash";

        let text_error = HashError::InvalidHashFormat("".to_string()).to_string();

        let result = compare("password123", invalid_hash).unwrap_err().to_string();
        assert_eq!(
            result[..text_error.len()].to_string(),
            // #ErrorMessage::InvalidHashFormat
            text_error
        )
    }

    #[test]
    fn test_hash_empty_password_should_fail() {
        let result = hash("");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            // # ErrorMessage::EmptyPassword
            HashError::PasswordIsEmpty
        )
    }

    #[test]
    fn test_hash_long_password_should_fail() {
        let result = hash("a".repeat(1000));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            // # ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH)
            HashError::PasswordExceedMaxLen(MAX_PASSWORD_LENGTH)
        );
    }
}

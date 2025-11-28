use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString},
};
use getrandom;

pub const MAX_PARAM_LENGTH: usize = 64;
pub const ERR_PARAM_EMPTY: &str = "Parameter is empty.";
pub const ERR_PARAM_EXCEED_MAX_LEN: &str = "The parameter exceeds the max length of ";
pub const ERR_HASHING_ERR: &str = "Error creating hash - ";
pub const ERR_INVALID_HASH_FORMAT: &str = "Invalid parameter hash format - ";

/// Create a hash for the value.
pub fn encode_hash(param: impl Into<String>) -> Result<String, String> {
    let param = param.into();

    if param.is_empty() {
        return Err(ERR_PARAM_EMPTY.to_string());
    }
    if param.len() > MAX_PARAM_LENGTH {
        return Err(format!("{}{}", ERR_PARAM_EXCEED_MAX_LEN, MAX_PARAM_LENGTH));
    }

    let mut bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    let _ = getrandom::fill(&mut bytes);
    let salt = SaltString::encode_b64(&bytes).unwrap();

    let param_hash = Argon2::default()
        .hash_password(param.as_bytes(), &salt)
        .map_err(|e| format!("{}{}", ERR_HASHING_ERR, e.to_string()))?
        .to_string();

    Ok(param_hash.to_owned())
}

/// Compare the hash for the value with the specified one.
pub fn compare_hash(param: impl Into<String>, hashed_param: &str) -> Result<bool, String> {
    let param = param.into();

    if param.is_empty() {
        return Err(ERR_PARAM_EMPTY.to_string());
    }
    if param.len() > MAX_PARAM_LENGTH {
        return Err(format!("{}{}", ERR_PARAM_EXCEED_MAX_LEN, MAX_PARAM_LENGTH));
    }

    let parsed_hash = PasswordHash::new(hashed_param).map_err(|e| format!("{}{}", ERR_INVALID_HASH_FORMAT, e.to_string()))?;

    let compare_res = Argon2::default().verify_password(param.as_bytes(), &parsed_hash);

    Ok(compare_res.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test() -> (String, String) {
        let password = "password123";
        let hashed_password = encode_hash(password).unwrap();
        (password.to_string(), hashed_password)
    }

    #[test]
    fn test_compare_hashed_passwords_should_return_true() {
        let (password, hashed_password) = setup_test();

        assert_eq!(compare_hash(&password, &hashed_password).unwrap(), true);
    }

    #[test]
    fn test_compare_hashed_passwords_should_return_false() {
        let (_, hashed_password) = setup_test();

        assert_eq!(compare_hash("wrongpassword", &hashed_password).unwrap(), false);
    }

    #[test]
    fn test_compare_empty_password_should_return_fail() {
        let (_, hashed_password) = setup_test();

        let result = compare_hash("", &hashed_password).unwrap_err();
        assert_eq!(result, ERR_PARAM_EMPTY)
    }

    #[test]
    fn test_compare_long_password_should_return_fail() {
        let (_, hashed_password) = setup_test();

        let long_password = "a".repeat(MAX_PARAM_LENGTH + 1);
        let result = compare_hash(&long_password, &hashed_password).unwrap_err();
        let error = format!("{}{}", ERR_PARAM_EXCEED_MAX_LEN, MAX_PARAM_LENGTH);
        assert_eq!(result, error);
    }

    #[test]
    fn test_compare_invalid_hash_should_fail() {
        let invalid_hash = "invalid-hash";

        let result = compare_hash("password123", invalid_hash).unwrap_err().to_string();
        let text_error = ERR_INVALID_HASH_FORMAT.to_string();
        assert_eq!(result[..text_error.len()].to_string(), text_error)
    }

    #[test]
    fn test_hash_empty_password_should_fail() {
        let result = encode_hash("");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ERR_PARAM_EMPTY)
    }

    #[test]
    fn test_hash_long_password_should_fail() {
        let result = encode_hash("a".repeat(MAX_PARAM_LENGTH + 1));
        let error = format!("{}{}", ERR_PARAM_EXCEED_MAX_LEN, MAX_PARAM_LENGTH);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), error);
    }
}

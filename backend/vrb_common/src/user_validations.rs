use crate::validators::{ValidationChecks, ValidationError};

// ** Section: "nickname" **

pub const NICKNAME_MIN: u8 = 3;
pub const NICKNAME_MAX: u8 = 64;
pub const NICKNAME_REGEX: &str = r"^[a-zA-Z]+[\w]+$";
// \w   Matches any letter, digit or underscore. Equivalent to [a-zA-Z0-9_].
// \W - Matches anything other than a letter, digit or underscore. Equivalent to [^a-zA-Z0-9_]
pub const MSG_NICKNAME_REQUIRED: &str = "nickname:required";
pub const MSG_NICKNAME_MIN_LENGTH: &str = "nickname:min_length";
pub const MSG_NICKNAME_MAX_LENGTH: &str = "nickname:max_length";
pub const MSG_NICKNAME_REGEX: &str = "nickname:regex";

// MIN=3, MAX=64, REG="^[a-zA-Z]+[\\w]+$"
pub fn validate_nickname(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NICKNAME_REQUIRED)?;
    ValidationChecks::min_length(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX_LENGTH)?;
    ValidationChecks::regexp(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
    Ok(())
}

// ** Section: "email" **

pub const EMAIL_MIN: u8 = 5;
// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address?
// Answer: An email address must not exceed 254 characters.
pub const EMAIL_MAX: u16 = 254;
pub const MSG_EMAIL_REQUIRED: &str = "email:required";
pub const MSG_EMAIL_MIN_LENGTH: &str = "email:min_length";
pub const MSG_EMAIL_MAX_LENGTH: &str = "email:max_length";
pub const MSG_EMAIL_EMAIL_TYPE: &str = "email:email_type";

// MIN=5, MAX=254, "email:email_type"
pub fn validate_email(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_EMAIL_REQUIRED)?;
    ValidationChecks::min_length(value, EMAIL_MIN.into(), MSG_EMAIL_MIN_LENGTH)?;
    ValidationChecks::max_length(value, EMAIL_MAX.into(), MSG_EMAIL_MAX_LENGTH)?;
    ValidationChecks::email(value, MSG_EMAIL_EMAIL_TYPE)?;
    Ok(())
}

// ** Section: "password" **

pub const PASSWORD_MIN: u8 = 6;
pub const PASSWORD_MAX: u8 = 64;
pub const PASSWORD_LOWERCASE_LETTER_REGEX: &str = r"[a-z]+";
pub const PASSWORD_CAPITAL_LETTER_REGEX: &str = r"[A-Z]+";
pub const PASSWORD_NUMBER_REGEX: &str = r"[\d]+";
// pub const PASSWORD_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,}$";
pub const MSG_PASSWORD_REQUIRED: &str = "password:required";
pub const MSG_PASSWORD_MIN_LENGTH: &str = "password:min_length";
pub const MSG_PASSWORD_MAX_LENGTH: &str = "password:max_length";
pub const MSG_PASSWORD_REGEX: &str = "password:regex";

// MIN=6, MAX=64, REG="[a-z]+","[A-Z]+","[\\d]+"
pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_PASSWORD_REGEX)?;
    Ok(())
}

// ** Section: "new_password" **

pub const MSG_NEW_PASSWORD_REQUIRED: &str = "new_password:required";
pub const MSG_NEW_PASSWORD_MIN_LENGTH: &str = "new_password:min_length";
pub const MSG_NEW_PASSWORD_MAX_LENGTH: &str = "new_password:max_length";
pub const MSG_NEW_PASSWORD_REGEX: &str = "new_password:regex";
pub const MSG_NEW_PASSWORD_EQUAL_OLD_VALUE: &str = "new_password:equal_to_old_value";

// MIN=6, MAX=64, REG="[a-z]+","[A-Z]+","[\\d]+" OR "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d\\W_]{6,}$"
pub fn validate_new_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NEW_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_NEW_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_NEW_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    Ok(())
}

pub fn validate_inequality(value1: &str, value2: &str) -> Result<(), ValidationError> {
    if value1.starts_with(value2) && value1.len() == value2.len() {
        let err = ValidationError::new(MSG_NEW_PASSWORD_EQUAL_OLD_VALUE);
        return Err(err);
    }
    Ok(())
}

// ** Section: "descript" **

pub const DESCRIPT_MIN: u8 = 2;
pub const DESCRIPT_MAX: u16 = 2048; // 2*1024

// ** Section: "theme" **

pub const THEME_MIN: u8 = 2;
pub const THEME_MAX: u8 = 32;

// ** Section: "locale" **

pub const LOCALE_MIN: u8 = 2;
pub const LOCALE_MAX: u8 = 32;

// ** - **
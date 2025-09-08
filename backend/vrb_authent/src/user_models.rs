use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use vrb_common::validators::{ValidationChecks, ValidationError};
use vrb_dbase::{enm_user_role::UserRole, schema};

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

// * * * * Section: models for "UserOrm". * * * *

// ** Model: "User". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String, // max_len: 255
    pub email: String,    // max_len: 255
    pub password: String, // max_len: 255
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i32, nickname: &str, email: &str, password: &str, role: UserRole) -> Self {
        let now = Utc::now();
        User {
            id,
            nickname: nickname.into(), // max_len: 255
            email: email.into(),       // max_len: 255
            password: password.into(), // max_len: 255
            role,
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

// ** Used: UserOrm::create_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUser {
    pub nickname: String,       // min_len=3 max_len=64
    pub email: String,          // min_len=5 max_len=254
    pub password: String,       // min_len=6 max_len=64
    pub role: Option<UserRole>, // default "user"
}

impl CreateUser {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> Self {
        CreateUser {
            nickname: nickname.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
            role: role.clone(),
        }
    }
}

// ** Used: UserOrm::modify_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUser {
    pub nickname: Option<String>, // min_len=3,max_len=64
    pub email: Option<String>,    // min_len=5,max_len=254,"email:email_type"
    pub password: Option<String>, // min_len=6,max_len=64
    pub role: Option<UserRole>,   // default "user"
}

impl ModifyUser {
    pub fn new(nickname: Option<String>, email: Option<String>, password: Option<String>, role: Option<UserRole>) -> Self {
        ModifyUser {
            nickname,
            email,
            password,
            role,
        }
    }
}

// ** Model: "Session". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub user_id: i32,
    pub num_token: Option<i32>,
}

impl Session {
    pub fn new(user_id: i32, num_token: Option<i32>) -> Self {
        Session { user_id, num_token }
    }
}

// ** Model: "Profile". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
    pub user_id: i32,
    pub avatar: Option<String>,
    pub descript: Option<String>,
    pub theme: Option<String>,
    pub locale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Profile {
    pub fn new(user_id: i32, avatar: Option<String>, descript: Option<String>, theme: Option<String>, locale: Option<String>) -> Self {
        let now = Utc::now();
        Profile {
            user_id,
            avatar,
            descript,
            theme,
            locale,
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

// * * * *  UserMock  * * * *

#[cfg(any(test, feature = "mockdata"))]
pub struct UserMock {}

#[cfg(any(test, feature = "mockdata"))]
impl UserMock {
    pub fn nickname_min() -> String {
        (0..(NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn nickname_max() -> String {
        (0..(NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn nickname_wrong() -> String {
        let nickname: String = (0..(NICKNAME_MIN - 1)).map(|_| 'a').collect();
        format!("{}#", nickname)
    }
    pub fn email_min() -> String {
        let suffix = "@us".to_owned();
        let email_min: usize = EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn email_max() -> String {
        let email_max: usize = EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        format!("{}@{}{}", prefix, suffix, domain)
    }
    pub fn email_wrong() -> String {
        let suffix = "@".to_owned();
        let email_min: usize = EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn password_min() -> String {
        (0..(PASSWORD_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn password_max() -> String {
        (0..(PASSWORD_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn password_wrong() -> String {
        (0..(PASSWORD_MIN)).map(|_| 'a').collect()
    }
    pub fn role_wrong() -> String {
        let role = UserRole::all_values().get(0).unwrap().to_string();
        role[0..(role.len() - 1)].to_string()
    }
}

// * * * *  Profile1Mock  * * * *

pub struct Profile1Mock {}

impl Profile1Mock {
    pub fn get_avatar(_user_id: i32) -> Option<String> {
        None
    }
    pub fn get_descript(user_id: i32) -> Option<String> {
        Some(format!("descript_{}", user_id))
    }
    pub fn get_theme(user_id: i32) -> Option<String> {
        if user_id % 2 == 0 {
            Some("dark".to_owned())
        } else {
            Some("light".to_owned())
        }
    }
    pub fn get_locale(user_id: i32) -> Option<String> {
        if user_id % 2 == 0 {
            Some("default".to_owned())
        } else {
            Some("en-US".to_owned())
        }
    }
    pub fn profile(user_id: i32) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            avatar: Self::get_avatar(user_id),
            descript: Self::get_descript(user_id),
            theme: Self::get_theme(user_id),
            locale: Self::get_locale(user_id),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

// * * * *    * * * *

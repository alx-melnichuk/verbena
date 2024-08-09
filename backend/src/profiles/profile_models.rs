use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::users::user_models::{User, UserRole};
use crate::utils::serial_datetime;
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const NICKNAME_MIN: u8 = 3;
pub const NICKNAME_MAX: u8 = 64;
pub const NICKNAME_REGEX: &str = r"^[a-zA-Z]+[\w]+$";
// \w   Matches any letter, digit or underscore. Equivalent to [a-zA-Z0-9_].
// \W - Matches anything other than a letter, digit or underscore. Equivalent to [^a-zA-Z0-9_]
pub const MSG_NICKNAME_REQUIRED: &str = "nickname:required";
pub const MSG_NICKNAME_MIN_LENGTH: &str = "nickname:min_length";
pub const MSG_NICKNAME_MAX_LENGTH: &str = "nickname:max_length";
pub const MSG_NICKNAME_REGEX: &str = "nickname:regex";

pub const EMAIL_MIN: u8 = 5;
// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address?
// Answer: An email address must not exceed 254 characters.
pub const EMAIL_MAX: u16 = 254;
pub const MSG_EMAIL_REQUIRED: &str = "email:required";
pub const MSG_EMAIL_MIN_LENGTH: &str = "email:min_length";
pub const MSG_EMAIL_MAX_LENGTH: &str = "email:max_length";
pub const MSG_EMAIL_EMAIL_TYPE: &str = "email:email_type";

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
pub const MSG_NEW_PASSWORD_REQUIRED: &str = "new_password:required";
pub const MSG_NEW_PASSWORD_MIN_LENGTH: &str = "new_password:min_length";
pub const MSG_NEW_PASSWORD_MAX_LENGTH: &str = "new_password:max_length";
pub const MSG_NEW_PASSWORD_REGEX: &str = "new_password:regex";
pub const MSG_NEW_PASSWORD_EQUAL_OLD_VALUE: &str = "new_password:equal_to_old_value";
// MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
pub fn validate_nickname(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NICKNAME_REQUIRED)?;
    ValidationChecks::min_length(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX_LENGTH)?;
    ValidationChecks::regexp(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
    Ok(())
}
// MIN=5,MAX=255,"email:email_type"
pub fn validate_email(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_EMAIL_REQUIRED)?;
    ValidationChecks::min_length(value, EMAIL_MIN.into(), MSG_EMAIL_MIN_LENGTH)?;
    ValidationChecks::max_length(value, EMAIL_MAX.into(), MSG_EMAIL_MAX_LENGTH)?;
    ValidationChecks::email(value, MSG_EMAIL_EMAIL_TYPE)?;
    Ok(())
}

pub fn validate_nickname_or_email(value: &str) -> Result<(), ValidationError> {
    if value.contains("@") {
        validate_email(&value).map_err(|err| err)?;
    } else {
        validate_nickname(&value).map_err(|err| err)?;
    }
    Ok(())
}
// MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+"
pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_PASSWORD_REGEX)?;
    Ok(())
}
// MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+"
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

// ** Section: database "profiles" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProfileTbl {
    pub user_id: i32,
    // Link to user avatar, optional
    pub avatar: Option<String>, // min_len=2 max_len=255 Nullable
    // User description.
    pub descript: String, // type: Text default ""
    // Default color theme. ["light","dark"]
    pub theme: String, // min_len=2 max_len=32 default "light"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub const PROFILE_DESCRIPT_DEF: &str = "";
pub const PROFILE_THEME_LIGHT_DEF: &str = "light";
pub const PROFILE_THEME_DARK: &str = "dark";

impl ProfileTbl {
    pub fn new(user_id: i32, avatar: Option<&str>, descript: Option<&str>, theme: Option<&str>) -> ProfileTbl {
        let now = Utc::now();
        ProfileTbl {
            user_id,
            avatar: avatar.map(|v| v.to_string()),
            descript: descript.unwrap_or(PROFILE_DESCRIPT_DEF).to_string(),
            theme: theme.unwrap_or(PROFILE_THEME_LIGHT_DEF).to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "user_id")]
    pub user_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "nickname")]
    pub nickname: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "email")]
    pub email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "password")]
    pub password: String,
    #[diesel(sql_type = crate::schema::sql_types::UserRole)]
    #[diesel(column_name = "role")]
    pub role: UserRole,
    pub avatar: Option<String>,
    pub descript: String,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Profile {
    pub fn new(
        user_id: i32,
        nickname: &str,
        email: &str,
        role: UserRole,
        avatar: Option<&str>,
        descript: Option<&str>,
        theme: Option<&str>,
    ) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            nickname: nickname.to_string(),
            email: email.to_string(),
            password: "".to_string(),
            role,
            avatar: avatar.map(|v| v.to_string()),
            descript: descript.unwrap_or(PROFILE_DESCRIPT_DEF).to_string(),
            theme: theme.unwrap_or(PROFILE_THEME_LIGHT_DEF).to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
    pub fn to_user(&self) -> User {
        User {
            id: self.user_id,
            nickname: self.nickname.to_string(),
            email: self.email.to_string(),
            password: "".to_string(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            role: self.role.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub role: UserRole,
    // Link to user avatar, optional
    pub avatar: Option<String>, // min_len=2 max_len=255 Nullable
    // User description.
    pub descript: String, // type: Text default ""
    // Default color theme. ["light","dark"]
    pub theme: String, // min_len=2 max_len=32 default "light"
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl ProfileDto {
    pub fn from(profile_user: Profile) -> ProfileDto {
        ProfileDto {
            id: profile_user.user_id,
            nickname: profile_user.nickname,
            email: profile_user.email,
            role: profile_user.role.clone(),
            avatar: profile_user.avatar.clone(),
            descript: profile_user.descript.clone(),
            theme: profile_user.theme.clone(),
            created_at: profile_user.created_at.clone(),
            updated_at: profile_user.updated_at.clone(),
        }
    }
}

// ** CreateProfile **

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProfile {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
    pub avatar: Option<String>,   // min_len=2 max_len=255 Nullable
    pub descript: Option<String>, // type: Text default ""
    pub theme: Option<String>,    // min_len=2 max_len=32 default "light"
}

impl CreateProfile {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> CreateProfile {
        CreateProfile {
            nickname: nickname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            role: role.clone(),
            avatar: None,
            descript: None,
            theme: None,
        }
    }
}

// ** UniquenessProfileDto **

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UniquenessProfileDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ** Section: "LoginProfile" **

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginProfileDto {
    pub nickname: String,
    pub password: String,
}

impl Validator for LoginProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname_or_email(&self.nickname).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginProfileResponseDto {
    pub profile_dto: ProfileDto,
    pub profile_tokens_dto: ProfileTokensDto,
}

// ** Section: "ProfileToken" **

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTokensDto {
    pub access_token: String,
    pub refresh_token: String,
}

// **  **

#[cfg(all(test, feature = "mockdata"))]
pub struct ProfileTest {}

#[cfg(all(test, feature = "mockdata"))]
impl ProfileTest {
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
        let suffix = "@us".to_string();
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
        let suffix = "@".to_string();
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
}

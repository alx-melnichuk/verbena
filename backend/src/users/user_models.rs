use chrono::{DateTime, Utc};
use diesel::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;
use validator::Validate;

use crate::errors::CD_VALIDATION;
use crate::schema;
use crate::utils::date_time_rfc2822z;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: UserRole,
}

// ** Section: DTO models. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct UserDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    #[serde(rename = "createdAt", with = "date_time_rfc2822z")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", with = "date_time_rfc2822z")]
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            id: user.id,
            nickname: user.nickname.to_owned(),
            email: user.email.to_owned(),
            password: "".to_string(),
            role: user.role.to_owned(),
            created_at: user.created_at.to_owned(),
            updated_at: user.updated_at.to_owned(),
        }
    }
}

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
pub const EMAIL_MAX: u16 = 255;
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

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUserDto {
    #[validate(custom = "UserValidate::nickname")]
    pub nickname: Option<String>,
    #[validate(custom = "UserValidate::email")]
    pub email: Option<String>,
    #[validate(custom = "UserValidate::password")]
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDto {
    #[validate(custom = "UserValidate::nickname")]
    pub nickname: String,
    #[validate(custom = "UserValidate::email")]
    pub email: String,
    #[validate(custom = "UserValidate::password")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct LoginUserDto {
    #[validate(custom = "UserValidate::nickname_or_email")]
    pub nickname: String,
    #[validate(custom = "UserValidate::password")]
    pub password: String,
}

// ** Registration User **

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct RegistrUserDto {
    #[validate(custom = "UserValidate::nickname")]
    pub nickname: String,
    #[validate(custom = "UserValidate::email")]
    pub email: String,
    #[validate(custom = "UserValidate::password")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistrUserResponseDto {
    pub nickname: String,
    pub email: String,
    #[serde(rename = "registrToken")]
    pub registr_token: String,
}

// ** RecoveryUserDto **

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct RecoveryUserDto {
    #[validate(custom = "UserValidate::email")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecoveryUserResponseDto {
    pub id: i32,
    pub email: String,
    #[serde(rename = "recoveryToken")]
    pub recovery_token: String,
}

// ** RecoveryDataDto **

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct RecoveryDataDto {
    #[validate(custom = "UserValidate::password")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecoveryDataResponseDto {
    pub nickname: String,
    pub email: String,
    #[serde(rename = "registrToken")]
    pub registr_token: String,
}

// ** UserRecovery **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::user_recovery)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRecovery {
    pub id: i32,
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::user_recovery)]
pub struct UserRecoveryDto {
    pub id: i32,
    pub user_id: i32,
    #[serde(rename = "finalDate", with = "date_time_rfc2822z")]
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_recovery)]
pub struct CreateUserRecoveryDto {
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

// ** UserValidate **

pub struct UserValidate {}

impl UserValidate {
    #[rustfmt::skip]
    fn check_required(value: &str, msg: &'static str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        if len == 0 {
            let mut err = validator::ValidationError::new(CD_VALIDATION);
            err.message = Some(Cow::Borrowed(msg));
            let data = true;
            err.add_param(Cow::Borrowed("required"), &data);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_min_length(value: &str, min: usize, msg: &'static str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        if len < min {
            let mut err = validator::ValidationError::new(CD_VALIDATION);
            err.message = Some(Cow::Borrowed(msg));
            let json = serde_json::json!({ "actualLength": len, "requiredLength": min });
            err.add_param(Cow::Borrowed("minlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_max_length(value: &str, max: usize, msg: &'static str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        if max < len {
            let mut err = validator::ValidationError::new(CD_VALIDATION);
            err.message = Some(Cow::Borrowed(msg));
            let json = serde_json::json!({ "actualLength": len, "requiredLength": max });
            err.add_param(Cow::Borrowed("maxlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_regexp(value: &str, reg_exp: &str, msg: &'static str) -> Result<(), validator::ValidationError> {
        let regex = Regex::new(reg_exp).unwrap();
        let result = regex.captures(value.clone());
        if result.is_none() {
            let mut err = validator::ValidationError::new(CD_VALIDATION);
            err.message = Some(Cow::Borrowed(msg));
            let json = serde_json::json!({ "actualValue": &value.clone(), "requiredPattern": &reg_exp.clone() });
            err.add_param(Cow::Borrowed("pattern"), &json);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_email(value: &str, msg: &'static str) -> Result<(), validator::ValidationError> {
        if !validator::validate_email(value) {
            let mut err = validator::ValidationError::new(CD_VALIDATION);
            err.message = Some(Cow::Borrowed(msg));
            let data = true;
            err.add_param(Cow::Borrowed("email"), &data);
            return Err(err);
        }
        Ok(())
    }

    pub fn nickname(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_required(value, MSG_NICKNAME_REQUIRED)?;
        Self::check_min_length(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN_LENGTH)?;
        Self::check_max_length(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX_LENGTH)?;
        Self::check_regexp(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
        Ok(())
    }
    pub fn email(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_required(value, MSG_EMAIL_REQUIRED)?;
        Self::check_min_length(value, EMAIL_MIN.into(), MSG_EMAIL_MIN_LENGTH)?;
        Self::check_max_length(value, EMAIL_MAX.into(), MSG_EMAIL_MAX_LENGTH)?;
        Self::check_email(value, MSG_EMAIL_EMAIL_TYPE)?;
        Ok(())
    }

    pub fn password(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_required(value, MSG_PASSWORD_REQUIRED)?;
        Self::check_min_length(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN_LENGTH)?;
        Self::check_max_length(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX_LENGTH)?;
        Self::check_regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
        Self::check_regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
        Self::check_regexp(value, PASSWORD_NUMBER_REGEX, MSG_PASSWORD_REGEX)?;
        Ok(())
    }
    pub fn nickname_or_email(value: &str) -> Result<(), validator::ValidationError> {
        if value.contains("@") {
            Self::email(&value.clone()).map_err(|err| err)?;
        } else {
            Self::nickname(&value.clone()).map_err(|err| err)?;
        }
        Ok(())
    }
}

#[cfg(feature = "mockdata")]
pub struct UserValidateTest {}
#[cfg(feature = "mockdata")]
impl UserValidateTest {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserTokensDto {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginUserResponseDto {
    #[serde(rename = "userDto")]
    pub user_dto: UserDto,
    #[serde(rename = "userTokensDto")]
    pub user_tokens_dto: UserTokensDto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenUserDto {
    pub token: String, // refreshToken
}

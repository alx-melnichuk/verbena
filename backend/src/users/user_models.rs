use chrono::{DateTime, Utc};
use diesel::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use validator::Validate;

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
pub const MSG_NICKNAME_MIN: &str = "must be more than 3 characters";
pub const NICKNAME_MAX: u8 = 64;
pub const MSG_NICKNAME_MAX: &str = "must be less than 64 characters";
pub const NICKNAME_REGEX: &str = r"^[a-zA-Z]+[\w]+$";
pub const MSG_NICKNAME_REGEX: &str = "must match /^[a-zA-Z]+[\\w]+$/";
// \w   Matches any letter, digit or underscore. Equivalent to [a-zA-Z0-9_].
// \W - Matches anything other than a letter, digit or underscore. Equivalent to [^a-zA-Z0-9_]

pub const EMAIL_MIN: u8 = 5;
pub const MSG_EMAIL_MIN: &str = "must be more than 5 characters";
pub const EMAIL_MAX: u16 = 255;
pub const MSG_EMAIL_MAX: &str = "must be less than 255 characters";
pub const MSG_EMAIL: &str = "must match `user@email.com`";

pub const PASSWORD_MIN: u8 = 6;
pub const MSG_PASSWORD_MIN: &str = "must be more than 6 characters";
pub const PASSWORD_MAX: u8 = 64;
pub const MSG_PASSWORD_MAX: &str = "must be less than 64 characters";

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

pub struct UserValidate {}

impl UserValidate {
    #[rustfmt::skip]
    fn new_err<T: Serialize>(code: &'static str, msg: &'static str, val: &T) -> validator::ValidationError {
        let mut err = validator::ValidationError::new(code.clone());
        err.message = Some(Cow::Borrowed(msg));
        err.add_param(Cow::Borrowed("value"), &val);
        err
    }
    #[rustfmt::skip]
    fn check_min(value: &str, min: usize, msg: &'static str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        if len < min {
            let mut err = Self::new_err("length", msg, &value.clone());
            err.add_param(Cow::Borrowed("min"), &min);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_max(value: &str, max: usize, msg: &'static str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        if max < len {
            let mut err = Self::new_err("length", msg, &value.clone());
            err.add_param(Cow::Borrowed("max"), &max);
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_reg_ex(value: &str, reg_ex: &str, msg: &'static str) -> Result<(), validator::ValidationError> {
        let reg_ex = Regex::new(reg_ex).unwrap(); // /^[\w\W_]+$/
        let result = reg_ex.captures(value.clone());
        if result.is_none() {
            let mut err = validator::ValidationError::new("regex");
            err.message = Some(Cow::Borrowed(msg));
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn check_email(value: &str, msg: &'static str) -> Result<(), validator::ValidationError> {
        if !validator::validate_email(value) {
            let mut err = validator::ValidationError::new("email");
            err.message = Some(Cow::Borrowed(msg));
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }
        Ok(())
    }

    pub fn nickname(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_min(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN)?;
        Self::check_max(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX)?;
        Self::check_reg_ex(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
        Ok(())
    }
    pub fn password(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_min(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN)?;
        Self::check_max(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX)?;
        Ok(())
    }
    pub fn email(value: &str) -> Result<(), validator::ValidationError> {
        Self::check_min(value, EMAIL_MIN.into(), MSG_EMAIL_MIN)?;
        Self::check_max(value, EMAIL_MAX.into(), MSG_EMAIL_MAX)?;
        Self::check_email(value, MSG_EMAIL)?;
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

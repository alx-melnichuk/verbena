use chrono::{DateTime, Utc};
use diesel::prelude::*;
use lazy_static::lazy_static;
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

impl UserDto {
    pub fn verify_for_create(&self) -> bool {
        self.nickname.len() > 0 && self.email.len() > 0 && self.password.len() > 0
    }
    pub fn verify_for_edit(&self) -> bool {
        self.nickname.len() == 0 || self.email.len() == 0 || self.password.len() == 0
    }
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

pub const EMAIL_MIN: u8 = 5;
pub const EMAIL_MAX: u16 = 255;

pub const PASSWORD_MIN: u8 = 6;
pub const PASSWORD_MAX: u8 = 64;

lazy_static! {
    static ref NICKNAME_REG: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    // static ref PASSWORD_REG: Regex = Regex::new(r"^[a-z]*[A-Z]*\d*[A-Za-z\d\W_]{6,}$").unwrap();
    static ref PASSWORD_REG: Regex = Regex::new(r"^[a-zA-Z]+\d*[A-Za-z\d\W_]{6,}$").unwrap();
    // \W - Matches anything other than a letter, digit or underscore. Equivalent to [^a-zA-Z0-9_]
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUserDto {
    #[validate(
        length(min = "NICKNAME_MIN", message = "must be more than 3 characters"),
        length(max = "NICKNAME_MAX", message = "must be less than 64 characters"),
        regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
    )]
    pub nickname: Option<String>,
    #[validate(
        length(min = "EMAIL_MIN", message = "must be more than 5 characters"),
        length(max = "EMAIL_MAX", message = "must be less than 255 characters"),
        email(message = "wrong email")
    )]
    pub email: Option<String>,
    #[validate(
        length(min = "PASSWORD_MIN", message = "must be more than 6 characters"),
        length(max = "PASSWORD_MAX", message = "must be less than 64 characters"),
        regex(
            path = "PASSWORD_REG",
            message = "wrong value /^[a-zA-Z]+\\d*[A-Za-z\\d\\W_]{6,}$/"
        )
    )]
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDto {
    #[validate(
        length(min = "NICKNAME_MIN", message = "must be more than 3 characters"),
        length(max = "NICKNAME_MAX", message = "must be less than 64 characters"),
        regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
    )]
    pub nickname: String,
    #[validate(
        length(min = "EMAIL_MIN", message = "must be more than 5 characters"),
        length(max = "EMAIL_MAX", message = "must be less than 255 characters"),
        email(message = "wrong email")
    )]
    pub email: String,
    #[validate(
        length(min = "PASSWORD_MIN", message = "must be more than 6 characters"),
        length(max = "PASSWORD_MAX", message = "must be less than 64 characters"),
        regex(
            path = "PASSWORD_REG",
            message = "wrong value /^[a-zA-Z]+\\d*[A-Za-z\\d\\W_]{6,}$/"
        )
    )]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct LoginUserDto {
    #[validate(
        length(min = "NICKNAME_MIN", message = "must be more than 3 characters"),
        length(max = "NICKNAME_MAX", message = "must be less than 64 characters"),
        regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
    )]
    // #[validate(custom = "LoginUserDto::validate_nickname_or_email")]
    // #[validate(
    //     custom = "LoginUserDto::validate_nickname",
    //     custom = "LoginUserDto::validate_email"
    // )]
    pub nickname: String,
    #[validate(
        length(min = "PASSWORD_MIN", message = "must be more than 6 characters"),
        length(max = "PASSWORD_MAX", message = "must be less than 64 characters"),
        regex(
            path = "PASSWORD_REG",
            message = "wrong value /^[a-zA-Z]+\\d*[A-Za-z\\d\\W_]{6,}$/"
        )
    )]
    pub password: String,
}

impl LoginUserDto {
    pub fn validate_nickname(value: &str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        // nuickname
        // #- length(min = "NICKNAME_MIN", message = "must be more than 3 characters"),
        let min: usize = NICKNAME_MIN.clone().into();
        if len < min {
            let mut err = validator::ValidationError::new("length");
            err.message = Some(Cow::Borrowed("must be more than 3 characters"));
            err.add_param(Cow::Borrowed("min"), &min);
            err.add_param(Cow::Borrowed("value"), &value.clone());
            // code: "length", message: Some("must be more than 6 characters"),params: {"min": Number(6), "value": String("aaaaa")}
            return Err(err);
        }
        // #- length(max = "NICKNAME_MAX", message = "must be less than 64 characters"),
        let max: usize = NICKNAME_MAX.clone().into();
        if max < len {
            let mut err = validator::ValidationError::new("length");
            err.message = Some(Cow::Borrowed("must be less than 64 characters"));
            err.add_param(Cow::Borrowed("max"), &max);
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }
        // regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
        // NICKNAME_REG
        let reg_ex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        let res = reg_ex.captures(value.clone());
        if res.is_none() {
            let mut err = validator::ValidationError::new("regex");
            err.message = Some(Cow::Borrowed("wrong value /^[a-zA-Z0-9_]+$/"));
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }

        Ok(())
    }
    pub fn validate_email(value: &str) -> Result<(), validator::ValidationError> {
        let len: usize = value.len();
        let min: usize = EMAIL_MIN.clone().into();
        // code: "length", message: Some("must be more than 3 characters"),params: {"min": Number(3), "value": String("aaaaa")}
        if len < min {
            let mut err = validator::ValidationError::new("length");
            err.message = Some(Cow::Borrowed("must be more than 5 characters"));
            err.add_param(Cow::Borrowed("min"), &min);
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }
        let max: usize = EMAIL_MAX.clone().into();
        // code: "length", message: Some("must be less than 64 characters"), params: { "max": Number(64), "value": String("aaaaa")} }
        if max < len {
            let mut err = validator::ValidationError::new("length");
            err.message = Some(Cow::Borrowed("must be less than 255 characters"));
            err.add_param(Cow::Borrowed("max"), &max);
            err.add_param(Cow::Borrowed("value"), &value.clone());
            return Err(err);
        }
        // regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
        Ok(())
    }
    pub fn validate_nickname_or_email(value: &str) -> Result<(), validator::ValidationError> {
        if value.contains("@") {
            // email
            // length(min = "EMAIL_MIN", message = "must be more than 5 characters"),
            // length(max = "EMAIL_MAX", message = "must be less than 255 characters"),
            // email(message = "wrong email")
            Self::validate_email(&value.clone()).map_err(|err| err)?;
        } else {
            // nickname
            // length(min = "NICKNAME_MIN", message = "must be more than 3 characters"),
            // length(max = "NICKNAME_MAX", message = "must be less than 64 characters"),
            // regex(path = "NICKNAME_REG", message = "wrong value /^[a-zA-Z0-9_]+$/")
            Self::validate_nickname(&value.clone()).map_err(|err| err)?;
        }
        if value == "xXxShad0wxXx" {
            // the value of the username will automatically be added later
            return Err(validator::ValidationError::new("terrible_username"));
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
pub struct ResponseLoginUserDto {
    #[serde(rename = "userDto")]
    pub user_dto: UserDto,
    #[serde(rename = "userTokensDto")]
    pub user_tokens_dto: UserTokensDto,
}

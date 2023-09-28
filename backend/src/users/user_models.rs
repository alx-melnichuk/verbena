use chrono::{DateTime, Utc};
use diesel::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Validate, Serialize, Deserialize, Clone, AsChangeset)]
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

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

// #[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct LoginUserDto {
    #[validate(
        length(min = 3, message = "must be more than 3 characters"),
        length(max = 64, message = "must be less than 64 characters")
    )]
    pub nickname: String,
    #[validate(
        length(min = 6, message = "must be more than 6 characters"),
        length(max = 64, message = "must be less than 64 characters")
    )]
    pub password: String,
}

/*#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokensDTO {
    pub accessToken: String,
    pub refreshToken: String,
}*/

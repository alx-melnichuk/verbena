use chrono::{DateTime, Utc};
use diesel::prelude::*;
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

#[derive(Debug, Validate, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUserDto {
    #[validate(
        length(min = 3, message = "must be more than 3 characters"),
        length(max = 64, message = "must be less than 64 characters")
    )]
    pub nickname: Option<String>,
    #[validate(
        length(min = 5, message = "must be more than 5 characters"),
        length(max = 255, message = "must be less than 255 characters")
    )]
    pub email: Option<String>,
    #[validate(
        length(min = 6, message = "must be more than 6 characters"),
        length(max = 64, message = "must be less than 64 characters")
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUserDto {
    pub nickname: String,
    pub password: String,
}

/*#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokensDTO {
    pub accessToken: String,
    pub refreshToken: String,
}*/

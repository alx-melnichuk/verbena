use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema;
use crate::utils::option_date_time;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct UserDTO {
    pub id: Option<i32>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    #[serde(
        default = "UserDTO::option_none",
        rename = "createdAt",
        skip_serializing_if = "Option::is_none",
        with = "option_date_time"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        default = "UserDTO::option_none",
        rename = "updatedAt",
        skip_serializing_if = "Option::is_none",
        with = "option_date_time"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDTO {
    fn from(user: User) -> Self {
        UserDTO {
            id: Some(user.id),
            nickname: Some(user.nickname),
            email: Some(user.email),
            password: Some(user.password),
            created_at: Some(user.created_at),
            updated_at: Some(user.updated_at),
        }
    }
}

impl UserDTO {
    pub fn is_empty(user_dto: &UserDTO) -> bool {
        user_dto.id.is_none()
            && user_dto.nickname.is_none()
            && user_dto.email.is_none()
            && user_dto.password.is_none()
    }
    pub fn new() -> Self {
        UserDTO {
            id: None,
            nickname: None,
            email: None,
            password: None,
            created_at: None,
            updated_at: None,
        }
    }
    pub fn option_none() -> Option<DateTime<Utc>> {
        Option::None
    }
    pub fn clear_optional(user_dto: &mut UserDTO) {
        user_dto.id = None;
        user_dto.created_at = None;
        user_dto.updated_at = None;
    }
}

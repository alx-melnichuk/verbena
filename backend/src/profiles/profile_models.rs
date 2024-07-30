use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::utils::serial_datetime;

// ** Section: database "profiles" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
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

impl Profile {
    pub fn new(user_id: i32, avatar: Option<&str>, descript: &str, theme: &str) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            avatar: avatar.map(|v| v.to_string()),
            descript: descript.to_string(),
            theme: theme.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct ProfileUser {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "user_id")]
    pub user_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "nickname")]
    pub nickname: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "email")]
    pub email: String,
    pub avatar: Option<String>,
    pub descript: String,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProfileUser {
    pub fn new(
        user_id: i32,
        nickname: &str,
        email: &str,
        avatar: Option<&str>,
        descript: &str,
        theme: &str,
    ) -> ProfileUser {
        let now = Utc::now();
        ProfileUser {
            user_id,
            nickname: nickname.to_string(),
            email: email.to_string(),
            avatar: avatar.map(|v| v.to_string()),
            descript: descript.to_string(),
            theme: theme.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUserDto {
    pub id: i32,
    #[schema(example = "Emma_Johnson2")]
    pub nickname: String,
    #[schema(example = "Emma_Johnson2@gmail.us")]
    pub email: String,
    // Link to user avatar, optional
    pub avatar: Option<String>, // min_len=2 max_len=255 Nullable
    // User description.
    #[schema(example = "Description Emma_Johnson2")]
    pub descript: String, // type: Text default ""
    // Default color theme. ["light","dark"]
    #[schema(example = "light")]
    pub theme: String, // min_len=2 max_len=32 default "light"
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl ProfileUserDto {
    pub fn from(profile_user: ProfileUser) -> ProfileUserDto {
        ProfileUserDto {
            id: profile_user.user_id,
            nickname: profile_user.nickname,
            email: profile_user.email,
            avatar: profile_user.avatar.clone(),
            descript: profile_user.descript.clone(),
            theme: profile_user.theme.clone(),
            created_at: profile_user.created_at.clone(),
            updated_at: profile_user.updated_at.clone(),
        }
    }
}

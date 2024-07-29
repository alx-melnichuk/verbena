use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema;
use crate::users::user_models::User;
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
pub const PROFILE_THEME_DEF: &str = "light";

impl Profile {
    pub fn new(user_id: i32, avatar: Option<String>, descript: Option<String>) -> Profile {
        let now = Utc::now();
        Profile {
            user_id: user_id,
            avatar: avatar,
            descript: descript.unwrap_or(PROFILE_DESCRIPT_DEF.to_string()),
            theme: PROFILE_THEME_DEF.to_string(),
            created_at: now,
            updated_at: now,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUserDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
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

impl ProfileUserDto {
    pub fn new(user: User, profile: Profile) -> ProfileUserDto {
        let updated_at = if user.updated_at > profile.updated_at {
            user.updated_at
        } else {
            profile.updated_at
        };
        ProfileUserDto {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            avatar: profile.avatar,
            descript: profile.descript,
            theme: profile.theme,
            created_at: user.created_at,
            updated_at: updated_at,
        }
    }
}

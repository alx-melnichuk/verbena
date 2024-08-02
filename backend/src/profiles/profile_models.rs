use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::users::user_models::{User, UserRole};
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
    pub fn new(user_id: i32, avatar: Option<&str>, descript: Option<&str>, theme: Option<&str>) -> Profile {
        let now = Utc::now();
        Profile {
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
    #[diesel(sql_type = crate::schema::sql_types::UserRole)]
    #[diesel(column_name = "role")]
    pub role: UserRole,
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
        role: UserRole,
        avatar: Option<&str>,
        descript: &str,
        theme: &str,
    ) -> ProfileUser {
        let now = Utc::now();
        ProfileUser {
            user_id,
            nickname: nickname.to_string(),
            email: email.to_string(),
            role,
            avatar: avatar.map(|v| v.to_string()),
            descript: descript.to_string(),
            theme: theme.to_string(),
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
pub struct ProfileUserDto {
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

impl ProfileUserDto {
    pub fn from(profile_user: ProfileUser) -> ProfileUserDto {
        ProfileUserDto {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProfileUser {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
    pub avatar: Option<String>,   // min_len=2 max_len=255 Nullable
    pub descript: Option<String>, // type: Text default ""
    pub theme: Option<String>,    // min_len=2 max_len=32 default "light"
}

impl CreateProfileUser {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> CreateProfileUser {
        CreateProfileUser {
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

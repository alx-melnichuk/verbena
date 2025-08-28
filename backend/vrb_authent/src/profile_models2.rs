use std::convert::From;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::serial_datetime;
use vrb_dbase::{enm_user_role::UserRole, schema};

use crate::user_models::User;


// * * * * Section: "database". * * * *

pub const PROFILE_THEME_LIGHT_DEF: &str = "light";
pub const PROFILE_THEME_DARK: &str = "dark";
pub const PROFILE_LOCALE_DEF: &str = "default";

// * * * * Section: models for "ProfileOrm". * * * *

// ** Model: "Profile". Used to return user profile data. **

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
    #[diesel(sql_type = schema::sql_types::UserRole)]
    #[diesel(column_name = "role")]
    pub role: UserRole,
    pub avatar: Option<String>,
    pub descript: Option<String>,
    pub theme: Option<String>,
    pub locale: Option<String>,
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
        locale: Option<&str>,
    ) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            nickname: nickname.to_owned(),
            email: email.to_owned(),
            password: "".to_owned(),
            role,
            avatar: avatar.map(|v| v.to_owned()),
            descript: descript.map(|v| v.to_owned()),
            theme: theme.map(|v| v.to_owned()),
            locale: locale.map(|v| v.to_owned()),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
    pub fn new2(
        user_id: i32,
        nickname: &str,
        email: &str,
        password: &str,
        role: UserRole,
        avatar: Option<&str>,
        descript: Option<&str>,
        theme: Option<&str>,
        locale: Option<&str>,
    ) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            nickname: nickname.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
            role,
            avatar: avatar.map(|v| v.to_owned()),
            descript: descript.map(|v| v.to_owned()),
            theme: theme.map(|v| v.to_owned()),
            locale: locale.map(|v| v.to_owned()),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

impl From<User> for Profile {
    fn from(user: User) -> Self {
        Profile {
            user_id: user.id,
            nickname: user.nickname,
            email: user.email,
            password: user.password,
            role: user.role,
            avatar: None,
            descript: None,
            theme: None,
            locale: None,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// ** Model Dto: "ProfileDto". Used: in "user_registr_controller::confirm_registration()". **

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
    pub descript: Option<String>, // type: Text default ""
    // Default color theme. ["light","dark"]
    pub theme: Option<String>, // min_len=2 max_len=32 default "light"
    // Default locale.
    pub locale: Option<String>, // min_len=2 max_len=32 default "default"
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl From<Profile> for ProfileDto {
    fn from(profile: Profile) -> Self {
        ProfileDto {
            id: profile.user_id,
            nickname: profile.nickname,
            email: profile.email,
            role: profile.role.clone(),
            avatar: profile.avatar.clone(),
            descript: profile.descript.clone(),
            theme: profile.theme.clone(),
            locale: profile.locale.clone(),
            created_at: profile.created_at.clone(),
            updated_at: profile.updated_at.clone(),
        }
    }
}

impl From<User> for ProfileDto {
    fn from(user: User) -> Self {
        ProfileDto {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            role: user.role,
            avatar: None,
            descript: None,
            theme: None,
            locale: None,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// ** Model: "CreateProfile". Used: UserOrm::create_profile_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateProfile {
    pub nickname: String,         // min_len=3 max_len=64
    pub email: String,            // min_len=5 max_len=254
    pub password: String,         // min_len=6 max_len=64
    pub role: Option<UserRole>,   // default "user"
    pub avatar: Option<String>,   // min_len=2 max_len=255 Nullable
    pub descript: Option<String>, // min_len=2,max_len=2048 default ""
    pub theme: Option<String>,    // min_len=2 max_len=32 default "light"
    pub locale: Option<String>,   // min_len=2 max_len=32 default "default"
}

impl CreateProfile {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> CreateProfile {
        CreateProfile {
            nickname: nickname.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
            role: role.clone(),
            avatar: None,
            descript: None,
            theme: None,
            locale: None,
        }
    }
}

// ** Model: "ModifyProfile". Used: UserOrm::modify_profile() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModifyProfile {
    pub nickname: Option<String>,       // min_len=3,max_len=64
    pub email: Option<String>,          // min_len=5,max_len=254,"email:email_type"
    pub password: Option<String>,       // min_len=6,max_len=64
    pub role: Option<UserRole>,         // default "user"
    pub avatar: Option<Option<String>>, // min_len=2,max_len=255 Nullable
    pub descript: Option<String>,       // min_len=2,max_len=2048 default ""
    pub theme: Option<String>,          // min_len=2,max_len=32 default "light"
    pub locale: Option<String>,         // min_len=2,max_len=32 default "default"
}

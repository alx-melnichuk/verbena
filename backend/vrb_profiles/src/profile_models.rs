use std::convert::From;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_authent::user_models::{self, Profile, User};
use vrb_common::{
    err, profile, serial_datetime,
    validators::{ValidationChecks, ValidationError, Validator},
};
use vrb_dbase::{enm_user_role::UserRole, schema};

// #
pub fn validate_nickname_or_email(value: &str) -> Result<(), ValidationError> {
    if value.contains("@") {
        user_models::validate_email(&value).map_err(|err| err)?;
    } else {
        user_models::validate_nickname(&value).map_err(|err| err)?;
    }
    Ok(())
}

// * * * * Section: models for "ProfileOrm". * * * *

// ** Used to return user profile data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "user_id")]
    pub user_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "nickname")]
    pub nickname: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "email")]
    pub email: String,
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

impl UserProfile {
    pub fn new(
        user_id: i32,
        nickname: &str,
        email: &str,
        role: UserRole,
        avatar: Option<&str>,
        descript: Option<&str>,
        theme: Option<&str>,
        locale: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        UserProfile {
            user_id,
            nickname: nickname.to_owned(),
            email: email.to_owned(),
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

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            user_id: user.id,
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

// ** Used: ProfileOrm::modify_user_profile() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModifyUserProfile {
    pub nickname: Option<String>,       // min_len=3,max_len=64
    pub email: Option<String>,          // min_len=5,max_len=254,"email:email_type"
    pub password: Option<String>,       // min_len=6,max_len=64
    pub role: Option<UserRole>,         // default "user"
    pub avatar: Option<Option<String>>, // min_len=2,max_len=255 Nullable
    pub descript: Option<String>,       // min_len=2,max_len=2048 default ""
    pub theme: Option<String>,          // min_len=2,max_len=32 default "light"
    pub locale: Option<String>,         // min_len=2,max_len=32 default "default"
}

// * * * * Section: models for the "profile_get_controller". * * * *

// ** Used: in "profile_controller::put_profile()" and many other methods. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileDto {
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

impl UserProfileDto {
    pub fn update_profile(&mut self, profile: Profile) -> &mut Self {
        self.avatar = profile.avatar;
        self.descript = profile.descript;
        self.theme = profile.theme;
        self.locale = profile.locale;
        self
    }
}

impl From<UserProfile> for UserProfileDto {
    fn from(profile: UserProfile) -> Self {
        UserProfileDto {
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

// ** Used: in "profile_controller::get_profile_mini_by_id()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileMiniDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub role: UserRole,
    // Link to user avatar, optional
    pub avatar: Option<String>, // min_len=2 max_len=255 Nullable
}

impl From<UserProfile> for UserProfileMiniDto {
    fn from(profile: UserProfile) -> Self {
        UserProfileMiniDto {
            id: profile.user_id,
            nickname: profile.nickname,
            email: profile.email,
            role: profile.role.clone(),
            avatar: profile.avatar.clone(),
        }
    }
}
// ** Used: in "profile_get_controller::get_profile_config()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileConfigDto {
    // Maximum size for avatar files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_max_size: Option<u32>,
    // List of valid input mime types for avatar files.
    // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
    pub avatar_valid_types: Vec<String>,
    // Avatar files will be converted to this MIME type.
    // Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_ext: Option<String>,
    // Maximum width of avatar image after saving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_max_width: Option<u32>,
    // Maximum height of avatar image after saving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_max_height: Option<u32>,
}

impl ProfileConfigDto {
    pub fn new(
        max_size: Option<u32>,
        valid_types: Vec<String>,
        ext: Option<String>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> ProfileConfigDto {
        ProfileConfigDto {
            avatar_max_size: max_size.clone(),
            avatar_valid_types: valid_types.clone(),
            avatar_ext: ext.clone(),
            avatar_max_width: max_width.clone(),
            avatar_max_height: max_height.clone(),
        }
    }
}

// * * * * Section: models for the "profile_controller". * * * *

// ** Used: in "profile_controller::put_profile()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyUserProfileDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>, // min_len=3,max_len=64
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>, // min_len=5,max_len=254,"email:email_type"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>, // min_len=2,max_len=2048 default ""
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>, // min_len=2,max_len=32 default "light"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>, // min_len=2,max_len=32 default "default"
}

impl ModifyUserProfileDto {
    pub fn valid_names<'a>() -> Vec<&'a str> {
        vec!["nickname", "email", "role", "descript", "theme", "locale"]
    }
}

impl Validator for ModifyUserProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.nickname {
            errors.push(user_models::validate_nickname(&value).err());
        }
        if let Some(value) = &self.email {
            errors.push(user_models::validate_email(&value).err());
        }
        if let Some(value) = &self.role {
            errors.push(user_models::validate_role(&value).err());
        }
        if let Some(value) = &self.descript {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(profile::validate_descript(&value).err());
            }
        }
        if let Some(value) = &self.theme {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(profile::validate_theme(&value).err());
            }
        }
        if let Some(value) = &self.locale {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(profile::validate_locale(&value).err());
            }
        }

        let list_is_some = vec![
            self.nickname.is_some(),
            self.email.is_some(),
            self.role.is_some(),
            self.descript.is_some(),
            self.theme.is_some(),
            self.locale.is_some(),
        ];
        let valid_names = Self::valid_names().join(",");
        #[rustfmt::skip]
        errors.push(
            ValidationChecks::no_fields_to_update(&list_is_some, &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err()
        );

        self.filter_errors(errors)
    }
}

impl Into<ModifyUserProfile> for ModifyUserProfileDto {
    fn into(self) -> ModifyUserProfile {
        let role = if let Some(role1) = self.role {
            UserRole::try_from(role1.as_str()).ok()
        } else {
            None
        };
        ModifyUserProfile {
            nickname: self.nickname.clone(),
            email: self.email.clone(),
            password: None,
            role: role,
            avatar: None,
            descript: self.descript.clone(),
            theme: self.theme.clone(),
            locale: self.locale.clone(),
        }
    }
}

// ** Used: in "profile_controller::put_profile_new_password()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewPasswordUserProfileDto {
    pub password: String,
    pub new_password: String,
}

impl Validator for NewPasswordUserProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_models::validate_password(&self.password).err());
        errors.push(user_models::validate_new_password(&self.new_password).err());

        // Determine whether there are errors.
        let is_exist_error = errors.iter().any(|err| err.is_some());
        if !is_exist_error {
            errors.push(user_models::validate_inequality(&self.new_password, &self.password).err());
        }

        self.filter_errors(errors)
    }
}

// ** Used: in "profile_controller::delete_profile()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
pub struct StreamLogo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "logo")]
    pub logo: String,
}

// * * * *  ProfileMock  * * * *

#[cfg(any(test, feature = "mockdata"))]
pub struct ProfileMock {}

#[cfg(any(test, feature = "mockdata"))]
impl ProfileMock {
    pub fn descript_min() -> String {
        (0..(profile::DESCRIPT_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn descript_max() -> String {
        (0..(profile::DESCRIPT_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn theme_min() -> String {
        (0..(profile::THEME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn theme_max() -> String {
        (0..(profile::THEME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn locale_min() -> String {
        (0..(profile::LOCALE_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn locale_max() -> String {
        (0..(profile::LOCALE_MAX + 1)).map(|_| 'a').collect()
    }
}

// * * * *    * * * *

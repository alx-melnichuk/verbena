use std::convert::From;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_dbase::db_enums::UserRole;
use vrb_dbase::schema;
use vrb_tools::{
    err, serial_datetime,
    validators::{ValidationChecks, ValidationError, Validator},
};

pub const NICKNAME_MIN: u8 = 3;
pub const NICKNAME_MAX: u8 = 64;
pub const NICKNAME_REGEX: &str = r"^[a-zA-Z]+[\w]+$";
// \w   Matches any letter, digit or underscore. Equivalent to [a-zA-Z0-9_].
// \W - Matches anything other than a letter, digit or underscore. Equivalent to [^a-zA-Z0-9_]
pub const MSG_NICKNAME_REQUIRED: &str = "nickname:required";
pub const MSG_NICKNAME_MIN_LENGTH: &str = "nickname:min_length";
pub const MSG_NICKNAME_MAX_LENGTH: &str = "nickname:max_length";
pub const MSG_NICKNAME_REGEX: &str = "nickname:regex";

pub const EMAIL_MIN: u8 = 5;
// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address?
// Answer: An email address must not exceed 254 characters.
pub const EMAIL_MAX: u16 = 254;
pub const MSG_EMAIL_REQUIRED: &str = "email:required";
pub const MSG_EMAIL_MIN_LENGTH: &str = "email:min_length";
pub const MSG_EMAIL_MAX_LENGTH: &str = "email:max_length";
pub const MSG_EMAIL_EMAIL_TYPE: &str = "email:email_type";

pub const PASSWORD_MIN: u8 = 6;
pub const PASSWORD_MAX: u8 = 64;
pub const PASSWORD_LOWERCASE_LETTER_REGEX: &str = r"[a-z]+";
pub const PASSWORD_CAPITAL_LETTER_REGEX: &str = r"[A-Z]+";
pub const PASSWORD_NUMBER_REGEX: &str = r"[\d]+";
// pub const PASSWORD_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d\W_]{6,}$";
pub const MSG_PASSWORD_REQUIRED: &str = "password:required";
pub const MSG_PASSWORD_MIN_LENGTH: &str = "password:min_length";
pub const MSG_PASSWORD_MAX_LENGTH: &str = "password:max_length";
pub const MSG_PASSWORD_REGEX: &str = "password:regex";
pub const MSG_NEW_PASSWORD_REQUIRED: &str = "new_password:required";
pub const MSG_NEW_PASSWORD_MIN_LENGTH: &str = "new_password:min_length";
pub const MSG_NEW_PASSWORD_MAX_LENGTH: &str = "new_password:max_length";
pub const MSG_NEW_PASSWORD_REGEX: &str = "new_password:regex";
pub const MSG_NEW_PASSWORD_EQUAL_OLD_VALUE: &str = "new_password:equal_to_old_value";

pub const DESCRIPT_MIN: u8 = 2;
pub const MSG_DESCRIPT_MIN_LENGTH: &str = "descript:min_length";
pub const DESCRIPT_MAX: u16 = 2048; // 2*1024
pub const MSG_DESCRIPT_MAX_LENGTH: &str = "descript:max_length";

pub const THEME_MIN: u8 = 2;
pub const MSG_THEME_MIN_LENGTH: &str = "theme:min_length";
pub const THEME_MAX: u8 = 32;
pub const MSG_THEME_MAX_LENGTH: &str = "theme:max_length";

pub const LOCALE_MIN: u8 = 2;
pub const MSG_LOCALE_MIN_LENGTH: &str = "locale:min_length";
pub const LOCALE_MAX: u8 = 32;
pub const MSG_LOCALE_MAX_LENGTH: &str = "locale:max_length";

pub const MSG_USER_ROLE_INVALID_VALUE: &str = "user_role:invalid_value";

// MIN=3, MAX=64, REG="^[a-zA-Z]+[\\w]+$"
pub fn validate_nickname(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NICKNAME_REQUIRED)?;
    ValidationChecks::min_length(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX_LENGTH)?;
    ValidationChecks::regexp(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
    Ok(())
}
// MIN=5, MAX=254, "email:email_type"
pub fn validate_email(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_EMAIL_REQUIRED)?;
    ValidationChecks::min_length(value, EMAIL_MIN.into(), MSG_EMAIL_MIN_LENGTH)?;
    ValidationChecks::max_length(value, EMAIL_MAX.into(), MSG_EMAIL_MAX_LENGTH)?;
    ValidationChecks::email(value, MSG_EMAIL_EMAIL_TYPE)?;
    Ok(())
}

pub fn validate_nickname_or_email(value: &str) -> Result<(), ValidationError> {
    if value.contains("@") {
        validate_email(&value).map_err(|err| err)?;
    } else {
        validate_nickname(&value).map_err(|err| err)?;
    }
    Ok(())
}
// MIN=6, MAX=64, REG="[a-z]+","[A-Z]+","[\\d]+"
pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_PASSWORD_REGEX)?;
    Ok(())
}
// MIN=6, MAX=64, REG="[a-z]+","[A-Z]+","[\\d]+" OR "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d\\W_]{6,}$"
pub fn validate_new_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NEW_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_NEW_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_NEW_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_NEW_PASSWORD_REGEX)?;
    Ok(())
}
pub fn validate_inequality(value1: &str, value2: &str) -> Result<(), ValidationError> {
    if value1.starts_with(value2) && value1.len() == value2.len() {
        let err = ValidationError::new(MSG_NEW_PASSWORD_EQUAL_OLD_VALUE);
        return Err(err);
    }
    Ok(())
}
// MIN=2, MAX=2048
pub fn validate_descript(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, DESCRIPT_MIN.into(), MSG_DESCRIPT_MIN_LENGTH)?;
    ValidationChecks::max_length(value, DESCRIPT_MAX.into(), MSG_DESCRIPT_MAX_LENGTH)?;
    Ok(())
}
// MIN=2, MAX=32
pub fn validate_theme(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, THEME_MIN.into(), MSG_THEME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, THEME_MAX.into(), MSG_THEME_MAX_LENGTH)?;
    Ok(())
}
// MIN=2, MAX=32
pub fn validate_locale(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, LOCALE_MIN.into(), MSG_LOCALE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, LOCALE_MAX.into(), MSG_LOCALE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_role(value: &str) -> Result<(), ValidationError> {
    let res_user_role = UserRole::try_from(value);
    if res_user_role.is_err() {
        ValidationChecks::valid_value(value, &[], MSG_USER_ROLE_INVALID_VALUE)?;
    }
    Ok(())
}

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
}

// ** Model: "CreateProfile". Used: ProfileOrm::create_profile_user() **

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

// ** Model: "ModifyProfile". Used: ProfileOrm::modify_profile() **

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

// ** Model: "Session". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub user_id: i32,
    pub num_token: Option<i32>,
}

// * * * * Section: models for the "profile_get_controller". * * * *

// ** Model Dto: "UniquenessProfileDto". Used: in "profile_get_controller::uniqueness_check()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UniquenessProfileDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ** Model Dto: "UniquenessProfileResponseDto". Used: in "profile_get_controller::uniqueness_check()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UniquenessProfileResponseDto {
    pub uniqueness: bool,
}

impl UniquenessProfileResponseDto {
    pub fn new(uniqueness: bool) -> Self {
        UniquenessProfileResponseDto { uniqueness }
    }
}

// ** Model Dto: "ProfileDto". Used: in "profile_get_controller::get_profile_by_id()" and many other methods. **
// **                          Used: in "profile_controller::put_profile()" and many other methods. **

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

// ** Model Dto: "ProfileConfigDto". Used: in "profile_get_controller::get_profile_config()". **

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

// ** Model Dto: "ModifyProfileDto". Used: in "profile_controller::put_profile()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyProfileDto {
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

impl ModifyProfileDto {
    pub fn valid_names<'a>() -> Vec<&'a str> {
        vec!["nickname", "email", "role", "descript", "theme", "locale"]
    }
}

impl Validator for ModifyProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.nickname {
            errors.push(validate_nickname(&value).err());
        }
        if let Some(value) = &self.email {
            errors.push(validate_email(&value).err());
        }
        if let Some(value) = &self.role {
            errors.push(validate_role(&value).err());
        }
        if let Some(value) = &self.descript {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(validate_descript(&value).err());
            }
        }
        if let Some(value) = &self.theme {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(validate_theme(&value).err());
            }
        }
        if let Some(value) = &self.locale {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(validate_locale(&value).err());
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
        let valid_names = ModifyProfileDto::valid_names().join(",");
        #[rustfmt::skip]
        errors.push(
            ValidationChecks::no_fields_to_update(&list_is_some, &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err()
        );

        self.filter_errors(errors)
    }
}

impl Into<ModifyProfile> for ModifyProfileDto {
    fn into(self) -> ModifyProfile {
        let role = if let Some(role1) = self.role {
            UserRole::try_from(role1.as_str()).ok()
        } else {
            None
        };
        ModifyProfile {
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

// ** Model Dto: "NewPasswordProfileDto". Used: in "profile_controller::put_profile_new_password()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewPasswordProfileDto {
    pub password: String,
    pub new_password: String,
}

impl Validator for NewPasswordProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_password(&self.password).err());
        errors.push(validate_new_password(&self.new_password).err());

        // Determine whether there are errors.
        let is_exist_error = errors.iter().any(|err| err.is_some());
        if !is_exist_error {
            errors.push(validate_inequality(&self.new_password, &self.password).err());
        }

        self.filter_errors(errors)
    }
}


// ** Model Dto: "NewPasswordProfileDto". Used: in "profile_controller::delete_profile()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
pub struct StreamLogo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "logo")]
    pub logo: String,
}

// * * * * Section: models for the "profile_auth_controller". * * * *

// ** Model Dto: "LoginProfileDto". Used: in "profile_auth_controller::login()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginProfileDto {
    pub nickname: String,
    pub password: String,
}

impl Validator for LoginProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname_or_email(&self.nickname).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "LoginProfileResponseDto". Used: in "profile_auth_controller::login()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginProfileResponseDto {
    pub profile_dto: ProfileDto,
    pub profile_tokens_dto: ProfileTokensDto,
}

// ** Model Dto: "TokenDto". Used: in "profile_auth_controller::update_token(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenDto {
    // refreshToken
    pub token: String,
}

// ** Model Dto: "ProfileTokensDto". Used: in "profile_auth_controller::update_token(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTokensDto {
    pub access_token: String,
    pub refresh_token: String,
}

// * * * * Section: models for the "profile_registr_controller". * * * *

// ** Model Dto: "RegistrProfileDto". Used: in "profile_registr_controller::registration(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrProfileDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl Validator for RegistrProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname(&self.nickname).err());
        errors.push(validate_email(&self.email).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "RegistrProfileResponseDto". Used: in "profile_registr_controller::registration(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrProfileResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
}

// ** Model Dto: "RecoveryProfileDto". Used: in "profile_registr_controller::recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryProfileDto {
    pub email: String,
}

impl Validator for RecoveryProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_email(&self.email).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "RecoveryProfileResponseDto". Used: in "profile_registr_controller::recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryProfileResponseDto {
    pub id: i32,
    pub email: String,
    pub recovery_token: String,
}

// ** Model Dto: "RecoveryDataDto". Used: in "profile_registr_controller::confirm_recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDataDto {
    pub password: String,
}

impl Validator for RecoveryDataDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "ClearForExpiredResponseDto". Used: in "profile_registr_controller::clear_for_expired(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClearForExpiredResponseDto {
    pub count_inactive_registr: usize,
    pub count_inactive_recover: usize,
}

// * * * *   * * * *

#[cfg(all(test, feature = "mockdata"))]
pub struct ProfileTest {}

#[cfg(all(test, feature = "mockdata"))]
impl ProfileTest {
    pub fn nickname_min() -> String {
        (0..(NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn nickname_max() -> String {
        (0..(NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn nickname_wrong() -> String {
        let nickname: String = (0..(NICKNAME_MIN - 1)).map(|_| 'a').collect();
        format!("{}#", nickname)
    }
    pub fn email_min() -> String {
        let suffix = "@us".to_owned();
        let email_min: usize = EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn email_max() -> String {
        let email_max: usize = EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        format!("{}@{}{}", prefix, suffix, domain)
    }
    pub fn email_wrong() -> String {
        let suffix = "@".to_owned();
        let email_min: usize = EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn password_min() -> String {
        (0..(PASSWORD_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn password_max() -> String {
        (0..(PASSWORD_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn password_wrong() -> String {
        (0..(PASSWORD_MIN)).map(|_| 'a').collect()
    }
    pub fn role_wrong() -> String {
        let role = UserRole::all_values().get(0).unwrap().to_string();
        role[0..(role.len() - 1)].to_string()
    }
    pub fn descript_min() -> String {
        (0..(DESCRIPT_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn descript_max() -> String {
        (0..(DESCRIPT_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn theme_min() -> String {
        (0..(THEME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn theme_max() -> String {
        (0..(THEME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn locale_min() -> String {
        (0..(LOCALE_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn locale_max() -> String {
        (0..(LOCALE_MAX + 1)).map(|_| 'a').collect()
    }
}

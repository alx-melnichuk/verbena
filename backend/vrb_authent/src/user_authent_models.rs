use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use vrb_common::{
    serial_datetime, user_validations,
    validators::{ValidationError, Validator},
};
use vrb_dbase::enm_user_role::UserRole;

use crate::user_models::User;

pub fn validate_nickname_or_email(value: &str) -> Result<(), ValidationError> {
    if value.contains("@") {
        user_validations::validate_email(&value).map_err(|err| err)?;
    } else {
        user_validations::validate_nickname(&value).map_err(|err| err)?;
    }
    Ok(())
}

// ** Section: "User Authent" **

// ** Used: in "user_authent_controller::users_uniqueness(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUniquenessDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ** Used: in "user_authent_controller::users_uniqueness(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUniquenessResponseDto {
    pub uniqueness: bool,
}

impl UserUniquenessResponseDto {
    pub fn new(uniqueness: bool) -> Self {
        UserUniquenessResponseDto { uniqueness }
    }
}

// ** Used: in "user_authent_controller::login()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub nickname: String,
    pub password: String,
}

impl Validator for LoginDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname_or_email(&self.nickname).err());
        errors.push(user_validations::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Used: in "user_authent_controller::login(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserProfileDto {
    pub id: i32,
    pub nickname: String, // max_len: 255
    pub email: String,    // max_len: 255
    pub role: UserRole,
    pub avatar: Option<String>, // min_len=2 max_len=255 Nullable
    pub descript: Option<String>, // type: Text default ""
    pub theme: Option<String>, // min_len=2 max_len=32 default "light"
    pub locale: Option<String>, // min_len=2 max_len=32 default "default"
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl From<User> for LoginUserProfileDto {
    fn from(user: User) -> Self {
        LoginUserProfileDto {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            role: user.role.clone(),
            avatar: None,
            descript: None,
            theme: None,
            locale: None,
            created_at: user.created_at.clone(),
            updated_at: user.updated_at.clone(),
        }
    }
}

// ** Used: in "user_authent_controller::login()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseDto {
    pub user_profile_dto: LoginUserProfileDto,
    pub token_user_response_dto: TokenUserResponseDto,
}

// ** Used: in "user_authent_controller::update_token(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenUserDto {
    // refreshToken
    pub token: String,
}

// ** Used: in "user_authent_controller::update_token(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenUserResponseDto {
    pub access_token: String,
    pub refresh_token: String,
}

// ** - **

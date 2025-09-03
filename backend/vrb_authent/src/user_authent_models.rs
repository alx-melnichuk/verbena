use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

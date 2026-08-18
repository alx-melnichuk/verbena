use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use vrb_common::validators::{ValidationError, Validator};

use crate::user_models;

// ** Section: "UserRecovery" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FromRow)]
pub struct UserRecovery {
    pub id: i32,
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct CreateUserRecovery {
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

// ** Used: in "user_recovery_controller::recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryUserDto {
    pub email: String,
}

impl Validator for RecoveryUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_models::validate_email(&self.email).err());

        self.filter_errors(errors)
    }
}

// ** Used: in "user_recovery_controller::recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryUserResponseDto {
    pub id: i32,
    pub email: String,
    pub recovery_token: String,
}

// ** Used: in "user_recovery_controller::confirm_recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmRecoveryUserResponseDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ** Model Dto: "RecoveryDataDto". Used: in "user_recovery_controller::confirm_recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDataDto {
    pub password: String,
}

impl Validator for RecoveryDataDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_models::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "RecoveryClearForExpiredResponseDto". Used: in "user_recovery_controller::clear_for_expired(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryClearForExpiredResponseDto {
    pub count_inactive_recover: usize,
}

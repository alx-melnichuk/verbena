use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::{
    validators::{ValidationError, Validator},
    user_validations,
};
use vrb_dbase::schema;

// ** Section: "UserRecovery" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::user_recovery)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRecovery {
    pub id: i32,
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_recovery)]
pub struct CreateUserRecovery {
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

// ** Model Dto: "RecoveryProfileDto". Used: in "user_recovery_controller::recovery(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryProfileDto {
    pub email: String,
}

impl Validator for RecoveryProfileDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_validations::validate_email(&self.email).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "RecoveryProfileResponseDto". Used: in "user_recovery_controller::recovery(). **

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

        errors.push(user_validations::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

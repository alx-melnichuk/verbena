use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::{
    user_validations,
    /*err, serial_datetime,*/
    validators::{/*ValidationChecks,*/ ValidationError, Validator},
};
use vrb_dbase::schema;

// * * * * Section: models for "UserRegistrOrm". * * * *

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::user_registration)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRegistr {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_registration)]
pub struct CreateUserRegistr {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

// * * * * Section: models for the "user_registr_controller". * * * *

// ** Used: in "user_registr_controller::registration(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrUserDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl Validator for RegistrUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_validations::validate_nickname(&self.nickname).err());
        errors.push(user_validations::validate_email(&self.email).err());
        errors.push(user_validations::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Used: in "user_registr_controller::registration(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrUserResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
}

// ** Used: in "user_registr_controller::confirm_registration(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmRegistrUserResponseDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

// ** Model Dto: "RegistrationClearForExpiredResponseDto". Used: in "user_registr_controller::clear_for_expired(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationClearForExpiredResponseDto {
    pub count_inactive_registr: usize,
}

// ** - **

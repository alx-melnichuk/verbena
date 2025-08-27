use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::{
    serial_datetime,
    validators::{ValidationError, Validator},
    user_validations,
};
use vrb_dbase::{db_enums::UserRole, schema};

// ** Section: database "users" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name = schema::users)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            id: user.id,
            nickname: user.nickname.to_owned(),
            email: user.email.to_owned(),
            password: "".to_string(),
            role: user.role.to_owned(),
            created_at: user.created_at.to_owned(),
            updated_at: user.updated_at.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUserDto {
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

impl Validator for ModifyUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(nickname_val) = &self.nickname {
            errors.push(user_validations::validate_nickname(nickname_val).err());
        }
        if let Some(email_val) = &self.email {
            errors.push(user_validations::validate_email(email_val).err());
        }
        if let Some(password_val) = &self.password {
            errors.push(user_validations::validate_password(password_val).err());
        }

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NewPasswordUserDto {
    // TODO del;
    pub password: String,
    pub new_password: String,
}

impl Validator for NewPasswordUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(user_validations::validate_password(&self.password).err());

        errors.push(user_validations::validate_new_password(&self.new_password).err());

        errors.push(user_validations::validate_inequality(&self.new_password, &self.password).err());

        self.filter_errors(errors)
    }
}

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

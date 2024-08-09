use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::profiles::profile_models;
use crate::schema;
use crate::utils::serial_datetime;
use crate::validators::{ValidationError, Validator};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, diesel_derive_enum::DbEnum, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
        }
    }
}

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
            errors.push(profile_models::validate_nickname(nickname_val).err());
        }
        if let Some(email_val) = &self.email {
            errors.push(profile_models::validate_email(email_val).err());
        }
        if let Some(password_val) = &self.password {
            errors.push(profile_models::validate_password(password_val).err());
        }

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NewPasswordUserDto {
    pub password: String,
    pub new_password: String,
}

impl Validator for NewPasswordUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_password(&self.password).err());

        errors.push(profile_models::validate_new_password(&self.new_password).err());

        errors.push(profile_models::validate_inequality(&self.new_password, &self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUser {
    pub nickname: String,       // min_len=3 max_len=64
    pub email: String,          // min_len=5 max_len=254
    pub password: String,       // min_len=6 max_len=64
    pub role: Option<UserRole>, // default "user"
}

impl CreateUser {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> CreateUser {
        CreateUser {
            nickname: nickname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            role: role.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDto {
    pub nickname: String, // min_len=3 max_len=64
    pub email: String,    // min_len=5 max_len=254
    pub password: String, // min_len=6 max_len=64
}

impl Validator for CreateUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_nickname(&self.nickname).err());
        errors.push(profile_models::validate_email(&self.email).err());
        errors.push(profile_models::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// * UniquenessUserDto *

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UniquenessUserDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ** Section: "LoginUser" **

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, ToSchema)]
#[diesel(table_name = schema::users)]
pub struct LoginUserDto {
    pub nickname: String,
    pub password: String,
}

impl Validator for LoginUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_nickname_or_email(&self.nickname).err());
        errors.push(profile_models::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserResponseDto {
    pub user_dto: UserDto,
    pub user_tokens_dto: UserTokensDto,
}

// ** Section: "UserRegistr" **

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::user_registration)]
#[serde(rename_all = "camelCase")]
pub struct UserRegistrDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    #[serde(with = "serial_datetime")]
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_registration)]
pub struct CreateUserRegistrDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RegistrUserDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl Validator for RegistrUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_nickname(&self.nickname).err());
        errors.push(profile_models::validate_email(&self.email).err());
        errors.push(profile_models::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistrUserResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::user_recovery)]
#[serde(rename_all = "camelCase")]
pub struct UserRecoveryDto {
    pub id: i32,
    pub user_id: i32,
    #[serde(with = "serial_datetime")]
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_recovery)]
pub struct CreateUserRecoveryDto {
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RecoveryUserDto {
    pub email: String,
}

impl Validator for RecoveryUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_email(&self.email).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryUserResponseDto {
    pub id: i32,
    pub email: String,
    pub recovery_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RecoveryDataDto {
    pub password: String,
}

impl Validator for RecoveryDataDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(profile_models::validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDataResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
}

// ** Section: "UserToken" // TODO del **

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserTokensDto {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TokenUserDto {
    // refreshToken
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClearForExpiredResponseDto {
    pub count_inactive_registr: usize,
    pub count_inactive_recover: usize,
}

#[cfg(all(test, feature = "mockdata"))]
pub struct UserModelsTest {}

#[cfg(all(test, feature = "mockdata"))]
impl UserModelsTest {
    pub fn nickname_min() -> String {
        (0..(profile_models::NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn nickname_max() -> String {
        (0..(profile_models::NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn nickname_wrong() -> String {
        let nickname: String = (0..(profile_models::NICKNAME_MIN - 1)).map(|_| 'a').collect();
        format!("{}#", nickname)
    }
    pub fn email_min() -> String {
        let suffix = "@us".to_string();
        let email_min: usize = profile_models::EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn email_max() -> String {
        let email_max: usize = profile_models::EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        format!("{}@{}{}", prefix, suffix, domain)
    }
    pub fn email_wrong() -> String {
        let suffix = "@".to_string();
        let email_min: usize = profile_models::EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn password_min() -> String {
        (0..(profile_models::PASSWORD_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn password_max() -> String {
        (0..(profile_models::PASSWORD_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn password_wrong() -> String {
        (0..(profile_models::PASSWORD_MIN)).map(|_| 'a').collect()
    }
}

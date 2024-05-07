use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::utils::serial_datetime;
use crate::validators::{ValidationChecks, ValidationError, Validator};

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
// MIN=3,MAX=64,"^[a-zA-Z]+[\\w]+$"
pub fn validate_nickname(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_NICKNAME_REQUIRED)?;
    ValidationChecks::min_length(value, NICKNAME_MIN.into(), MSG_NICKNAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, NICKNAME_MAX.into(), MSG_NICKNAME_MAX_LENGTH)?;
    ValidationChecks::regexp(value, NICKNAME_REGEX, MSG_NICKNAME_REGEX)?; // /^[a-zA-Z]+[\w]+$/
    Ok(())
}
// MIN=5,MAX=255,"email:email_type"
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
// MIN=6,MAX=64,"[a-z]+","[A-Z]+","[\\d]+"
pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_PASSWORD_REQUIRED)?;
    ValidationChecks::min_length(value, PASSWORD_MIN.into(), MSG_PASSWORD_MIN_LENGTH)?;
    ValidationChecks::max_length(value, PASSWORD_MAX.into(), MSG_PASSWORD_MAX_LENGTH)?;
    ValidationChecks::regexp(value, PASSWORD_LOWERCASE_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_CAPITAL_LETTER_REGEX, MSG_PASSWORD_REGEX)?;
    ValidationChecks::regexp(value, PASSWORD_NUMBER_REGEX, MSG_PASSWORD_REGEX)?;
    Ok(())
}

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
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

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, ToSchema)]
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
            errors.push(validate_nickname(nickname_val).err());
        }
        if let Some(email_val) = &self.email {
            errors.push(validate_email(email_val).err());
        }
        if let Some(password_val) = &self.password {
            errors.push(validate_password(password_val).err());
        }

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, ToSchema)]
#[diesel(table_name = schema::users)]
pub struct PasswordUserDto {
    pub password: Option<String>,
}

impl Validator for PasswordUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(password_val) = &self.password {
            errors.push(validate_password(password_val).err());
        }

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl Validator for CreateUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname(&self.nickname).err());
        errors.push(validate_email(&self.email).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

// ** Section: "Login User" **

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

        errors.push(validate_nickname_or_email(&self.nickname).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserResponseDto {
    pub user_dto: UserDto,
    pub user_tokens_dto: UserTokensDto,
}

// ** Section: database "user_registration" **

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistrUserDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl Validator for RegistrUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_nickname(&self.nickname).err());
        errors.push(validate_email(&self.email).err());
        errors.push(validate_password(&self.password).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrUserResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
}

// ** Section: database "user_recovery" **

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecoveryUserDto {
    pub email: String,
}

impl Validator for RecoveryUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_email(&self.email).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryUserResponseDto {
    pub id: i32,
    pub email: String,
    pub recovery_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDataResponseDto {
    pub nickname: String,
    pub email: String,
    pub registr_token: String,
}

// ** Section: "User Token" **

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserTokensDto {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenUserDto {
    // refreshToken
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let suffix = "@us".to_string();
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
        let suffix = "@".to_string();
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
}

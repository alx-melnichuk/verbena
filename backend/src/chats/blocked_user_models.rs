use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const BLOCKED_NICKNAME_MIN: u8 = 3;
pub const BLOCKED_NICKNAME_MAX: u8 = 64;
pub const MSG_BLOCKED_NICKNAME_MIN_LENGTH: &str = "blocked_nickname:min_length";
pub const MSG_BLOCKED_NICKNAME_MAX_LENGTH: &str = "blocked_nickname:max_length";
pub const MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT: &str = "blocked_oneOptionalMustPresent";

// MIN=3, MAX=64
pub fn validate_blocked_nickname(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, BLOCKED_NICKNAME_MIN.into(), MSG_BLOCKED_NICKNAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, BLOCKED_NICKNAME_MAX.into(), MSG_BLOCKED_NICKNAME_MAX_LENGTH)?;
    Ok(())
}

// * * * * Section: models for "BlockedUserOrm". * * * *

// ** Model: "BlockedUser". Used to return "blocked_user" data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::blocked_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockedUser {
    pub id: i32,
    pub user_id: i32,
    pub blocked_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "blocked_nickname")]
    pub blocked_nickname: String,
    pub block_date: DateTime<Utc>,
}

impl BlockedUser {
    pub fn new(
        id: i32,
        user_id: i32,
        blocked_id: i32,
        blocked_nickname: String,
        opt_block_date: Option<DateTime<Utc>>,
    ) -> BlockedUser {
        BlockedUser {
            id,
            user_id,
            blocked_id,
            blocked_nickname,
            block_date: opt_block_date.unwrap_or(Utc::now()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockedUserDto {
    pub id: i32,
    pub user_id: i32,
    pub blocked_id: i32,
    pub blocked_nickname: String,
    pub block_date: DateTime<Utc>,
}

impl From<BlockedUser> for BlockedUserDto {
    fn from(blocked_user: BlockedUser) -> Self {
        BlockedUserDto {
            id: blocked_user.id,
            user_id: blocked_user.user_id,
            blocked_id: blocked_user.blocked_id,
            blocked_nickname: blocked_user.blocked_nickname.clone(),
            block_date: blocked_user.block_date.clone(),
        }
    }
}

// ** Model: "CreateBlockedUser". Used: BlockedUserOrm::create_blocked_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateBlockedUser {
    pub user_id: i32,
    pub blocked_id: Option<i32>,
    pub blocked_nickname: Option<String>,
}

impl CreateBlockedUser {
    pub fn new(user_id: i32, blocked_id: Option<i32>, blocked_nickname: Option<String>) -> CreateBlockedUser {
        CreateBlockedUser {
            user_id,
            blocked_id,
            blocked_nickname,
        }
    }
}

impl Validator for CreateBlockedUser {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(nickname_val) = &self.blocked_nickname {
            errors.push(validate_blocked_nickname(nickname_val).err());
        }
        if self.blocked_nickname.is_none() && self.blocked_id.is_none() {
            let fields = "blocked_id, blocked_nickname";
            let msg = MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT;
            errors.push(ValidationChecks::one_optional_fields_must_present(&[false, false], fields, msg).err());
        }

        self.filter_errors(errors)
    }
}

// ** Model Dto: "CreateBlockedUserDto". Used: in "chat_controller::post_blocked_user()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBlockedUserDto {
    pub blocked_id: Option<i32>,
    pub blocked_nickname: Option<String>,
}

impl Validator for CreateBlockedUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(nickname_val) = &self.blocked_nickname {
            errors.push(validate_blocked_nickname(nickname_val).err());
        }
        if self.blocked_nickname.is_none() && self.blocked_id.is_none() {
            let fields = "blocked_id, blocked_nickname";
            let msg = MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT;
            errors.push(ValidationChecks::one_optional_fields_must_present(&[false, false], fields, msg).err());
        }

        self.filter_errors(errors)
    }
}

// ** Model: "DeleteBlockedUser". Used: BlockedUserOrm::delete_blocked_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeleteBlockedUser {
    pub user_id: i32,
    pub blocked_id: Option<i32>,
    pub blocked_nickname: Option<String>,
}

impl DeleteBlockedUser {
    pub fn new(user_id: i32, blocked_id: Option<i32>, blocked_nickname: Option<String>) -> DeleteBlockedUser {
        DeleteBlockedUser {
            user_id,
            blocked_id,
            blocked_nickname,
        }
    }
}

impl Validator for DeleteBlockedUser {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(nickname_val) = &self.blocked_nickname {
            errors.push(validate_blocked_nickname(nickname_val).err());
        }
        if self.blocked_nickname.is_none() && self.blocked_id.is_none() {
            let fields = "blocked_id, blocked_nickname";
            let msg = MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT;
            errors.push(ValidationChecks::one_optional_fields_must_present(&[false, false], fields, msg).err());
        }

        self.filter_errors(errors)
    }
}

// ** Model Dto: "DeleteBlockedUserDto". Used: in "chat_controller::delete_blocked_user()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBlockedUserDto {
    pub blocked_id: Option<i32>,
    pub blocked_nickname: Option<String>,
}

impl Validator for DeleteBlockedUserDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(nickname_val) = &self.blocked_nickname {
            errors.push(validate_blocked_nickname(nickname_val).err());
        }
        if self.blocked_nickname.is_none() && self.blocked_id.is_none() {
            let fields = "blocked_id, blocked_nickname";
            let msg = MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT;
            errors.push(ValidationChecks::one_optional_fields_must_present(&[false, false], fields, msg).err());
        }

        self.filter_errors(errors)
    }
}

// * * * *    * * * *

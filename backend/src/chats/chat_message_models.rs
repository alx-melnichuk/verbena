use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::{
    serial_datetime, serial_datetime_option,
    validators::{ValidationChecks, ValidationError, Validator},
};
use vrb_dbase::schema;

// ** Models: "CreateChatMessage", "ModifyChatMessage". **

pub const MESSAGE_MIN: u16 = 1;
pub const MSG_MESSAGE_MIN_LENGTH: &str = "message:min_length";
pub const MESSAGE_MAX: u16 = 255;
pub const MSG_MESSAGE_MAX_LENGTH: &str = "message:max_length";

// MIN=1, MAX=255
pub fn validate_message(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, MESSAGE_MIN.into(), MSG_MESSAGE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, MESSAGE_MAX.into(), MSG_MESSAGE_MAX_LENGTH)?;
    Ok(())
}

// ** Models: "CreateBlockedUser", "DeleteBlockedUser". **

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

// * * * * Section: models for "ChatMessageOrm". * * * *

// ** Model: "ChatMessage". Used to return "chat_message" data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::chat_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: i32,
    pub stream_id: i32,
    pub user_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    #[diesel(column_name = "user_name")]
    pub user_name: String,
    pub msg: Option<String>, // min_len=1 max_len=254 Nullable
    pub date_created: DateTime<Utc>,
    pub date_changed: Option<DateTime<Utc>>,
    pub date_removed: Option<DateTime<Utc>>,
}

impl ChatMessage {
    pub fn new(
        id: i32,
        stream_id: i32,
        user_id: i32,
        user_name: String,
        msg: Option<String>,
        date_created: DateTime<Utc>,
        date_changed: Option<DateTime<Utc>>,
        date_removed: Option<DateTime<Utc>>,
    ) -> ChatMessage {
        ChatMessage {
            id,
            stream_id,
            user_id,
            user_name,
            msg: msg,
            date_created,
            date_changed,
            date_removed,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageDto {
    pub id: i32,
    #[serde(with = "serial_datetime")]
    pub date: DateTime<Utc>,
    pub member: String,
    pub msg: String,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub date_edt: Option<DateTime<Utc>>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub date_rmv: Option<DateTime<Utc>>,
}

impl From<ChatMessage> for ChatMessageDto {
    fn from(chat_message: ChatMessage) -> Self {
        ChatMessageDto {
            id: chat_message.id,
            date: chat_message.date_created.clone(),
            member: chat_message.user_name.clone(),
            msg: chat_message.msg.unwrap_or("".to_owned()),
            date_edt: chat_message.date_changed.clone(),
            date_rmv: chat_message.date_removed.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::chat_message_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessageLog {
    pub id: i32,
    pub chat_message_id: i32,
    pub old_msg: String,
    pub date_update: DateTime<Utc>,
}

impl ChatMessageLog {
    pub fn new(id: i32, chat_message_id: i32, old_msg: &str, date_update: DateTime<Utc>) -> ChatMessageLog {
        ChatMessageLog {
            id,
            chat_message_id,
            old_msg: old_msg.to_string(),
            date_update,
        }
    }
}

// ** Model: "CreateChatMessage". Used: ChatMessageOrm::create_chat_message() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateChatMessage {
    pub stream_id: i32,
    pub user_id: i32,
    pub msg: String, // min_len=1 max_len=255 Nullable
}

impl CreateChatMessage {
    pub fn new(stream_id: i32, user_id: i32, msg: &str) -> CreateChatMessage {
        CreateChatMessage {
            stream_id,
            user_id,
            msg: msg.to_owned(),
        }
    }
}

impl Validator for CreateChatMessage {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_message(&self.msg).err());

        self.filter_errors(errors)
    }
}

// ** Model Dto: "CreateChatMessageDto". Used: in "chat_controller::post_chat_message()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatMessageDto {
    pub stream_id: i32,
    pub msg: String, // min_len=1 max_len=255 Nullable
}

impl Validator for CreateChatMessageDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_message(&self.msg).err());

        self.filter_errors(errors)
    }
}

// ** Model: "ModifyChatMessage". Used: ChatMessageOrm::modify_chat_message() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModifyChatMessage {
    pub msg: String, // min_len=1 max_len=255
}

impl ModifyChatMessage {
    pub fn new(msg: String) -> ModifyChatMessage {
        ModifyChatMessage { msg: msg.clone() }
    }
}

impl Validator for ModifyChatMessage {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if self.msg.len() > 0 {
            // If the string is empty, the DB will assign NULL.
            errors.push(validate_message(&self.msg).err());
        }

        self.filter_errors(errors)
    }
}

// ** Model Dto: "ModifyChatMessageDto". Used: in "chat_controller::put_chat_message()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyChatMessageDto {
    pub msg: String, // min_len=1 max_len=255
}

impl Validator for ModifyChatMessageDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if self.msg.len() > 0 {
            errors.push(validate_message(&self.msg).err());
        }

        self.filter_errors(errors)
    }
}

// ** Model: "SearchChatMessage". Used: ChatMessageOrm::filter_chat_messages() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchChatMessage {
    pub stream_id: i32,
    pub is_sort_des: Option<bool>,
    pub min_date_created: Option<DateTime<Utc>>,
    pub max_date_created: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl SearchChatMessage {
    pub fn new(
        stream_id: i32,
        is_sort_des: Option<bool>,
        min_date_created: Option<DateTime<Utc>>,
        max_date_created: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> SearchChatMessage {
        SearchChatMessage {
            stream_id,
            is_sort_des,
            min_date_created,
            max_date_created,
            limit,
        }
    }
}

impl SearchChatMessage {
    pub fn convert(search_chat_message: SearchChatMessageDto) -> Self {
        let mut limit: Option<usize> = None;
        if let Some(limit1) = search_chat_message.limit {
            limit = if limit1 >= 0 { Some(usize::try_from(limit1).unwrap()) } else { None }
        }
        SearchChatMessage {
            stream_id: search_chat_message.stream_id,
            is_sort_des: search_chat_message.is_sort_des.clone(),
            min_date_created: search_chat_message.min_date.clone(),
            max_date_created: search_chat_message.max_date.clone(),
            limit,
        }
    }
}

// * SearchChatMessageDto *

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchChatMessageDto {
    pub stream_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_sort_des: Option<bool>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub min_date: Option<DateTime<Utc>>,
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub max_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

// ** Model: "ChatAccess". Used: ChatMessageOrm::get_chat_access() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
pub struct ChatAccess {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "stream_id")]
    pub stream_id: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "stream_owner")]
    pub stream_owner: i32,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    #[diesel(column_name = "stream_live")]
    pub stream_live: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    #[diesel(column_name = "is_blocked")]
    pub is_blocked: bool,
}

impl ChatAccess {
    pub fn new(stream_id: i32, stream_owner: i32, stream_live: bool, is_blocked: bool) -> ChatAccess {
        ChatAccess {
            stream_id,
            stream_owner,
            stream_live,
            is_blocked,
        }
    }
}

// ** Model: "ChatStreamLive". Used: ChatMessageOrm::get_stream_live() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
pub struct ChatStreamLive {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "stream_id")]
    pub stream_id: i32,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    #[diesel(column_name = "stream_live")]
    pub stream_live: bool,
}

impl ChatStreamLive {
    pub fn new(stream_id: i32, stream_live: bool) -> ChatStreamLive {
        ChatStreamLive { stream_id, stream_live }
    }
}

// * * * *    * * * *

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
    pub fn new(id: i32, user_id: i32, blocked_id: i32, blocked_nickname: String, opt_block_date: Option<DateTime<Utc>>) -> BlockedUser {
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

#[cfg(all(test, feature = "mockdata"))]
pub struct ChatMessageTest {}

#[cfg(all(test, feature = "mockdata"))]
impl ChatMessageTest {
    pub fn message_min() -> String {
        (0..(MESSAGE_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn message_norm() -> String {
        (0..(MESSAGE_MIN + 1)).map(|_| 'a').collect()
    }
    pub fn message_max() -> String {
        (0..(MESSAGE_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn blocked_nickname_min() -> String {
        (0..(BLOCKED_NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn blocked_nickname_max() -> String {
        (0..(BLOCKED_NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
}

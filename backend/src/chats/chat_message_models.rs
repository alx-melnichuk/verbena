use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;
use crate::settings::err;
use crate::utils::serial_datetime;
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const MESSAGE_MIN: u8 = 1;
pub const MSG_MESSAGE_MIN_LENGTH: &str = "message:min_length";
pub const MESSAGE_MAX: u8 = 255;
pub const MSG_MESSAGE_MAX_LENGTH: &str = "message:max_length";

// MIN=1, MAX=255
pub fn validate_message(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, MESSAGE_MIN.into(), MSG_MESSAGE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, MESSAGE_MAX.into(), MSG_MESSAGE_MAX_LENGTH)?;
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
    pub date_update: DateTime<Utc>,
    pub is_changed: bool,
    pub is_removed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(
        id: i32,
        stream_id: i32,
        user_id: i32,
        user_name: String,
        msg: Option<String>,
        date_update: DateTime<Utc>,
        is_changed: bool,
        is_removed: bool,
    ) -> ChatMessage {
        let now = Utc::now();
        ChatMessage {
            id,
            stream_id,
            user_id,
            user_name,
            msg: msg,
            date_update,
            is_changed,
            is_removed,
            created_at: now.clone(),
            updated_at: now.clone(),
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
    pub is_edt: bool,
    pub is_rmv: bool,
}

impl ChatMessageDto {
    pub fn convert(chat_message: ChatMessage) -> Self {
        ChatMessageDto {
            id: chat_message.id,
            date: chat_message.date_update.clone(),
            member: chat_message.user_name.clone(),
            msg: chat_message.msg.unwrap_or("".to_owned()),
            is_edt: chat_message.is_changed,
            is_rmv: chat_message.is_removed,
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
    pub stream_id: Option<i32>,
    pub user_id: Option<i32>,
    pub msg: Option<String>, // min_len=1 max_len=255 Nullable
}

impl ModifyChatMessage {
    pub fn new(stream_id: Option<i32>, user_id: Option<i32>, msg: Option<String>) -> ModifyChatMessage {
        ModifyChatMessage {
            stream_id,
            user_id,
            msg: msg.clone(),
        }
    }
    pub fn valid_names<'a>() -> Vec<&'a str> {
        vec!["stream_id", "user_id", "msg"]
    }
}

impl Validator for ModifyChatMessage {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.msg {
            if value.len() > 0 {
                // If the string is empty, the DB will assign NULL.
                errors.push(validate_message(&value).err());
            }
        }

        let list_is_some = vec![self.stream_id.is_some(), self.user_id.is_some(), self.msg.is_some()];
        let valid_names = Self::valid_names().join(",");
        errors.push(
            ValidationChecks::no_fields_to_update(&list_is_some, &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err(),
        );

        self.filter_errors(errors)
    }
}

// ** Model Dto: "ModifyChatMessageDto". Used: in "chat_controller::put_chat_message()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyChatMessageDto {
    pub stream_id: Option<i32>,
    pub user_id: Option<i32>,
    pub msg: Option<String>, // min_len=1 max_len=255 Nullable
}

impl ModifyChatMessageDto {
    pub fn valid_names<'a>() -> Vec<&'a str> {
        vec!["stream_id", "user_id", "msg"]
    }
}

impl Validator for ModifyChatMessageDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.msg {
            errors.push(validate_message(&value).err());
        }

        // Checking for at least one required field.
        let list_is_some = vec![self.stream_id.is_some(), self.user_id.is_some(), self.msg.is_some()];
        let valid_names = ModifyChatMessageDto::valid_names().join(",");
        errors.push(
            ValidationChecks::no_fields_to_update(&list_is_some, &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err(),
        );

        self.filter_errors(errors)
    }
}

// ** Model: "FilterChatMessage". Used: ChatMessageOrm::filter_chat_messages() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FilterChatMessage {
    pub stream_id: i32,
    pub is_sort_des: Option<bool>,
    pub border_by_id: Option<i32>,
    pub limit: Option<i32>,
}

impl FilterChatMessage {
    pub fn new(
        stream_id: i32,
        is_sort_des: Option<bool>,
        border_by_id: Option<i32>,
        limit: Option<i32>,
    ) -> FilterChatMessage {
        FilterChatMessage {
            stream_id,
            is_sort_des,
            border_by_id,
            limit,
        }
    }
}

impl FilterChatMessage {
    pub fn convert(filter_chat_message: FilterChatMessageDto) -> Self {
        FilterChatMessage {
            stream_id: filter_chat_message.stream_id,
            is_sort_des: filter_chat_message.is_sort_des.clone(),
            border_by_id: filter_chat_message.border_by_id.clone(),
            limit: filter_chat_message.limit.clone(),
        }
    }
}

// * FilterChatMessageDto *

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilterChatMessageDto {
    pub stream_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_sort_des: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_by_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

// * * * *    * * * *

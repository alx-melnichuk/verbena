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

// * * * * Section: "database". * * * *

// * * * * Section: models for "ChatMessageOrm". * * * *

// ** Model: "ChatMessage". Used to return "chat_message" data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::chat_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: i32,
    pub stream_id: i32,
    pub user_id: i32,
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
    pub stream_id: i32,
    pub user_id: i32,
    pub msg: String, // min_len=1 max_len=254 Nullable
    #[serde(with = "serial_datetime")]
    pub date_update: DateTime<Utc>,
    pub is_changed: bool,
    pub is_removed: bool,
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
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
}

impl ModifyChatMessage {
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

// ** Model: "FilterChatMessage". Used: ChatMessageOrm::filter_chat_messages() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FilterChatMessage {
    pub id: Option<i32>,
    pub stream_id: Option<i32>,
    pub user_id: Option<i32>,
    pub is_sort_des: Option<bool>,
    pub id_more: Option<i32>,
    pub id_less: Option<i32>,
    pub limit: Option<i32>,
}

impl FilterChatMessage {
    pub fn new(id: Option<i32>, stream_id: Option<i32>, user_id: Option<i32>) -> FilterChatMessage {
        FilterChatMessage {
            id,
            stream_id,
            user_id,
            is_sort_des: None,
            id_more: None,
            id_less: None,
            limit: None,
        }
    }
}

impl FilterChatMessage {
    pub fn convert(filter_chat_message: FilterChatMessageDto) -> Self {
        FilterChatMessage {
            id: filter_chat_message.id.clone(),
            stream_id: filter_chat_message.stream_id.clone(),
            user_id: filter_chat_message.user_id.clone(),
            is_sort_des: filter_chat_message.is_sort_des.clone(),
            id_more: filter_chat_message.id_more.clone(),
            id_less: filter_chat_message.id_less.clone(),
            limit: filter_chat_message.limit.clone(),
        }
    }
}

// * FilterChatMessageDto *

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilterChatMessageDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_sort_des: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_more: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_less: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl FilterChatMessageDto {
    pub fn optional_fields<'a>() -> Vec<&'a str> {
        vec!["id", "stream_id", "user_id"]
    }
}

impl Validator for FilterChatMessageDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        let list_is_some = vec![self.id.is_some(), self.stream_id.is_some(), self.user_id.is_some()];
        let fields = Self::optional_fields().join(",");
        errors.push(
            // Checking, one of the optional fields must be present.
            ValidationChecks::one_optional_fields_must_present(
                &list_is_some,
                &fields,
                err::MSG_ONE_OPTIONAL_FIELDS_MUST_PRESENT,
            )
            .err(),
        );

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

// * * * *    * * * *

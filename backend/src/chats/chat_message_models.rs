use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
//use utoipa::ToSchema;

use crate::schema;

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
    pub msg: String, // min_len=1 max_len=254
}

impl CreateChatMessage {
    pub fn new(stream_id: i32, user_id: i32, msg: &str) -> CreateChatMessage {
        CreateChatMessage {
            stream_id,
            user_id,
            msg: msg.to_string(),
        }
    }
}

// ** Model: "ModifyChatMessage". Used: ChatMessageOrm::modify_chat_message() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModifyChatMessage {
    pub stream_id: Option<i32>,
    pub user_id: Option<i32>,
    pub msg: Option<String>, // min_len=1,max_len=254
}

impl ModifyChatMessage {
    pub fn new(stream_id: Option<i32>, user_id: Option<i32>, msg: Option<String>) -> ModifyChatMessage {
        ModifyChatMessage {
            stream_id,
            user_id,
            msg,
        }
    }
}

// * * * *    * * * *

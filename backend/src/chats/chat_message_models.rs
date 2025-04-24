use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema;

// * * * * Section: "database". * * * *

/*diesel::table! {
    chat_messages (id) {
        id -> Int4,
        stream_id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        msg -> Varchar,
        date_created -> Timestamptz,
        date_changed -> Nullable<Timestamptz>,
        date_removed -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}*/

// * * * * Section: models for "ChatMessageOrm". * * * *

// ** Model: "ChatMessage". Used to return "chat_message" data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::chat_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
//#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i32,
    pub stream_id: i32,
    pub user_id: i32,
    pub msg: String,
    pub date_created: DateTime<Utc>,
    pub date_changed: Option<DateTime<Utc>>,
    pub date_removed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(
        id: i32,
        stream_id: i32,
        user_id: i32,
        msg: &str,
        date_created: DateTime<Utc>,
        date_changed: Option<DateTime<Utc>>,
        date_removed: Option<DateTime<Utc>>,
    ) -> ChatMessage {
        let now = Utc::now();
        ChatMessage {
            id,
            stream_id,
            user_id,
            msg: msg.to_string(),
            date_created,
            date_changed,
            date_removed,
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

/*diesel::table! {
    chat_message_logs (id) {
        id -> Int4,
        chat_message_id -> Int4,
        #[max_length = 255]
        old_msg -> Varchar,
        #[max_length = 255]
        new_msg -> Varchar,
        date_changed -> Timestamptz,
    }
}*/

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

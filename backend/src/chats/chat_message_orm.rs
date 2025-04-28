use crate::chats::chat_message_models::{ChatMessage, ChatMessageLog, CreateChatMessage, ModifyChatMessage};

pub trait ChatMessageOrm {
    /// Get a list of "chat_message_log" for the specified "chat_message_id".
    fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String>;

    /// Add a new entry (chat_message).
    fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String>;

    /// Modify an entity (chat_message).
    fn modify_chat_message(
        &self,
        id: i32,
        modify_chat_message: ModifyChatMessage,
    ) -> Result<Option<ChatMessage>, String>;

    /// Delete an entity (chat_message).
    fn delete_chat_message(&self, id: i32) -> Result<Option<ChatMessage>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(all(test, feature = "mockdata")))]
    use super::impls::ChatMessageOrmApp;
    #[cfg(not(all(test, feature = "mockdata")))]
    pub fn get_chat_message_orm_app(pool: DbPool) -> ChatMessageOrmApp {
        ChatMessageOrmApp::new(pool)
    }

    #[cfg(all(test, feature = "mockdata"))]
    use super::tests::ChatMessageOrmApp;
    #[cfg(all(test, feature = "mockdata"))]
    pub fn get_chat_message_orm_app(_: DbPool) -> ChatMessageOrmApp {
        ChatMessageOrmApp::new()
    }
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {

    use diesel::{self, prelude::*, sql_types};

    use crate::dbase;
    // use crate::schema;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct ChatMessageOrmApp {
        pub pool: dbase::DbPool,
    }

    impl ChatMessageOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            ChatMessageOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl ChatMessageOrm for ChatMessageOrmApp {
        /// Get a list of "chat_message_log" for the specified "chat_message_id".
        fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_chat_message_log($1);")
                .bind::<sql_types::Integer, _>(chat_message_id);

            let list: Vec<ChatMessageLog> = query
                .load(&mut conn)
                .map_err(|e| format!("get_chat_message_log: {}", e.to_string()))?;

            Ok(list)
        }

        /// Add a new entry (chat_message).
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_chat_message($1,$2,$3);")
                .bind::<sql_types::Integer, _>(create_chat_message.stream_id) // $1
                .bind::<sql_types::Integer, _>(create_chat_message.user_id) // $2
                .bind::<sql_types::Text, _>(create_chat_message.msg); // $3

            // Run a query with Diesel to create a new user and return it.
            let chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .map_err(|e| format!("create_chat_message: {}", e.to_string()))?;

            Ok(chat_message)
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            modify_chat_message: ModifyChatMessage,
        ) -> Result<Option<ChatMessage>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from modify_chat_message($1,$2,$3,$4);")
                .bind::<sql_types::Integer, _>(id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(modify_chat_message.stream_id) // $2
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(modify_chat_message.user_id) // $3
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_chat_message.msg); // $4

            // TODO ??
            // Run a query with Diesel to create a new user and return it.
            let chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_chat_message: {}", e.to_string()))?;

            Ok(chat_message)
        }

        /// Delete an entity (chat_message).
        fn delete_chat_message(&self, id: i32) -> Result<Option<ChatMessage>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_chat_message($1);").bind::<sql_types::Integer, _>(id);

            // Run a query using Diesel to delete the "chat_message" entity by ID and return the data for that entity.
            let opt_chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_chat_message: {}", e.to_string()))?;

            Ok(opt_chat_message)
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use std::collections::HashMap;

    use chrono::Utc;

    use super::*;

    pub const CHAT_MESSAGE_ID: i32 = 1500;
    pub const CHAT_MESSAGE_LOG_ID: i32 = 1600;

    #[derive(Debug, Clone)]
    pub struct ChatMessageOrmApp {
        pub chat_message_vec: Vec<ChatMessage>,
        pub chat_message_log_map: HashMap<i32, Vec<ChatMessageLog>>,
    }

    impl ChatMessageOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ChatMessageOrmApp {
                chat_message_vec: Vec::new(),
                chat_message_log_map: HashMap::new(),
            }
        }
        /// Create a new instance with the specified ChatMessage list.
        pub fn create(chat_message_list: &[ChatMessage], chat_message_log_list: &[ChatMessageLog]) -> Self {
            let mut chat_message_vec: Vec<ChatMessage> = Vec::new();
            let mut chat_message_log_map: HashMap<i32, Vec<ChatMessageLog>> = HashMap::new();

            let mut tmp_ch_msg_lg_map: HashMap<i32, Vec<ChatMessageLog>> = HashMap::new();

            for chat_msg_log in chat_message_log_list.iter() {
                // Search for the value "chat_message_id" in the directory.
                if let Some(chat_msg_log_list) = tmp_ch_msg_lg_map.get_mut(&chat_msg_log.chat_message_id) {
                    (*chat_msg_log_list).push(chat_msg_log.clone());
                } else {
                    // If the value is not in the directory, then create a new vector.
                    let mut chat_msg_log_list = Vec::<ChatMessageLog>::new();
                    chat_msg_log_list.push(chat_msg_log.clone());
                    tmp_ch_msg_lg_map.insert(chat_msg_log.chat_message_id, chat_msg_log_list);
                }
            }
            let mut log_id: i32 = CHAT_MESSAGE_LOG_ID;

            for (idx, chat_message) in chat_message_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let id = CHAT_MESSAGE_ID + delta;
                let new_chat_message = ChatMessage::new(
                    id,
                    chat_message.stream_id,
                    chat_message.user_id,
                    chat_message.msg.clone(),
                    chat_message.date_update.clone(),
                    chat_message.is_changed,
                    chat_message.is_removed,
                );
                chat_message_vec.push(new_chat_message);

                if chat_message.is_changed {
                    if let Some(ch_msg_lg_vec) = tmp_ch_msg_lg_map.get(&chat_message.id) {
                        let mut ch_msg_log_list = Vec::<ChatMessageLog>::new();

                        for ch_msg_lg in ch_msg_lg_vec.iter() {
                            log_id += 1;
                            let new_ch_msg_log = ChatMessageLog {
                                id: log_id,
                                chat_message_id: id,
                                old_msg: ch_msg_lg.old_msg.clone(),
                                date_update: ch_msg_lg.date_update.clone(),
                            };
                            ch_msg_log_list.push(new_ch_msg_log);
                        }

                        chat_message_log_map.insert(id, ch_msg_log_list);
                    }
                }
            }
            ChatMessageOrmApp {
                chat_message_vec,
                chat_message_log_map,
            }
        }
    }

    impl ChatMessageOrm for ChatMessageOrmApp {
        /// Get a list of "chat_message_log" for the specified "chat_message_id".
        fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String> {
            let opt_chat_message = self.chat_message_vec.iter().find(|chat_msg| (*chat_msg).id == chat_message_id);
            if let Some(chat_message) = opt_chat_message {}
            Ok(vec![])
        }

        /// Add a new entry (chat_message).
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String> {
            let idx: i32 = self.chat_message_vec.len().try_into().unwrap();
            let chat_message_id: i32 = CHAT_MESSAGE_ID + idx;

            let chat_message = ChatMessage::new(
                chat_message_id,
                create_chat_message.stream_id,
                create_chat_message.user_id,
                Some(create_chat_message.msg.clone()),
                Utc::now(),
                false,
                false,
            );

            Ok(chat_message)
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            modify_chat_message: ModifyChatMessage,
        ) -> Result<Option<ChatMessage>, String> {
            let opt_chat_message = self.chat_message_vec.iter().find(|chat_msg| (*chat_msg).id == id);
            let opt_chat_message3: Option<ChatMessage> = if let Some(chat_message) = opt_chat_message {
                #[rustfmt::skip]
                let msg_len: i8 = match modify_chat_message.msg {
                    Some(ref val) => if val.len() > 0 { 1 } else { 0 },
                    None => -1,
                };
                let chat_message2 = ChatMessage {
                    id: chat_message.id,
                    stream_id: modify_chat_message.stream_id.unwrap_or(chat_message.stream_id),
                    user_id: modify_chat_message.user_id.unwrap_or(chat_message.user_id),
                    msg: modify_chat_message.msg.clone(),
                    date_update: Utc::now(),
                    is_changed: if msg_len > 0 { true } else { chat_message.is_changed },
                    is_removed: if msg_len == 0 { true } else { chat_message.is_removed },
                    created_at: chat_message.created_at,
                    updated_at: Utc::now(),
                };
                Some(chat_message2)
            } else {
                None
            };
            Ok(opt_chat_message3)
        }

        /// Delete an entity (chat_message).
        fn delete_chat_message(&self, id: i32) -> Result<Option<ChatMessage>, String> {
            let opt_chat_message = self.chat_message_vec.iter().find(|chat_msg| (*chat_msg).id == id);
            Ok(opt_chat_message.map(|u| u.clone()))
        }
    }
}

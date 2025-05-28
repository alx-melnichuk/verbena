use crate::chats::chat_message_models::{
    ChatMessage, ChatMessageLog, CreateChatMessage, FilterChatMessage, ModifyChatMessage,
};

pub trait ChatMessageOrm {
    /// Get a list of "chat_message_log" for the specified "chat_message_id".
    fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String>;

    /// Filter entities (chat_messages) by specified parameters.
    fn filter_chat_messages(&self, filter_chat_message: FilterChatMessage) -> Result<Vec<ChatMessage>, String>;

    /// Add a new entry (chat_message).
    fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String>;

    /// Modify an entity (chat_message).
    fn modify_chat_message(
        &self,
        id: i32,
        opt_by_user_id: Option<i32>,
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
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};

    use crate::chats::{
        chat_message_models::{ChatMessage, ChatMessageLog, CreateChatMessage, FilterChatMessage, ModifyChatMessage},
        chat_message_orm::ChatMessageOrm,
    };
    use crate::dbase;
    use crate::validators::Validator;

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
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_chat_message_log($1);")
                .bind::<sql_types::Integer, _>(chat_message_id);

            let list: Vec<ChatMessageLog> = query
                .load(&mut conn)
                .map_err(|e| format!("get_chat_message_log: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_chat_message_logs() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(list)
        }

        /// Filter entities (chat_messages) by specified parameters.
        fn filter_chat_messages(&self, flt_chat_msg: FilterChatMessage) -> Result<Vec<ChatMessage>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from filter_chat_messages($1,$2,$3,$4);")
                .bind::<sql_types::Integer, _>(flt_chat_msg.stream_id) //$1
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(flt_chat_msg.is_sort_des) // $2
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(flt_chat_msg.border_by_id) // $3
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(flt_chat_msg.limit); // $4

            // Run a query using Diesel to find a list of entities (ChatMessage) based on the given parameters.
            let chat_messages: Vec<ChatMessage> = query
                //.returning(ChatMessage::as_returning())
                //.get_results::<ChatMessage>(&mut conn)
                .load(&mut conn)
                .map_err(|e| format!("filter_chat_messages: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("filter_chat_messages1() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(chat_messages)
        }

        /// Add a new entry (chat_message).
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let validation_res = create_chat_message.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

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

            if let Some(timer) = timer {
                info!("create_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(chat_message)
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            opt_by_user_id: Option<i32>,
            modify_chat_message: ModifyChatMessage,
        ) -> Result<Option<ChatMessage>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let validation_res = modify_chat_message.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from modify_chat_message($1,$2,$3,$4,$5);")
                .bind::<sql_types::Integer, _>(id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(opt_by_user_id) // $2
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(modify_chat_message.stream_id) // $3
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(modify_chat_message.user_id) // $4
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_chat_message.msg); // $5

            // Run a query with Diesel to modify the entity and return it.
            let chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_chat_message: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(chat_message)
        }

        /// Delete an entity (chat_message).
        fn delete_chat_message(&self, id: i32) -> Result<Option<ChatMessage>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_chat_message($1);").bind::<sql_types::Integer, _>(id);

            // Run a query using Diesel to delete the entity by ID and return it.
            let opt_chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_chat_message: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
            }
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
                    chat_message.user_name.clone(),
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
            if let Some(_chat_message) = opt_chat_message {}
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
                "user_name".to_owned(),
                Some(create_chat_message.msg.clone()),
                Utc::now(),
                false,
                false,
            );

            Ok(chat_message)
        }

        /// Filter entities (chat_messages) by specified parameters.
        fn filter_chat_messages(&self, _flt_chat_msg: FilterChatMessage) -> Result<Vec<ChatMessage>, String> {
            Ok(vec![])
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            opt_by_user_id: Option<i32>,
            modify_chat_message: ModifyChatMessage,
        ) -> Result<Option<ChatMessage>, String> {
            let opt_chat_message = self
                .chat_message_vec
                .iter()
                .find(|chat_msg| {
                    let check_user_id = if let Some(by_user_id) = opt_by_user_id {
                        (*chat_msg).user_id == by_user_id
                    } else {
                        true
                    };
                    (*chat_msg).id == id && check_user_id
                })
                .map(|chat_msg| chat_msg.clone());

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
                    user_name: "user_name".to_owned(),
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

use vrb_dbase::dbase::DbPool;

use crate::chat_message_models::{
    BlockedUser, ChatAccess, ChatMessage, ChatMessageLog, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser, ModifyChatMessage,
    SearchChatMessage,
};

pub trait ChatMessageOrm {
    /// Get a list of "chat_message_log" for the specified "chat_message_id".
    fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String>;

    /// Filter entities (chat_messages) by specified parameters.
    fn filter_chat_messages(&self, search_chat_message: SearchChatMessage) -> Result<Vec<ChatMessage>, String>;

    /// Add a new entry (chat_message).
    fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<Option<ChatMessage>, String>;

    /// Modify an entity (chat_message).
    fn modify_chat_message(&self, id: i32, user_id: i32, modify_chat_message: ModifyChatMessage) -> Result<Option<ChatMessage>, String>;

    /// Delete an entity (chat_message).
    fn delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, String>;

    /// Get information about the live of the stream.
    fn get_stream_live(&self, stream_id: i32) -> Result<Option<bool>, String>;

    /// Get chat access information. (ChatAccess)
    fn get_chat_access(&self, stream_id: i32, user_id: i32) -> Result<Option<ChatAccess>, String>;

    /// Get a list of blocked users.
    fn get_blocked_users(&self, user_id: i32) -> Result<Vec<BlockedUser>, String>;

    /// Add a new entry (blocked_user).
    fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String>;

    /// Delete an entity (blocked_user).
    fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String>;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_chat_message_orm_app(pool: DbPool) -> impls::ChatMessageOrmApp {
    impls::ChatMessageOrmApp::new(pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_chat_message_orm_app(_: DbPool) -> tests::ChatMessageOrmApp {
    tests::ChatMessageOrmApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};
    use vrb_common::validators::Validator;
    use vrb_dbase::dbase;

    use crate::{
        chat_message_models::{
            BlockedUser, ChatAccess, ChatMessage, ChatMessageLog, ChatStreamLive, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser,
            ModifyChatMessage, SearchChatMessage,
        },
        chat_message_orm::ChatMessageOrm,
    };

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

            let query = diesel::sql_query("select * from get_chat_message_log($1);").bind::<sql_types::Integer, _>(chat_message_id);

            let list: Vec<ChatMessageLog> = query.load(&mut conn).map_err(|e| format!("get_chat_message_log: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_chat_message_logs() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(list)
        }

        /// Filter entities (chat_messages) by specified parameters.
        fn filter_chat_messages(&self, flt_chat_msg: SearchChatMessage) -> Result<Vec<ChatMessage>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let opt_limit = flt_chat_msg.limit.map(|v| i32::try_from(v).unwrap());

            let query = diesel::sql_query("select * from filter_chat_messages($1,$2,$3,$4,$5);")
                .bind::<sql_types::Integer, _>(flt_chat_msg.stream_id) //$1
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(flt_chat_msg.is_sort_des) // $2
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(flt_chat_msg.min_date_created) // $3
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(flt_chat_msg.max_date_created) // $4
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(opt_limit); // $5

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
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<Option<ChatMessage>, String> {
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
            let opt_chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("create_chat_message: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_chat_message)
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            user_id: i32,
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

            let query = diesel::sql_query("select * from modify_chat_message($1,$2,$3);")
                .bind::<sql_types::Integer, _>(id) // $1
                .bind::<sql_types::Integer, _>(user_id) // $2
                .bind::<sql_types::Text, _>(modify_chat_message.msg); // $3

            // Run a query with Diesel to modify the entity and return it.
            let opt_chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_chat_message: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_chat_message)
        }

        /// Delete an entity (chat_message).
        fn delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_chat_message($1,$2);")
                .bind::<sql_types::Integer, _>(id)
                .bind::<sql_types::Integer, _>(user_id); // $2

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

        /// Get information about the live of the stream.
        fn get_stream_live(&self, stream_id: i32) -> Result<Option<bool>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            #[rustfmt::skip]
            let query =
                diesel::sql_query("select * from get_stream_live($1);").bind::<sql_types::Integer, _>(stream_id);

            let opt_chat_stream_live = query
                .get_result::<ChatStreamLive>(&mut conn)
                .optional()
                .map_err(|e| format!("get_stream_live: {}", e.to_string()))?;

            let opt_stream_live = opt_chat_stream_live.map(|v| v.stream_live.clone());

            if let Some(timer) = timer {
                info!("get_stream_live() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_live)
        }

        /// Get chat access information. (ChatAccess)
        fn get_chat_access(&self, stream_id: i32, user_id: i32) -> Result<Option<ChatAccess>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_chat_access($1,$2);")
                .bind::<sql_types::Integer, _>(stream_id)
                .bind::<sql_types::Integer, _>(user_id);

            let opt_chat_access = query
                .get_result::<ChatAccess>(&mut conn)
                .optional()
                .map_err(|e| format!("get_chat_access: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_chat_access() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_chat_access)
        }

        /// Get a list of blocked users.
        fn get_blocked_users(&self, user_id: i32) -> Result<Vec<BlockedUser>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            #[rustfmt::skip]
            let query = diesel::sql_query("select * from get_blocked_users($1);")
                .bind::<sql_types::Integer, _>(user_id); // $1

            // Run a query with Diesel to create a new user and return it.
            #[rustfmt::skip]
            let blocked_user_list: Vec<BlockedUser> = query
                .load(&mut conn)
                .map_err(|e| format!("get_blocked_users: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_blocked_users() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(blocked_user_list)
        }

        /// Add a new entry (blocked_user).
        fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let validation_res = create_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_blocked_user($1,$2,$3);")
                .bind::<sql_types::Integer, _>(create_blocked_user.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(create_blocked_user.blocked_id) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_blocked_user.blocked_nickname); // $3

            // Run a query with Diesel to create a new user and return it.
            let blocked_user = query
                .get_result::<BlockedUser>(&mut conn)
                .optional()
                .map_err(|e| format!("create_blocked_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(blocked_user)
        }

        /// Delete an entity (blocked_user).
        fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let validation_res = delete_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_blocked_user($1,$2,$3);")
                .bind::<sql_types::Integer, _>(delete_blocked_user.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(delete_blocked_user.blocked_id) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(delete_blocked_user.blocked_nickname); // $3

            // Run a query with Diesel to delete the entity and return it.
            let blocked_user = query
                .get_result::<BlockedUser>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_blocked_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(blocked_user)
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use std::{cell::RefCell, cmp::Ordering, collections::HashMap};

    use actix_web::web;
    use chrono::{DateTime, Duration, SubsecRound, Timelike, Utc};
    use vrb_common::{
        consts_user_test::{USER1_ID, USER1_NAME, USER2_ID, USER2_NAME, USER3_ID, USER3_NAME, USER4_ID, USER4_NAME},
        validators::Validator
    };

    use crate::{
        chat_message_models::{
            BlockedUser, ChatAccess, ChatMessage, ChatMessageLog, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser,
            ModifyChatMessage, SearchChatMessage,
        },
        chat_message_orm::ChatMessageOrm,
    };

    pub const CH_MSG_STREAM_ID: i32 = 1;
    pub const CHAT_MESSAGE_ID: i32 = 1500;
    pub const CHAT_MESSAGE_LOG_ID: i32 = 1600;
    pub const BLOCKED_USER_ID: i32 = 1700;

    pub const STREAM1_ID: i32 = 1; // Owner user idx 0 (live: true)  1100 oliver_taylor
    pub const STREAM2_ID: i32 = 2; // Owner user idx 1 (live: true)  1101 robert_brown
    pub const STREAM3_ID: i32 = 3; // Owner user idx 2 (live: false) 1102 mary_williams
    pub const STREAM4_ID: i32 = 4; // Owner user idx 3  blocked      1103 ava_wilson

    #[derive(Debug, Clone)]
    pub struct UserMini {
        pub id: i32,
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct ChatMessageOrmApp {
        pub chat_message_vec: Vec<ChatMessage>,
        pub chat_message_log_map: HashMap<i32, Vec<ChatMessageLog>>,
        pub blocked_user_vec: Box<RefCell<Vec<BlockedUser>>>,
        pub user_vec: Vec<UserMini>,
    }

    fn round_subsecs(d1: DateTime<Utc>) -> DateTime<Utc> {
        d1.with_nanosecond(d1.round_subsecs(3).nanosecond()).unwrap()
    }

    impl ChatMessageOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ChatMessageOrmApp {
                chat_message_vec: Vec::new(),
                chat_message_log_map: HashMap::new(),
                blocked_user_vec: Box::new(RefCell::new(Vec::new())),
                user_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified ChatMessage list.
        pub fn create(
            chat_message_list: &[ChatMessage],
            chat_message_log_list: &[ChatMessageLog],
            blocked_user_list: &[BlockedUser],
            users_list: &[UserMini],
        ) -> Self {
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
                    round_subsecs(chat_message.date_created.clone()),
                    chat_message.date_changed.clone().map(|d| round_subsecs(d)),
                    chat_message.date_removed.clone().map(|d| round_subsecs(d)),
                );
                chat_message_vec.push(new_chat_message);

                if chat_message.date_changed.is_some() {
                    if let Some(ch_msg_lg_vec) = tmp_ch_msg_lg_map.get(&chat_message.id) {
                        let mut ch_msg_log_list = Vec::<ChatMessageLog>::new();

                        for ch_msg_lg in ch_msg_lg_vec.iter() {
                            log_id += 1;
                            let new_ch_msg_log = ChatMessageLog {
                                id: log_id,
                                chat_message_id: id,
                                old_msg: ch_msg_lg.old_msg.clone(),
                                date_update: round_subsecs(ch_msg_lg.date_update.clone()),
                            };
                            ch_msg_log_list.push(new_ch_msg_log);
                        }

                        chat_message_log_map.insert(id, ch_msg_log_list);
                    }
                }
            }

            let mut blocked_user_vec: Vec<BlockedUser> = Vec::new();
            for (idx, blocked_user) in blocked_user_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let new_blocked_user = BlockedUser::new(
                    BLOCKED_USER_ID + delta,
                    blocked_user.user_id,
                    blocked_user.blocked_id,
                    blocked_user.blocked_nickname.clone(),
                    Some(round_subsecs(blocked_user.block_date.clone())),
                );
                blocked_user_vec.push(new_blocked_user);
            }
            let user_vec: Vec<UserMini> = Vec::from(users_list);

            ChatMessageOrmApp {
                chat_message_vec,
                chat_message_log_map,
                blocked_user_vec: Box::new(RefCell::new(blocked_user_vec)),
                user_vec,
            }
        }
        #[rustfmt::skip]
        pub fn is_stream_id_exists(&self, opt_stream_id: Option<i32>) -> bool {
            if let Some(stream_id) = opt_stream_id { ChatMessageOrmTest::stream_ids().contains(&stream_id) } else { true }
        }
        pub fn find_user_by_id(&self, id: i32) -> Option<UserMini> {
            self.user_vec.iter().find(|v| v.id == id).map(|v| v.clone())
        }
        pub fn find_user_by_name(&self, name: &str) -> Option<UserMini> {
            self.user_vec.iter().find(|v| v.name == name).map(|v| v.clone())
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
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<Option<ChatMessage>, String> {
            let is_stream_id_exists = self.is_stream_id_exists(Some(create_chat_message.stream_id));
            let is_user_id_exists = ChatMessageOrmTest::user_ids().contains(&create_chat_message.user_id);

            if create_chat_message.msg.len() == 0 || !is_stream_id_exists || !is_user_id_exists {
                return Ok(None);
            }

            let idx: i32 = self.chat_message_vec.len().try_into().unwrap();
            let chat_message_id: i32 = CHAT_MESSAGE_ID + idx;
            let user_name = ChatMessageOrmTest::get_user_name(create_chat_message.user_id).clone();

            let chat_message = ChatMessage::new(
                chat_message_id,
                create_chat_message.stream_id,
                create_chat_message.user_id,
                user_name,
                Some(create_chat_message.msg.clone()),
                Utc::now(),
                None,
                None,
            );

            Ok(Some(chat_message))
        }

        /// Filter entities (chat_messages) by specified parameters.
        fn filter_chat_messages(&self, flt_chat_msg: SearchChatMessage) -> Result<Vec<ChatMessage>, String> {
            let stream_id: i32 = flt_chat_msg.stream_id;
            let opt_min_date_created: Option<DateTime<Utc>> = flt_chat_msg.min_date_created;
            let opt_max_date_created: Option<DateTime<Utc>> = flt_chat_msg.max_date_created;
            let mut list: Vec<ChatMessage> = Vec::new();

            for ch_msg in self.chat_message_vec.iter() {
                let mut is_add_value = true;

                if ch_msg.stream_id != stream_id {
                    is_add_value = false;
                }
                if is_add_value {
                    if let Some(min_date_created) = opt_min_date_created {
                        is_add_value = min_date_created < ch_msg.date_created;
                    }
                }
                if is_add_value {
                    if let Some(max_date_created) = opt_max_date_created {
                        is_add_value = ch_msg.date_created < max_date_created;
                    }
                }

                if is_add_value {
                    list.push(ch_msg.clone());
                }
            }

            let is_sort_des: bool = flt_chat_msg.is_sort_des.unwrap_or(false);
            #[rustfmt::skip]
            list.sort_by(|a, b| {
                let res = if a.date_created < b.date_created {
                    Ordering::Less
                } else if a.date_created > b.date_created {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                };
                if is_sort_des { res.reverse() } else { res }
            });

            let limit = flt_chat_msg.limit.unwrap_or(20);

            let mid = if limit < list.len() { limit } else { list.len() };
            let (left, _right) = list.split_at(mid);
            let result = left.iter().map(|chat_msg| chat_msg.clone()).collect();

            Ok(result)
        }

        /// Modify an entity (chat_message).
        fn modify_chat_message(
            &self,
            id: i32,
            user_id: i32,
            modify_chat_message: ModifyChatMessage,
        ) -> Result<Option<ChatMessage>, String> {
            let opt_chat_message = self
                .chat_message_vec
                .iter()
                .find(|chat_msg| (*chat_msg).id == id && (*chat_msg).user_id == user_id)
                .map(|chat_msg| chat_msg.clone());

            if opt_chat_message.is_none() {
                return Ok(None);
            }
            let chat_message = opt_chat_message.unwrap();

            let mut chat_message1 = chat_message.clone();

            let date = Utc::now();
            if modify_chat_message.msg.len() > 0 {
                chat_message1.date_changed = Some(date);
            } else {
                chat_message1.date_removed = Some(date);
            }
            chat_message1.msg = Some(modify_chat_message.msg.clone());

            Ok(Some(chat_message1))
        }

        /// Delete an entity (chat_message).
        fn delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, String> {
            let opt_chat_message = self
                .chat_message_vec
                .iter()
                .find(|chat_msg| (*chat_msg).id == id && (*chat_msg).user_id == user_id)
                .map(|chat_msg| chat_msg.clone());

            Ok(opt_chat_message)
        }

        /// Get information about the live of the stream.
        fn get_stream_live(&self, stream_id: i32) -> Result<Option<bool>, String> {
            let idx_stream_id = ChatMessageOrmTest::stream_ids().iter().position(|v| *v == stream_id);
            if idx_stream_id.is_none() {
                return Ok(None);
            }
            let stream_live = stream_id != ChatMessageOrmTest::stream_ids().get(2).unwrap().clone();

            Ok(Some(stream_live))
        }

        /// Get chat access information. (ChatAccess)
        fn get_chat_access(&self, stream_id: i32, user_id: i32) -> Result<Option<ChatAccess>, String> {
            let idx_stream_id = ChatMessageOrmTest::stream_ids().iter().position(|v| *v == stream_id);
            let idx_user_id = ChatMessageOrmTest::user_ids().iter().position(|v| *v == user_id);
            if idx_stream_id.is_none() || idx_user_id.is_none() {
                return Ok(None);
            }
            let idx_stream_id = idx_stream_id.unwrap();

            let stream_owner = ChatMessageOrmTest::user_ids().get(idx_stream_id).unwrap().clone();
            // let stream_live = idx_stream_id != 2;
            let stream_live = stream_id != ChatMessageOrmTest::stream_ids().get(2).unwrap().clone();
            #[rustfmt::skip]
            let is_blocked = (*self.blocked_user_vec).borrow().iter().find(|v| v.blocked_id == user_id).is_some();

            Ok(Some(ChatAccess::new(stream_id, stream_owner, stream_live, is_blocked)))
        }

        /// Get a list of blocked users.
        fn get_blocked_users(&self, user_id: i32) -> Result<Vec<BlockedUser>, String> {
            let vec = (*self.blocked_user_vec).borrow();
            #[rustfmt::skip]
            let result: Vec<BlockedUser> = vec.iter()
                .filter(|v| (*v).user_id == user_id).map(|v| v.clone()).collect();
            Ok(result)
        }

        /// Add a new entry (blocked_user).
        fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String> {
            if create_blocked_user.blocked_id.is_none() && create_blocked_user.blocked_nickname.is_none() {
                return Ok(None);
            }
            let validation_res = create_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }
            let mut opt_user_mini: Option<UserMini> = None;
            if let Some(blocked_id) = create_blocked_user.blocked_id {
                opt_user_mini = self.find_user_by_id(blocked_id);
            } else if let Some(blocked_nickname) = create_blocked_user.blocked_nickname {
                opt_user_mini = self.find_user_by_name(&blocked_nickname);
            }
            let mut result: Option<BlockedUser> = None;
            let mut vec = (*self.blocked_user_vec).borrow_mut();
            if let Some(user_mini) = opt_user_mini {
                let opt_blocked_user = vec
                    .iter()
                    .find(|v| {
                        (*v).user_id == create_blocked_user.user_id
                            && (*v).blocked_id == user_mini.id
                            && (*v).blocked_nickname.eq(&user_mini.name)
                    })
                    .map(|v| v.clone());

                if let Some(blocked_user) = opt_blocked_user {
                    result = Some(blocked_user);
                } else {
                    let cnt = vec.len();
                    let idx: i32 = cnt.try_into().unwrap();
                    let blocked_user = BlockedUser::new(
                        BLOCKED_USER_ID + idx,
                        create_blocked_user.user_id,
                        user_mini.id,
                        user_mini.name.clone(),
                        Some(round_subsecs(Utc::now())),
                    );
                    vec.push(blocked_user.clone());
                    result = Some(blocked_user);
                }
            }
            Ok(result)
        }

        /// Delete an entity (blocked_user).
        fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String> {
            if delete_blocked_user.blocked_id.is_none() && delete_blocked_user.blocked_nickname.is_none() {
                return Ok(None);
            }
            let validation_res = delete_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }
            let mut opt_user_mini: Option<UserMini> = None;
            if let Some(blocked_id) = delete_blocked_user.blocked_id {
                opt_user_mini = self.find_user_by_id(blocked_id);
            } else if let Some(blocked_nickname) = delete_blocked_user.blocked_nickname {
                opt_user_mini = self.find_user_by_name(&blocked_nickname);
            }

            let mut result: Option<BlockedUser> = None;
            let mut vec = (*self.blocked_user_vec).borrow_mut();
            if let Some(user_mini) = opt_user_mini {
                let opt_index = vec.iter().position(|v| {
                    (*v).user_id == delete_blocked_user.user_id
                        && (*v).blocked_id == user_mini.id
                        && (*v).blocked_nickname.eq(&user_mini.name)
                });
                if let Some(index) = opt_index {
                    let blocked_user = vec.remove(index);
                    result = Some(blocked_user);
                }
            }
            Ok(result)
        }
    }

    pub struct ChatMessageOrmTest {}

    impl ChatMessageOrmTest {
        pub fn user_ids() -> Vec<i32> {
            vec![USER1_ID, USER2_ID, USER3_ID, USER4_ID]
        }
        pub fn stream_ids() -> Vec<i32> {
            vec![
                1, // Owner user idx 0 (live: true)  1100 oliver_taylor
                2, // Owner user idx 1 (live: true)  1101 robert_brown
                3, // Owner user idx 2 (live: false) 1102 mary_williams
                4, // Owner user idx 3  blocked      1103 ava_wilson
            ]
        }
        pub fn get_user_name(user_id: i32) -> String {
            match user_id {
                USER1_ID => USER1_NAME,
                USER2_ID => USER2_NAME,
                USER3_ID => USER3_NAME,
                USER4_ID => USER4_NAME,
                _ => "",
            }
            .to_string()
        }
        fn get_blocked_user_vec() -> Vec<BlockedUser> {
            let mut result: Vec<BlockedUser> = Vec::new();
            let user_ids = Self::user_ids();
            let blocked_id = user_ids.last().unwrap().clone();
            let blocked_name = Self::get_user_name(blocked_id);
            for (idx, user_id) in user_ids.iter().enumerate() {
                if *user_id == blocked_id {
                    continue;
                }
                let id = BLOCKED_USER_ID + i32::try_from(idx).unwrap();
                let blocked_nickname = blocked_name.clone();
                result.push(BlockedUser::new(id, *user_id, blocked_id, blocked_nickname, None));
            }
            result
        }
        #[rustfmt::skip]
        fn get_user_mini() -> Vec<UserMini> {
            Self::user_ids().iter().map(|v| UserMini {id: *v, name: Self::get_user_name(*v)}).collect()
        }
        pub fn chat_messages(count_msg: i32) -> (Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>, Vec<UserMini>) {
            let mut chat_message_list: Vec<ChatMessage> = Vec::new();
            let chat_message_log_list: Vec<ChatMessageLog> = Vec::new();

            let mut date_created: DateTime<Utc> = Utc::now() - Duration::minutes(i64::try_from(count_msg + 2).unwrap());
            let stream_id = CH_MSG_STREAM_ID;
            let mut user_id = USER1_ID;
            for idx in 1..=count_msg {
                let msg = Some(format!("msg1{}{}", (if idx < 10 { "0" } else { "" }), idx + 1));
                let user_name = Self::get_user_name(user_id);

                let ch_msg = ChatMessage::new(idx, stream_id, user_id, user_name, msg, date_created, None, None);
                chat_message_list.push(ch_msg);
                date_created = date_created + Duration::minutes(1);
                user_id = if user_id == USER4_ID { USER1_ID } else { user_id + 1 };
            }

            let blocked_user_list: Vec<BlockedUser> = Self::get_blocked_user_vec();
            let user_mini_vec: Vec<UserMini> = Self::get_user_mini();
            let users_list = user_mini_vec.clone();
            #[rustfmt::skip]
            let chat_message_orm = ChatMessageOrmApp::create(
                &chat_message_list, &chat_message_log_list, &blocked_user_list, &users_list);

            let mut chat_message_log_vec: Vec<ChatMessageLog> = Vec::new();
            for (_key, value_vec) in chat_message_orm.chat_message_log_map.iter() {
                for chat_message_log in value_vec {
                    chat_message_log_vec.push(chat_message_log.clone());
                }
            }
            let blocked_user_vec = (*chat_message_orm.blocked_user_vec).borrow().clone();
            let chat_message_vec = chat_message_orm.chat_message_vec.clone();

            (chat_message_vec, chat_message_log_vec, blocked_user_vec, user_mini_vec)
        }
        pub fn cfg_chat_message_orm(
            data_cm: (Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>, Vec<UserMini>),
        ) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_chat_message_orm = web::Data::new(ChatMessageOrmApp::create(&data_cm.0, &data_cm.1, &data_cm.2, &data_cm.3));
                config.app_data(web::Data::clone(&data_chat_message_orm));
            }
        }
    }
}

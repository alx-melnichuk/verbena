use crate::chats::chat_message_models::{ChatMessage, ChatMessageLog, CreateChatMessage, ModifyChatMessage};

pub trait ChatMessageOrm {
    /// Get a list of "chat_message_log" for the specified "chat_message_id".
    fn get_chat_message_logs(&self, chat_message_id: i32) -> Result<Vec<ChatMessageLog>, String>;

    /// Add a new entry (chat_message).
    fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String>;

    /// Modify an entity (chat_message).
    fn modify_chat_message(
        &self,
        user_id: i32,
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

use crate::chats::chat_message_models::{ChatMessage, CreateChatMessage};

pub trait ChatMessageOrm {
    /// Add a new entry (chat_message).
    fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String>;
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
        /// Add a new entry (chat_message).
        fn create_chat_message(&self, create_chat_message: CreateChatMessage) -> Result<ChatMessage, String> {
            eprintln!("}} ChatMessageOrmApp.create_chat_message() ..."); // #
                                                                         // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_chat_message($1,$2,$3);")
                .bind::<sql_types::Integer, _>(create_chat_message.stream_id) // $1
                .bind::<sql_types::Integer, _>(create_chat_message.user_id) // $2
                .bind::<sql_types::Text, _>(create_chat_message.msg); // $3

            // Run a query with Diesel to create a new user and return it.
            let chat_message = query
                .get_result::<ChatMessage>(&mut conn)
                .map_err(|e| format!("create_profile_user: {}", e.to_string()))?;
            eprintln!("}} ChatMessageOrmApp.create_chat_message() ...finish"); // #
            Ok(chat_message)
        }
    }
}

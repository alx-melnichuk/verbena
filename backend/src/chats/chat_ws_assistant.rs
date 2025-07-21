use log::error;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{
    chat_message_models::{
        BlockedUser, ChatAccess, ChatMessage, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser,
        ModifyChatMessage,
    },
    chat_message_orm::ChatMessageOrm,
};
use crate::errors::AppError;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{profile_models::Profile /*profile_orm::ProfileOrm*/};
use crate::sessions::config_jwt;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::tokens::decode_token;
use crate::settings::err;
use crate::utils::token_verification::check_token_and_get_profile;

#[derive(Debug, Clone)]
pub struct ChatStream {
    pub id: i32,
    pub user_id: i32,
    pub live: bool,
}

// ** ChatWsAssistant **

#[derive(Debug, Clone)]
pub struct ChatWsAssistant {
    config_jwt: config_jwt::ConfigJwt,
    chat_message_orm: ChatMessageOrmApp,
    profile_orm: ProfileOrmApp,
    session_orm: SessionOrmApp,
}

// ** ChatWsAssistant implementation **

impl ChatWsAssistant {
    pub fn new(
        config_jwt: config_jwt::ConfigJwt,
        chat_message_orm: ChatMessageOrmApp,
        profile_orm: ProfileOrmApp,
        session_orm: SessionOrmApp,
    ) -> Self {
        ChatWsAssistant {
            config_jwt,
            chat_message_orm,
            profile_orm,
            session_orm,
        }
    }
    /** Decode the token. And unpack the two parameters from the token. */
    pub fn decode_and_verify_token(&self, token: &str) -> Result<(i32, i32), String> {
        let jwt_secret: &[u8] = self.config_jwt.jwt_secret.as_bytes();
        // Decode the token. Unpack two parameters from the token.
        decode_token(token, jwt_secret).map_err(|e| format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e))
    }
    /** Check the number token for correctness and get the user profile. */
    pub async fn check_num_token_and_get_profile(&self, user_id: i32, num_token: i32) -> Result<Profile, AppError> {
        let session_orm: SessionOrmApp = self.session_orm.clone();
        let profile_orm: ProfileOrmApp = self.profile_orm.clone();
        // Check the token for correctness and get the user profile.
        check_token_and_get_profile(user_id, num_token, &session_orm, &profile_orm).await
    }
    /** Create a new user message in the chat. */
    pub async fn execute_create_chat_message(
        &self,
        stream_id: i32,
        user_id: i32,
        msg: &str,
    ) -> Result<Option<ChatMessage>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let create_chat_message = CreateChatMessage::new(stream_id, user_id, msg);
        // Add a new entity (stream).
        chat_message_orm.create_chat_message(create_chat_message).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        })
    }
    /** Change a user's message in a chat. */
    pub async fn execute_modify_chat_message(
        &self,
        id: i32,
        user_id: i32,
        new_msg: &str,
    ) -> Result<Option<ChatMessage>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let modify_chat_message = ModifyChatMessage::new(new_msg.to_owned());
        // Modify an entity (chat_message).
        chat_message_orm
            .modify_chat_message(id, user_id, modify_chat_message)
            .map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            })
    }
    /** Delete a user's message in a chat. */
    pub async fn execute_delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        // Add a new entity (stream).
        chat_message_orm.delete_chat_message(id, user_id).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        })
    }
    /** Perform blocking/unblocking of a user. */
    pub async fn execute_block_user(
        &self,
        is_block: bool,
        user_id: i32,
        blocked_id: Option<i32>,
        blocked_nickname: Option<String>,
    ) -> Result<Option<BlockedUser>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        if is_block {
            // Add a new entry (blocked_user).
            chat_message_orm
                .create_blocked_user(CreateBlockedUser::new(user_id, blocked_id, blocked_nickname))
                .map_err(|e| {
                    error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e)
                })
        } else {
            // Delete an entity (blocked_user).
            chat_message_orm
                .delete_blocked_user(DeleteBlockedUser::new(user_id, blocked_id, blocked_nickname))
                .map_err(|e| {
                    error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e)
                })
        }
    }

    /** Get information about the live of the stream. */
    pub async fn get_stream_live(&self, stream_id: i32) -> Result<Option<bool>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();

        chat_message_orm.get_stream_live(stream_id).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
    }

    /** Get chat access information. (ChatAccess) */
    pub async fn get_chat_access(&self, stream_id: i32, user_id: i32) -> Result<Option<ChatAccess>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();

        chat_message_orm.get_chat_access(stream_id, user_id).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
    }
}

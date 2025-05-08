use log::error;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::chat_message_orm::ChatMessageOrm;
use crate::errors::AppError;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{profile_models::Profile /*profile_orm::ProfileOrm*/};
use crate::sessions::config_jwt;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
// use crate::sessions::session_orm::SessionOrm;
use crate::chats::chat_message_models::{ChatMessage, CreateChatMessage, ModifyChatMessage};
use crate::sessions::tokens::decode_token;
use crate::settings::err;
use crate::utils::token_verification::check_token_and_get_profile;

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
        decode_token(token, jwt_secret).map_err(|e| format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e))
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
        create_chat_message: CreateChatMessage,
    ) -> Result<ChatMessage, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
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
        modify_chat_message: ModifyChatMessage,
    ) -> Result<Option<ChatMessage>, AppError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        // Modify an entity (chat_message).
        chat_message_orm.modify_chat_message(id, modify_chat_message).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        })
    }
}

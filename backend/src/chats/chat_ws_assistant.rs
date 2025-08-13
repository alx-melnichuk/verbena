use actix_web::http::StatusCode;
use log::error;
use vrb_common::api_error::{code_to_str, ApiError};
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_dbase::user_auth::user_auth_orm::impls::UserAuthOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_dbase::user_auth::user_auth_orm::tests::UserAuthOrmApp;
use vrb_dbase::user_auth::{config_jwt, user_auth_models::User, user_auth_orm::UserAuthOrm};
use vrb_tools::{err, token_coding};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{
    chat_message_models::{
        BlockedUser, ChatAccess, ChatMessage, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser, ModifyChatMessage,
    },
    chat_message_orm::ChatMessageOrm,
};
use crate::extractors::authentication2::{is_session_not_found, is_unacceptable_token_id, is_unacceptable_token_num};

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
    user_auth_orm: UserAuthOrmApp,
}

// ** ChatWsAssistant implementation **

impl ChatWsAssistant {
    pub fn new(config_jwt: config_jwt::ConfigJwt, chat_message_orm: ChatMessageOrmApp, user_auth_orm: UserAuthOrmApp) -> Self {
        ChatWsAssistant {
            config_jwt,
            chat_message_orm,
            user_auth_orm,
        }
    }
    /** Decode the token. And unpack the two parameters from the token. */
    pub fn decode_and_verify_token(&self, token: &str) -> Result<(i32, i32), String> {
        let jwt_secret: &[u8] = self.config_jwt.jwt_secret.as_bytes();
        // Decode the token. Unpack two parameters from the token.
        token_coding::decode_token(token, jwt_secret).map_err(|e| format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e))
    }
    /** Check the correctness of the numeric token and get the user data. */
    pub fn check_num_token_and_get_user(&self, user_id: i32, num_token: i32) -> Result<User, ApiError> {
        let user_auth_orm: UserAuthOrmApp = self.user_auth_orm.clone();

        // Find user session by "id" from token.
        let opt_session = user_auth_orm.get_session_by_id(user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            return ApiError::create(507, err::MSG_DATABASE, &e); // 507
        })?;
        // If the session is missing, then return an error406("NotAcceptable", "session_not_found; user_id: {}").
        let session = is_session_not_found(opt_session, user_id)?;
        // Each session contains an additional numeric value "num_token".
        // If "num_token" is not equal to "session.num_token", return error401(c)("Unauthorized","unacceptable_token_num; user_id: {}").
        let _ = is_unacceptable_token_num(&session, num_token, user_id)?;
        // Find user by "id" from token.
        let opt_user = user_auth_orm.get_user_by_id(user_id, false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })?;
        // If the user is missing, then return an error401(d)("Unauthorized", "unacceptable_token_id; user_id: {}").
        let user = is_unacceptable_token_id(opt_user, user_id)?;
        Ok(user)
    }
    /** Create a new user message in the chat. */
    pub async fn execute_create_chat_message(&self, stream_id: i32, user_id: i32, msg: &str) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let create_chat_message = CreateChatMessage::new(stream_id, user_id, msg);
        // Add a new entity (stream).
        chat_message_orm.create_chat_message(create_chat_message).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }
    /** Change a user's message in a chat. */
    pub async fn execute_modify_chat_message(&self, id: i32, user_id: i32, new_msg: &str) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let modify_chat_message = ModifyChatMessage::new(new_msg.to_owned());
        // Modify an entity (chat_message).
        chat_message_orm.modify_chat_message(id, user_id, modify_chat_message).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }
    /** Delete a user's message in a chat. */
    pub async fn execute_delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        // Add a new entity (stream).
        chat_message_orm.delete_chat_message(id, user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }
    /** Perform blocking/unblocking of a user. */
    pub async fn execute_block_user(
        &self,
        is_block: bool,
        user_id: i32,
        blocked_id: Option<i32>,
        blocked_nickname: Option<String>,
    ) -> Result<Option<BlockedUser>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        if is_block {
            // Add a new entry (blocked_user).
            chat_message_orm
                .create_blocked_user(CreateBlockedUser::new(user_id, blocked_id, blocked_nickname))
                .map_err(|e| {
                    error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                    ApiError::create(507, err::MSG_DATABASE, &e) // 507
                })
        } else {
            // Delete an entity (blocked_user).
            chat_message_orm
                .delete_blocked_user(DeleteBlockedUser::new(user_id, blocked_id, blocked_nickname))
                .map_err(|e| {
                    error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                    ApiError::create(507, err::MSG_DATABASE, &e) // 507
                })
        }
    }

    /** Get information about the live of the stream. */
    pub async fn get_stream_live(&self, stream_id: i32) -> Result<Option<bool>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();

        chat_message_orm.get_stream_live(stream_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507 // 507
        })
    }

    /** Get chat access information. (ChatAccess) */
    pub async fn get_chat_access(&self, stream_id: i32, user_id: i32) -> Result<Option<ChatAccess>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();

        chat_message_orm.get_chat_access(stream_id, user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507 // 507
        })
    }
}

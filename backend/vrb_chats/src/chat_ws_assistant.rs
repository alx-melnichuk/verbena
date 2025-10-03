use actix_web::http::StatusCode;
use log::error;
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_authent::user_orm::impls::UserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_authent::user_orm::tests::UserOrmApp;
use vrb_authent::{
    authentication::{is_session_not_found, is_unacceptable_token_id, is_unacceptable_token_num},
    config_jwt,
    user_models::User,
    user_orm::UserOrm,
};
use vrb_common::{
    api_error::{ApiError, code_to_str},
    err,
};
use vrb_tools::token_coding;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chat_message_orm::tests::ChatMessageOrmApp;
use crate::{
    chat_message_models::{
        BlockedUser, ChatAccess, ChatMessage, CreateBlockedUser, CreateChatMessage, DeleteBlockedUser, ModifyChatMessage,
    },
    chat_message_orm::ChatMessageOrm,
};

#[derive(Debug, Clone)]
pub struct ChatStream {
    pub id: i32,
    pub user_id: i32,
    pub live: bool,
}

// ** AssistantChatMsg **

pub trait AssistantChatMsg {
    /** Create a new user message in the chat. */
    fn execute_create_chat_message(&self, stream_id: i32, user_id: i32, msg: &str) -> Result<Option<ChatMessage>, ApiError>;
    /** Change a user's message in a chat. */
    fn execute_modify_chat_message(&self, id: i32, user_id: i32, new_msg: &str) -> Result<Option<ChatMessage>, ApiError>;
    /** Delete a user's message in a chat. */
    fn execute_delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, ApiError>;
}

// ** AssistantBlockUser **

pub trait AssistantBlockUser {
    /** Perform blocking/unblocking of a user. */
    fn execute_block_user(
        &self,
        is_block: bool,
        user_id: i32,
        blocked_id: Option<i32>,
        blocked_nickname: Option<String>,
    ) -> Result<Option<BlockedUser>, ApiError>;
}

// ** ChatWsAssistant **

#[derive(Debug, Clone)]
pub struct ChatWsAssistant {
    config_jwt: config_jwt::ConfigJwt,
    chat_message_orm: ChatMessageOrmApp,
    user_orm: UserOrmApp,
}

// ** ChatWsAssistant implementation **

impl ChatWsAssistant {
    pub fn new(config_jwt: config_jwt::ConfigJwt, chat_message_orm: ChatMessageOrmApp, user_orm: UserOrmApp) -> Self {
        ChatWsAssistant {
            config_jwt,
            chat_message_orm,
            user_orm,
        }
    }
    /** Decode the token. And unpack the two parameters from the token. */
    pub fn decode_and_verify_token(&self, token: &str) -> Result<(i32, i32), String> {
        let jwt_secret: &[u8] = self.config_jwt.jwt_secret.as_bytes();
        // Decode the token. Unpack two parameters from the token.
        token_coding::decode_token(token, jwt_secret).map_err(|e| format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e))
    }
    /** Check the correctness of the numeric token and get the user data. */
    pub async fn check_num_token_and_get_user(&self, user_id: i32, num_token: i32) -> Result<User, ApiError> {
        let user_orm: UserOrmApp = self.user_orm.clone();

        // Token verification:
        // 1. Search for a session by "id" from the token;
        let opt_session = user_orm.get_session_by_id(user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            return ApiError::create(507, err::MSG_DATABASE, &e); // 507
        })?;
        // If the session does not exist, return error 406("NotAcceptable", "session_not_found; user_id: {}").
        let session = is_session_not_found(opt_session, user_id)?;
        // 2. Compare "num_token" from session with "num_token" from token;
        // To block hacking, the session contains a numeric value "num_token".
        // If session.num_token is not equal to token.num_token,return error401(c)("Unauthorized","unacceptable_token_num; user_id: {}")
        let _ = is_unacceptable_token_num(&session, num_token, user_id)?;
        // 3. If everything is correct, then search for the user by "user_id" from the token;
        let opt_user = user_orm.get_user_by_id(user_id, false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })?;
        // If the user is not present, return error401(d)("Unauthorized", "unacceptable_token_id; user_id: {}").
        let user = is_unacceptable_token_id(opt_user, user_id)?;
        Ok(user)
    }
    /** Get chat access information. (ChatAccess) */
    pub async fn get_chat_access(&self, stream_id: i32, opt_user_id: Option<i32>) -> Result<Option<ChatAccess>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();

        chat_message_orm.get_chat_access(stream_id, opt_user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }
}

// ** AssistantBlockUser **

impl AssistantBlockUser for ChatWsAssistant {
    /** Perform blocking/unblocking of a user. */
    fn execute_block_user(
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
}

// ** AssistantChatMsg **

impl AssistantChatMsg for ChatWsAssistant {
    /** Create a new user message in the chat. */
    fn execute_create_chat_message(&self, stream_id: i32, user_id: i32, msg: &str) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let create_chat_message = CreateChatMessage::new(stream_id, user_id, msg);
        // Add a new entity (stream).
        chat_message_orm.create_chat_message(create_chat_message).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }

    /** Change a user's message in a chat. */
    fn execute_modify_chat_message(&self, id: i32, user_id: i32, new_msg: &str) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        let modify_chat_message = ModifyChatMessage::new(new_msg.to_owned());
        // Modify an entity (chat_message).
        chat_message_orm.modify_chat_message(id, user_id, modify_chat_message).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }

    /** Delete a user's message in a chat. */
    fn execute_delete_chat_message(&self, id: i32, user_id: i32) -> Result<Option<ChatMessage>, ApiError> {
        let chat_message_orm: ChatMessageOrmApp = self.chat_message_orm.clone();
        // Add a new entity (stream).
        chat_message_orm.delete_chat_message(id, user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
    }
}

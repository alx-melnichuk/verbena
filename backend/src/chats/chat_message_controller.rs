use std::{borrow::Cow, ops::Deref, time::Instant as tm};

use actix_files::NamedFile;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_actors::ws;
use chrono::{DateTime, Duration, TimeZone, Utc};
use log::{error, info, log_enabled, Level::Info};
use utoipa;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::blocked_user_orm::impls::BlockedUserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::blocked_user_orm::tests::BlockedUserOrmApp;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{
    chat_message_models::{
        ChatMessageDto, CreateChatMessage, CreateChatMessageDto, FilterChatMessage, FilterChatMessageDto,
        ModifyChatMessage, ModifyChatMessageDto,
    },
    chat_message_orm::ChatMessageOrm,
    chat_ws_assistant::ChatWsAssistant,
    chat_ws_session::ChatWsSession,
};
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::sessions::config_jwt;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::settings::err;
use crate::users::user_models::UserRole;
use crate::utils::parser;
use crate::validators::{msg_validation, Validator};

// 403 Access denied - insufficient user rights.
pub const MSG_MODIFY_ANOTHER_USERS_CHAT_MESSAGE: &str = "modify_another_users_chat_message";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        if cfg!(debug_assertions) {
            config
                // GET /chat
                .service(chat);
        }
        config
            // GET /ws
            .service(get_ws_chat)
            // GET /api/chat_messages
            .service(get_chat_message)
            // POST /api/chat_messages
            .service(post_chat_message)
            // PUT /api/chat_messages/{id}
            .service(put_chat_message)
            // DELETE /api/chat_messages/{id}
            .service(delete_chat_message);
    }
}

// ** Section: get_ws_chat **

/// get_ws_chat
///
/// Implementation of sending and receiving messages in chat.
///
/// When accessing the URL "wss://localhost:8080/ws" (GET), a web socket is opened and interaction with the server occurs.
///
/// Successful response status: 101 Switching Protocols.
///
/// The following commands are processed:
///
/// - ## The "echo" command.
/// Checking the connection to the server (send text to receive it back).
///
/// *Client* :<img width=200/>*Server* :<br/>
/// `{ "echo": "text echo" }`<img width=35/>`{ "echo": "text echo" }`<br/>
///
/// *Client* :<br/>
/// `{ "echo": "" }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'echo'" }`<br/>
///
/// - ## The "name" command.
/// Set user nickname (not required for authorized).
///
/// *Client* :<img width=200/>*Server* :<br/>
/// `{ "name": "nickname" }`<img width=43/>`{ "name": "nickname" }`<br/>
///
/// *Client* :<br/>
/// `{ "name": "" }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'name'" }`<br/>
///
/// - ## The "join" command.
/// Perform join the chat room with authentication.
///
/// *Client* :<br/>
/// `{ "join": 1, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
///
/// ```text
/// {
///   "join": number,        // Stream ID.
///   "access": string,      // Token received after authorization.
/// }
/// ```
/// *Server* (Reply to the initiator):<br/>
/// `{ "join": 1, "member": "oliver_taylor", "count": 1, "is_owner": false, "is_blocked": false }`<br/>
///
/// *Server* (Reply to everyone else):<br/>
/// `{ "join": 1, "member": "oliver_taylor", "count": 1 }`<br/>
///
/// ```text
/// {
///   "join": number,        // Stream ID.
///   "member": string,      // User nickname.
///   "count": number,       // Number of connected users.
///   "is_owner": boolean,   // The user is the owner of the chat.
///   "is_blocked": boolean, // The user has been blocked.
/// }
/// ```
/// *Client* :<br/>
/// `{ "join": 0, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'join'" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "session_not_found; user_id: 1" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
/// *Server* :<br/>
/// `{ "err": 409, "code": "Conflict", "message": "was_already_join_to_room" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
/// *Server* :<br/>
/// `{ "err": 409, "code": "Conflict", "message": "stream_not_active" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 999999, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT..." }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "stream_not_found; stream_id: 999999" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT...err" }`<br/>
/// *Server* :<br/>
/// `{ "err": 401, "code": "Unauthorized", "message": "invalid_or_expired_token; Base64Url must contain: 'A-Z','a-z','0-9','-','_' and have a length that is a multiple of 4." }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT...err" }`<br/>
/// *Server* :<br/>
/// `{ "err": 401, "code": "Unauthorized", "message": "unacceptable_token_num; user_id: 1" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 4, "access":"BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT...err" }`<br/>
/// *Server* :<br/>
/// `{ "err": 401, "code": "Unauthorized", "message": "unacceptable_token_id; user_id: 1" }`<br/>
///
/// - ## The "join" command (without authentication).
/// Perform join the chat room without authentication.
///
/// *Client* :<br/>
/// `{ "join": 1 }`<br/>
///
/// ```text
/// {
///   "join": number,        // Stream ID.
/// }
/// ```
/// *Server* (Reply to the initiator):<br/>
/// `{ "join": 1, "member": "", "count": 1, "is_owner": false, "is_blocked": true }`<br/>
///
/// *Server* (Reply to everyone else):<br/>
/// `{ "join": 1, "member": "", "count": 1 }`<br/>
///
/// ```text
/// {
///   "join": number,        // Stream ID.
///   "member": string,      // User nickname.
///   "count": number,       // Number of connected users.
///   "is_owner": boolean,   // The user is the owner of the chat. Always false.
///   "is_blocked": boolean, // The user has been blocked. Always true.
/// }
/// ```
/// *Client* :<br/>
/// `{ "join": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'join'" }`<br/>
///
/// *Client* :<br/>
/// `{ "join": 3 }`<br/>
/// *Server* :<br/>
/// `{ "err": 409, "code": "Conflict", "message": "stream_not_active" }`<br/>
///
/// - ## The "leave" command.
/// Leave from the chat room. You can only be present in one room. Therefore, when leaving the room, the ID value can be set to 0.
///
/// *Client* :<br/>
/// `{ "leave": 0 }`<br/>
///
/// ```text
/// {
///   "leave": number,        // Stream ID.
/// }
/// ```
/// *Server* :<br/>
/// `{ "leave": 1, "member": "oliver_taylor", "count": 0 }`<br/>
///
/// ```text
/// {
///   "leave": number,        // Stream ID.
///   "member": string,       // User nickname.
///   "count": number,        // Number of connected users.
/// }
/// ```
/// *Client* :<br/>
/// `{ "leave": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// - ## The "count" command.
/// Request for number of connected users.
///
/// *Client* :<br/>
/// `{ "count": 0 }`<br/>
///
/// ```text
/// {
///   "count": number,        // Number of connected users.
/// }
/// ```
/// *Server* :<br/>
/// `{ "count": 1 }`<br/>
///
/// ```text
/// {
///   "count": number,        // Number of connected users.
/// }
/// ```
/// *Client* :<br/>
/// `{ "count": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// - ## The "msg" command.
/// Send a message to the chat room. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "msg": "message 1" }`<br/>
///
/// ```text
/// {
///   "msg": string,          // Message test.
/// }
/// ```
/// *Server* :<br/>
/// `{ "msg": "message 1", "id":1, "member": "ethan_brown", "date": "2020-03-11T09:00:00.000Z", "isEdt": false, "isRmv": false }`<br/>
///
/// ```text
/// {
///   "msg": string,          // Message test.
///   "id": number,           // Message ID.
///   "member": string,       // The nickname of the user who sent the message.
///   "date": string,         // Date string in ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
///   "isEdt": boolean,       // Message editing indicator.
///   "isRmv": boolean,       // Message deletion indicator.
/// }
/// ```
/// *Client* :<br/>
/// `{ "msg": "" }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'msg'" }`<br/>
///
/// *Client* :<br/>
/// `{ "msg": "text1" }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "msg": "text2" }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "block_on_sending_messages" }`<br/>
///
/// - ## The "msgPut" command.
/// Correcting a message in a chat room. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "msgPut": "message 2", "id": 1 }`<br/>
///
/// ```text
/// {
///   "msgPut": string,       // Message test.
///   "id": number,           // Message ID.
/// }
/// ```
/// *Server* :<br/>
/// `{ "msg": "message 2", "id": 1, "member": "oliver_taylor", "date": "2020-03-11T09:10:00.000Z", "isEdt": true, "isRmv": false }`<br/>
///
/// ```text
/// {
///   "msg": string,          // Message test.
///   "id": number,           // Message ID.
///   "member": string,       // The nickname of the user who sent the message.
///   "date": string,         // Date string in ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
///   "isEdt": boolean,       // Message editing indicator.
///   "isRmv": boolean,       // Message deletion indicator.
/// }
/// ```
/// *Client* :<br/>
/// `{ "msgPut": "", "id": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'msgPut'" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgPut": "text1", "id": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'id'" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgPut": "text1", "id": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgPut": "text2", "id": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "block_on_sending_messages" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgPut": "text9", "id": 999999 }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "chat_message_not_found; id: 999999, user_id: 1" }`<br/>
///
/// - ## The "msgCut" command.
/// Deleting the text of a message to the chat room. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "msgCut": "", "id": 1 }`<br/>
///
/// ```text
/// {
///   "msgCut": string,       // The ID of the message whose content you want to remove.
///   "id": number,           // Message ID.
/// }
/// ```
/// *Server* :<br/>
/// `{ "msg": "", "id": 1, "member": "oliver_taylor", "date": "2020-03-11T09:20:00.000Z", "isEdt": false, "isRmv": true }`<br/>
///
/// ```text
/// {
///   "msg": string,          // Message test.
///   "id": number,           // Message ID.
///   "member": string,       // The nickname of the user who sent the message.
///   "date": string,         // Date string in ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
///   "isEdt": boolean,       // Message editing indicator.
///   "isRmv": boolean,       // Message deletion indicator.
/// }
/// ```
/// *Client* :<br/>
/// `{ "msgCut": "", "id": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'id'" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgCut": "", "id": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgCut": "", "id": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "block_on_sending_messages" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgCut": "", "id": 999999 }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "chat_message_not_found; id: 999999, user_id: 1" }`<br/>
///
/// - ## The "msgRmv" command.
/// Deleting a message in a chat. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "msgRmv": 1 }`<br/>
///
/// ```text
/// {
///   "msgRmv": number,       // The ID of the message to be deleted.
/// }
/// ```
/// *Server* :<br/>
/// `{ "msgRmv": 1 }`<br/>
///
/// ```text
/// {
///   "msgRmv": number,       // The ID of the message to be deleted.
/// }
/// ```
/// *Client* :<br/>
/// `{ "msgRmv": 0 }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'msgRmv'" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgRmv": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgRmv": 1 }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "block_on_sending_messages" }`<br/>
///
/// *Client* :<br/>
/// `{ "msgRmv": 999999 }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "chat_message_not_found; id: 999999, user_id: 1" }`<br/>
///
/// - ## The "block" command.
/// The stream owner can block a user. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "block": "nickname" }`<br/>
///
/// ```text
/// {
///   "block": string,       // The nickname of the user to be blocked.
/// }
/// ```
/// *Server* :<br/>
/// `{ "block": "nickname", "is_in_chat": false }`<br/>
///
/// ```text
/// {
///   "block": string,       // The nickname of the user who was blocked.
///   "is_in_chat": boolean, // Indication that the user is in a chat.
/// }
/// ```
/// *Client* :<br/>
/// `{ "block": "" }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'block'" }`<br/>
///
/// *Client* :<br/>
/// `{ "block": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "block": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "stream_owner_rights_missing" }`<br/>
///
/// *Client* :<br/>
/// `{ "block": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "user_not_found; blocked_nickname: 'nickname'" }`<br/>
///
/// - ## The "unblock" command.
/// The stream owner can unblock a user. Available only to authorized users.
///
/// *Client* :<br/>
/// `{ "unblock": "nickname" }`<br/>
///
/// ```text
/// {
///   "unblock": string,     // The nickname of the user to be unblocked.
/// }
/// ```
/// *Server* :<br/>
/// `{ "unblock": "nickname", "is_in_chat": false }`<br/>
///
/// ```text
/// {
///   "unblock": string,     // The nickname of the user who was unblocked.
///   "is_in_chat": boolean, // Indication that the user is in a chat.
/// }
/// ```
/// *Client* :<br/>
/// `{ "unblock": "" }`<br/>
/// *Server* :<br/>
/// `{ "err": 400, "code": "BadRequest", "message": "parameter_not_defined; name: 'unblock" } } }`<br/>
///
/// *Client* :<br/>
/// `{ "unblock": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 406, "code": "NotAcceptable", "message": "was_no_join_command" }`<br/>
///
/// *Client* :<br/>
/// `{ "unblock": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 403, "code": "Forbidden", "message": "stream_owner_rights_missing" }`<br/>
///
/// *Client* :<br/>
/// `{ "unblock": "nickname" }`<br/>
/// *Server* :<br/>
/// `{ "err": 404, "code": "NotFound", "message": "user_not_found; blocked_nickname: 'nickname'" }`<br/>
///
#[utoipa::path(
    responses(
        (status = 101, description = "Connecting a websocket to a server."),
    ),
)]
#[get("/ws")]
pub async fn get_ws_chat(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    blocked_user_orm: web::Data<BlockedUserOrmApp>,
    request: actix_web::HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse<actix_web::body::BoxBody>, actix_web::Error> {
    let config_jwt = config_jwt.get_ref().clone();
    let chat_message_orm_app = chat_message_orm.get_ref().clone();
    let profile_orm_app = profile_orm.get_ref().clone();
    let session_orm_app = session_orm.get_ref().clone();
    let blocked_user_orm = blocked_user_orm.get_ref().clone();
    #[rustfmt::skip]
    let assistant = ChatWsAssistant::new(
        config_jwt, chat_message_orm_app, profile_orm_app, session_orm_app, blocked_user_orm);

    let chat_ws_session = ChatWsSession::new(
        u64::default(),    // id: u64,
        i32::default(),    // room_id: i32,
        i32::default(),    // user_id: i32,
        String::default(), // user_name: String,
        bool::default(),   // is_owner: bool,
        bool::default(),   // is_blocked: bool,
        assistant,         // assistant: ChatWsAssistant
    );
    ws::start(chat_ws_session, &request, stream)
}

#[cfg(debug_assertions)]
#[get("/chat")]
async fn chat() -> impl Responder {
    NamedFile::open_async("./static/chat_index.html").await.unwrap()
}

fn get_ch_msgs(start: u16, finish: u16) -> Vec<ChatMessageDto> {
    let mut result: Vec<ChatMessageDto> = Vec::new();
    if 19 < start || 19 < finish {
        return vec![];
    }
    let mut current: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 7, 1, 10, 30, 0).unwrap();
    let is_asc = start < finish;
    let mut idx = start;
    if idx > 0 {
        current = current + Duration::minutes((idx * 5).into());
    }
    let dlt_minutes = if is_asc { 5 } else { -5 };
    while is_asc && idx <= finish || !is_asc && idx >= finish {
        #[rustfmt::skip]
        let member = if idx % 2 == 0 { "ava_wilson" } else if idx % 3 == 0 { "ethan_brown" } else { "james_miller" };
        result.push(ChatMessageDto {
            id: 200 + i32::from(idx),
            date: current.clone(),
            member: member.into(),
            msg: format!("Demo message {}", idx),
            is_edt: false,
            is_rmv: false,
        });

        current = current + Duration::minutes(dlt_minutes);
        if !is_asc && idx == 0 {
            break;
        }
        idx = if is_asc { idx + 1 } else { idx - 1 };
    }
    result
}

// ** Section: ChatMessage Get **

/// get_chat_message
///
/// Get a list of messages from a chat.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1
/// ```
/// 
/// The `stream_id` parameter (required) specifies the stream identifier.
/// 
/// The `is_sort_des` parameter is a descending sorting flag (default is false, i.e. the default sorting is "ascending").
/// 
/// ?? border_by_id
/// 
/// The `limit` parameter determines the number of records in the response.
/// 
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=true&border_by_id=120&limit=20
/// ```
/// Returns the found list of chat messages (Vec<`ChatMessageDto`>) with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The result is an array of chat messages.", body = Vec<ChatMessageDto>,
        examples(
            ("sort_ascending" = (description = "Chat messages are sorted in ascending order, number of entries 4. `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=true&limit=4`",
                summary = "sort ascending", value = json!(get_ch_msgs(0, 3))
            )),
            ("sort_ascending_part2" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 203 (part 2). `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=true&limit=4&border_by_id=203`",
                summary = "sort ascending part2", value = json!(get_ch_msgs(4, 7))
            )),
            ("sort_ascending_part3" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 207 (part 3). `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=true&limit=4&border_by_id=207`",
                summary = "sort ascending part3", value = json!(get_ch_msgs(8, 11))
            )),
            ("sort_descending" = (description = "Chat messages are sorted in descending order. `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=false&limit=4`",
                summary = "sort descending", value = json!(get_ch_msgs(11, 8))
            )),
            ("sort_descending_part2" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 2). `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=false&limit=4&border_by_id=208`",
                summary = "sort descending part2", value = json!(get_ch_msgs(7, 4))
            )),
            ("sort_descending_part3" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 3). `curl -i -X GET http://localhost:8080/api/chat_messages?stream_id=1&is_sort_des=false&limit=4&border_by_id=204`",
                summary = "sort descending part3", value = json!(get_ch_msgs(7, 4))
            )),
        ),
        ),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 506, description = "Blocking error.", body = AppError,
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError,
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_chat_message(
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    query_params: web::Query<FilterChatMessageDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get search parameters.
    let filter_chat_message = FilterChatMessage::convert(query_params.into_inner());
    
    let res_data = web::block(move || {
        // Find for an entity (stream event) by SearchStreamEvent.
        let res_data =
        chat_message_orm.filter_chat_messages(filter_chat_message)
        .map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let chat_messages = match res_data { Ok(v) => v, Err(e) => return Err(e) };
    let chat_message_dto_list: Vec<ChatMessageDto> = chat_messages.iter()
        .map(|ch_msg| ChatMessageDto::from(ch_msg.clone()))
        .collect();

    if let Some(timer) = timer {
        info!("get_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Ok().json(chat_message_dto_list)) // 200
}

#[rustfmt::skip]
#[post("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateChatMessageDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let create_chat_message_dto: CreateChatMessageDto = json_body.into_inner();

    let stream_id = create_chat_message_dto.stream_id;
    let msg = create_chat_message_dto.msg.clone();
    
    let create_chat_message = CreateChatMessage::new(stream_id, user_id, &msg);

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2.create_chat_message(create_chat_message).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));
    
    if let Some(timer) = timer {
        info!("post_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Created().json(chat_message_dto)) // 201
    } else {
        let json = serde_json::json!({ "stream_id": stream_id, "msg": &msg });
        let message = format!("{}; stream_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, stream_id, &msg);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        Err(AppError::not_acceptable406(&message) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

// PUT /api/chat_messages/{id}
#[rustfmt::skip]
#[put("/api/chat_messages/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<ModifyChatMessageDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let opt_user_id: Option<i32> = if profile.role == UserRole::Admin { None } else { Some(profile.user_id) };

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;
    
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let modify_chat_message_dto: ModifyChatMessageDto = json_body.into_inner();
    let msg = modify_chat_message_dto.msg.clone();
   
    let modify_chat_message = ModifyChatMessage::new(msg.clone());

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2
            .modify_chat_message(id, opt_user_id, modify_chat_message)
            .map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;
    
    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));
    
    if let Some(timer) = timer {
        info!("put_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Ok().json(chat_message_dto)) // 200
    } else {
        let json = serde_json::json!({ "id": id, "user_id": profile.user_id, "msg": msg });
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, id, profile.user_id, msg);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        Err(AppError::not_acceptable406(&message) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

// DELETE /api/chat_messages/{id}
#[rustfmt::skip]
#[delete("/api/chat_messages/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let opt_user_id: Option<i32> = if profile.role == UserRole::Admin { None } else { Some(profile.user_id) };

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;
    
    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2
            .delete_chat_message(id, opt_user_id)
            .map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));

    if let Some(timer) = timer {
        info!("delete_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Ok().json(chat_message_dto)) // 200
    } else {
        let json = serde_json::json!({ "id": id, "user_id": profile.user_id });
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}", err::MSG_PARAMETER_UNACCEPTABLE, id, profile.user_id);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        Err(AppError::not_acceptable406(&message) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::{http, web};
    use chrono::{DateTime, Utc};

    use crate::chats::{
        blocked_user_models::BlockedUser,
        blocked_user_orm::tests::{BlockedUserOrmApp, UserMini},
        chat_message_models::{ChatMessage, ChatMessageLog},
        chat_message_orm::tests::{ChatMessageOrmApp, ChatMsgTest},
    };
    use crate::profiles::{
        profile_models::Profile, profile_orm::tests::ProfileOrmApp, profile_orm::tests::PROFILE_USER_ID as PROFILE_ID,
    };
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::users::user_models::UserRole;
    use crate::utils::token::BEARER;

    pub const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    pub const MSG_JSON_MISSING_FIELD: &str = "Json deserialize error: missing field";
    pub const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    pub const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    /** 1-"Oliver_Taylor", 2-"Robert_Brown", 3-"Mary_Williams", 4-"Ava_Wilson" */
    fn create_profile(user_id: i32) -> Profile {
        let user_ids = ChatMsgTest::user_ids().clone();
        #[rustfmt::skip]
        let user_id1 = if user_id > 0 { user_id } else { user_ids.get(0).unwrap().clone() };
        let idx_user_id = user_ids.iter().position(|&u| u == user_id1).unwrap();
        let user_id = user_ids.get(idx_user_id).unwrap().clone();
        let nickname = ChatMsgTest::user_names().get(idx_user_id).unwrap().clone();
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(user_id, &nickname, &format!("{}@gmail.com", &nickname), role);
        profile
    }
    pub fn create_chat_message(
        id: i32,
        stream_id: i32,
        user_id: i32,
        msg: &str,
        date_update: DateTime<Utc>,
    ) -> ChatMessage {
        #[rustfmt::skip]
        let stream_id = if stream_id > 0 { stream_id } else { ChatMsgTest::stream_ids().get(0).unwrap().clone() };
        let user_ids = ChatMsgTest::user_ids().clone();
        #[rustfmt::skip]
        let user_id1 = if user_id > 0 { user_id } else { user_ids.get(0).unwrap().clone() };
        let idx_user_id = user_ids.iter().position(|&u| u == user_id1).unwrap();
        let user_id = user_ids.get(idx_user_id).unwrap().clone();
        let user_name = ChatMsgTest::user_names().get(idx_user_id).unwrap().clone();
        let msg = Some(msg.to_string());
        ChatMessage::new(id, stream_id, user_id, user_name, msg, date_update, false, false)
    }
    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    // get_num_token(profile1.user_id)
    pub fn get_num_token(user_id: i32) -> i32 {
        40000 + user_id
    }
    // get_token(config_jwt, profile1.user_id)
    pub fn get_token(config_jwt: config_jwt::ConfigJwt, user_id: i32) -> String {
        let num_token = get_num_token(user_id);
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        token
    }
    pub fn get_profiles(count: u8) -> (Vec<Profile>, Vec<Session>) {
        let mut session_vec: Vec<Session> = Vec::new();

        let cnt = if count < 5 { count } else { 4 };
        let mut profile_list: Vec<Profile> = Vec::new();
        for idx in 0..cnt {
            profile_list.push(create_profile(PROFILE_ID + i32::from(idx)));
        }
        let profile_vec = ProfileOrmApp::create(&profile_list).profile_vec;

        for profile in profile_vec.iter() {
            #[rustfmt::skip]
            session_vec.push(SessionOrmApp::new_session(profile.user_id, Some(get_num_token(profile.user_id))));
        }
        (profile_vec, session_vec)
    }
    pub fn get_chat_messages() -> (Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>) {
        let date_update: DateTime<Utc> = Utc::now();
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_ids = ChatMsgTest::user_ids();
        let user_id1 = user_ids.get(0).unwrap().clone();
        let user_id2 = user_ids.get(1).unwrap().clone();

        let chat_message1 = create_chat_message(1, stream_id, user_id1, "msg101", date_update);
        let chat_message2 = create_chat_message(2, stream_id, user_id2, "msg201", date_update);

        let chat_message_list: Vec<ChatMessage> = vec![chat_message1, chat_message2];
        let chat_message_log_list: Vec<ChatMessageLog> = Vec::new();
        let blocked_user_list: Vec<BlockedUser> = ChatMsgTest::get_blocked_user_vec();

        let chat_message_orm =
            ChatMessageOrmApp::create(&chat_message_list, &chat_message_log_list, &blocked_user_list);

        let chat_message_vec = chat_message_orm.chat_message_vec.clone();
        let mut chat_message_log_vec: Vec<ChatMessageLog> = Vec::new();

        for (_key, value_vec) in chat_message_orm.chat_message_log_map.iter() {
            for chat_message_log in value_vec {
                chat_message_log_vec.push(chat_message_log.clone());
            }
        }
        let blocked_user_vec = chat_message_orm.blocked_user_vec.clone();

        (chat_message_vec, chat_message_log_vec, blocked_user_vec)
    }
    #[rustfmt::skip]
    pub fn get_cfg_data(mode: i32) -> (config_jwt::ConfigJwt
    , (Vec<Profile>, Vec<Session>, Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>), String) {
        let config_jwt = config_jwt::get_test_config();
        let mut token = "".to_string();
        
        let mut profile_vec: Vec<Profile> = Vec::new();
        let mut session_vec: Vec<Session> = Vec::new();
        let mut chat_message_vec: Vec<ChatMessage> = Vec::new();
        let mut chat_message_log_vec: Vec<ChatMessageLog> = Vec::new();
        let mut blocked_user_vec: Vec<BlockedUser> = ChatMsgTest::get_blocked_user_vec();
        if mode > 0 {
            let count_user = if mode == 1 { 2 } else { 4 };
            let (profile_vec1, session_vec1) = get_profiles(count_user);
            profile_vec = profile_vec1;
            session_vec = session_vec1;
            
            let user_id1 = profile_vec.get(0).unwrap().user_id;
            
            token = get_token(config_jwt.clone(), user_id1);
        }
        // mode: 3
        if mode > 2 {
            let res_data = get_chat_messages();
            chat_message_vec = res_data.0;
            chat_message_log_vec = res_data.1;
            blocked_user_vec = res_data.2;
        }
        let cfg_c = config_jwt;
        let data_c = (profile_vec, session_vec, chat_message_vec, chat_message_log_vec, blocked_user_vec);
        (cfg_c, data_c, token)
    }
    pub fn configure_chat_message(
        cfg_c: config_jwt::ConfigJwt,
        data_c: (
            Vec<Profile>,
            Vec<Session>,
            Vec<ChatMessage>,
            Vec<ChatMessageLog>,
            Vec<BlockedUser>,
        ),
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            #[rustfmt::skip]
            let user_mini_vec: Vec<UserMini> = data_c.0.iter().map(|v| UserMini { id: v.user_id, name: v.nickname.clone() }).collect();
            let data_config_jwt = web::Data::new(cfg_c);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_chat_message_orm = web::Data::new(ChatMessageOrmApp::create(&data_c.2, &data_c.3, &data_c.4));
            let data_blocked_user_orm = web::Data::new(BlockedUserOrmApp::create(&data_c.4, &user_mini_vec));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_chat_message_orm))
                .app_data(web::Data::clone(&data_blocked_user_orm));
        }
    }
}

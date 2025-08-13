use actix_web::{get, web, HttpResponse};
use actix_web_actors::ws;
use utoipa;
use vrb_dbase::user_auth::config_jwt;
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_dbase::user_auth::user_auth_orm::impls::UserAuthOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_dbase::user_auth::user_auth_orm::tests::UserAuthOrmApp;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{chat_ws_assistant::ChatWsAssistant, chat_ws_session::ChatWsSession};

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /ws
            .service(get_ws_chat);
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
    user_auth_orm: web::Data<UserAuthOrmApp>,
    request: actix_web::HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse<actix_web::body::BoxBody>, actix_web::Error> {
    let config_jwt = config_jwt.get_ref().clone();
    let chat_message_orm_app = chat_message_orm.get_ref().clone();
    let user_auth_orm_app = user_auth_orm.get_ref().clone();
    #[rustfmt::skip]
    let assistant = ChatWsAssistant::new(
        config_jwt, chat_message_orm_app, user_auth_orm_app);

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

use std::{borrow::Cow, collections::HashMap, ops::Deref, time::Instant as tm};

use actix_web::{
    delete, get,
    http::StatusCode,
    post, put,
    web::{self, Query},
    HttpResponse,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use log::{error, info, log_enabled, Level::Info};
use serde_json::json;
use utoipa;
use vrb_common::{
    api_error::{code_to_str, ApiError},
    parser,
    validators::{msg_validation, Validator},
};
use vrb_dbase::db_enums::UserRole;
use vrb_tools::err;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{
    chat_message_models::{
        BlockedUser, BlockedUserDto, ChatMessage, ChatMessageDto, CreateBlockedUser, CreateBlockedUserDto, CreateChatMessage,
        CreateChatMessageDto, DeleteBlockedUser, DeleteBlockedUserDto, ModifyChatMessage, ModifyChatMessageDto, SearchChatMessage,
        SearchChatMessageDto, MESSAGE_MAX,
    },
    chat_message_orm::ChatMessageOrm,
};
use crate::extractors::authentication2::{Authenticated2, RequireAuth2};

// 403 Access denied - insufficient user rights.
pub const MSG_MODIFY_ANOTHER_USERS_CHAT_MESSAGE: &str = "modify_another_users_chat_message";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /api/chat_messages
            .service(get_chat_message)
            // POST /api/chat_messages
            .service(post_chat_message)
            // PUT /api/chat_messages/{id}
            .service(put_chat_message)
            // DELETE /api/chat_messages/{id}
            .service(delete_chat_message)
            // GET /api/blocked_users/{stream_id}
            .service(get_blocked_users)
            // POST /api/blocked_users
            .service(post_blocked_user)
            // DELETE /api/blocked_users
            .service(delete_blocked_user);
    }
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
            date_edt: None,
            date_rmv: None,
        });

        current = current + Duration::minutes(dlt_minutes);
        if !is_asc && idx == 0 {
            break;
        }
        idx = if is_asc { idx + 1 } else { idx - 1 };
    }
    result
}

// ** Section: ChatMessage **

/// get_chat_message
///
/// Get a list of messages from a chat (page by page).
///
/// Request structure:
/// ```text
/// {
///   streamId: number,         // required - chat ID (Stream ID);
///   isSortDes?: boolean,      // optional - descending sorting flag (default false);
///   minDate?: DateTime<Utc>,  // optional - minimum end date for chat message selection; 
///   maxDate?: DateTime<Utc>,  // optional - maximum end date of selection of chat messages;
///   limit?: number,           // optional - number of records on the page (20 by default);
/// }
/// ```
/// 
/// For "minDate" the result is strictly greater than the specified date.
/// For "maxDate" the result is strictly less than the specified date.
/// 
/// It is recommended to enter the date and time in ISO8601 format.
/// ```text
/// var d1 = new Date();
/// { minDate: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "minDate": "2020-01-20T22:10:57+02:00" }
/// ```
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1
/// ```
/// 
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&maxDate=2020-07-01T10:45:00.000Z&limit=20
/// ```
///
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&minDate=2020-07-01T11:10:00.000Z&limit=20
/// ```
/// Returns the found list of chat messages (Vec<`ChatMessageDto`>) with status 200.
///
/// The structure is returned:
/// ```text
/// [
///   {
///     id: Number,               // required - chat message ID;
///     date: DateTime<Utc>,      // required - date of the chat message;
///     member: String,           // required - nickname of the chat message user;
///     msg: String,              // required - chat message text;
///     dateEdt?: DateTime<Utc>,  // optional - the date the chat message text was last edited;
///     dateRmv?: DateTime<Utc>,  // optional - date the chat message was deleted;
///   }
/// ]
/// ```
/// 
/// Date and time are transmitted in ISO8601 format ("2020-01-20T20:10:57.000Z").
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The result is an array of chat messages.", body = Vec<ChatMessageDto>,
        examples(
            ("sort_ascending_part1" = (description = "Chat messages are sorted in ascending order, number of entries 4. 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&limit=4`",
                summary = "sort ascending part1", value = json!(get_ch_msgs(0, 3))
            )),
            ("sort_ascending_part2" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 203 (part 2). 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&maxDate=2020-07-01T10:45:00.000Z&limit=4`",
                summary = "sort ascending part2", value = json!(get_ch_msgs(4, 7))
            )),
            ("sort_ascending_part3" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 207 (part 3). 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&maxDate=2020-07-01T11:05:00.000Z&limit=4`",
                summary = "sort ascending part3", value = json!(get_ch_msgs(8, 11))
            )),
            ("sort_descending_part1" = (description = "Chat messages are sorted in descending order. 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&limit=4`",
                summary = "sort descending part1", value = json!(get_ch_msgs(11, 8))
            )),
            ("sort_descending_part2" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 2). 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&minDate=2020-07-01T11:10:00.000Z&limit=4`",
                summary = "sort descending part2", value = json!(get_ch_msgs(7, 4))
            )),
            ("sort_descending_part3" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 3). 
            `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&minDate=2020-07-01T10:50:00.000Z&limit=4`",
                summary = "sort descending part3", value = json!(get_ch_msgs(3, 0))
            )),
        ),
        ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 506, description = "Blocking error.", body = ApiError,
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError,
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/chat_messages", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn get_chat_message(
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    query_params: web::Query<SearchChatMessageDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get search parameters.
    let search_chat_message = SearchChatMessage::convert(query_params.into_inner());
    
    let chat_message_orm2 = chat_message_orm.get_ref().clone();

    let res_data = web::block(move || {
        // Find for an entity (stream event) by SearchStreamEvent.
        let res_data =
        chat_message_orm2.filter_chat_messages(search_chat_message)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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

/// post_chat_message
/// 
/// Create a new message in the chat.
/// 
/// Request structure:
/// ```text
/// {
///   streamId: Number,   // required - stream identifier;
///   msg: String,        // required - text of the new message;
/// }
/// ```
/// 
/// The minimum length of a new message is 1 character. 
/// The maximum length of a new message is 255 characters.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/chat_messages \
/// -d '{"streamId": 123, "msg": "mesage1"}' \
/// -H 'Content-Type: application/json'
/// ```
/// Returns the new message entity (`ChatMessageDto`) with status 200.
/// The new message is received by all active users of the chat in real time.
/// 
/// The structure is returned:
/// ```text
/// {
///   id: Number,               // required - chat message ID;
///   date: DateTime<Utc>,      // required - date of the chat message;
///   member: String,           // required - nickname of the chat message user;
///   msg: String,              // required - chat message text;
///   dateEdt?: DateTime<Utc>,  // optional - the date the chat message text was last edited;
///   dateRmv?: DateTime<Utc>,  // optional - date the chat message was deleted;
/// }
/// ```
/// 
/// Date and time are transmitted in ISO8601 format ("2020-01-20T20:10:57.000Z").
///
#[utoipa::path(
    responses(
        (status = 201, description = "The result of the request is a new chat message.", body = ChatMessageDto,
            example = json!(ChatMessageDto::from(
            ChatMessage::new(123, 98, 37, "emma_johnson".to_string(), Some("message1".to_string()), Utc::now(), None, None) )) ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, "streamId: 123, msg: \"message2\"")
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "streamId": 123, "msg": "message2" })) )),
        (status = 417, body = [ApiError], description =
            "Validation error. `curl -i -X POST http://localhost:8080/api/chat_messages 
            -d '{ \"streamId\": 123, \"msg\": \"\" }' -H 'Content-Type: application/json'`",
            example = json!(ApiError::validations(
                (CreateChatMessageDto { stream_id: 123, msg: "".to_string() }).validate().err().unwrap()) )),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[post("/api/chat_messages", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn post_chat_message(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateChatMessageDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let create_chat_message_dto: CreateChatMessageDto = json_body.into_inner();

    let stream_id = create_chat_message_dto.stream_id;
    let msg = create_chat_message_dto.msg.clone();
    
    let create_chat_message = CreateChatMessage::new(stream_id, user.id, &msg);

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2.create_chat_message(create_chat_message).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));
    
    if let Some(timer) = timer {
        info!("post_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Created().json(chat_message_dto)) // 201
    } else {
        let json = serde_json::json!({ "stream_id": stream_id, "msg": &msg });
        let msg = format!("stream_id: {}, msg: \"{}\"",  stream_id, &msg);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PARAMETER_UNACCEPTABLE, &msg);
        Err(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, &msg) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

fn message_max() -> String {
    (0..(MESSAGE_MAX + 1)).map(|_| 'a').collect()
}
/// put_chat_message
///
/// Update a chat message.
///
/// Request structure:
/// ```text
/// {
///   msg: String,              // required - chat message text;
/// }
/// ```
/// The minimum length of a new message is 1 character. 
/// The maximum length of a new message is 255 characters.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/chat_messages/123 \
/// -d '{"msg": "mesage2"}' \
/// -H 'Content-Type: application/json'
/// ```
/// Returns an entity with the corrected message (`ChatMessageDto`) with status 200.
/// The corrected message is received by all active users of the chat in real time.
/// 
/// The structure is returned:
/// ```text
/// {
///   id: Number,               // required - chat message ID;
///   date: DateTime<Utc>,      // required - date of the chat message;
///   member: String,           // required - nickname of the chat message user;
///   msg: String,              // required - chat message text;
///   dateEdt?: DateTime<Utc>,  // optional - the date the chat message text was last edited;
///   dateRmv?: DateTime<Utc>,  // optional - date the chat message was deleted;
/// }
/// ```
/// 
/// Date and time are transmitted in ISO8601 format ("2020-01-20T20:10:57.000Z").
///
/// A user with administrator rights can edit messages of other chat users.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/chat_messages/123?userId=3 \
/// -d '{"msg": "mesage2"}' \
/// -H 'Content-Type: application/json'
/// ```
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Update the current user profile with new data.", body = ChatMessageDto,
            examples(
            ("msg_current_user" = (summary = "Message of the current user.",
                description = "Update the current user's message. `curl -i -X PUT http://localhost:8080/api/chat_messages/123 
                -d '{\"msg\": \"mesage2\"} -H 'Content-Type: application/json'`",
                value = json!(ChatMessageDto::from(ChatMessage::new(123, 98, 37, "emma_johnson".to_string()
                    , Some("message2".to_string()), Utc::now() + Duration::minutes(-10), Some(Utc::now()), None) ) )
            )),
            ("msg_some_other_user" = (summary = "Message of some other user. (Admin)",
                description = "Update another user's message. `curl -i -X PUT http://localhost:8080/api/chat_messages/123?userId=30 
                -d '{\"msg\": \"mesage2\"} -H 'Content-Type: application/json'`",
                value = json!(ChatMessageDto::from(ChatMessage::new(123, 98, 30, "robert_brown".to_string()
                    , Some("message2".to_string()), Utc::now() + Duration::minutes(-10), Some(Utc::now()), None) ) )
            )) ),
        ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, "id: 123, user_id: 30, msg: \"message2\"")
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "id": 123, "user_id": 30,"msg": "message2" })) )),
        (status = 416, description = "Error parsing input parameter.", body = ApiError,
            examples(
            ("msg_current_user" = (summary = "Message of the current user.",
                description = "Error parsing input parameter. `curl -i -X PUT http://localhost:8080/api/chat_messages/123a 
                    -d '{\"msg\": \"mesage2\"} -H 'Content-Type: application/json'`",
                value = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                    , "`id` - invalid digit found in string (123a)"))    
            )),
            ("msg_some_other_user" = (summary = "Message of some other user. (Admin)", 
                description = "Error parsing input parameter. `curl -i -X PUT http://localhost:8080/api/chat_messages/123?userId=30a 
                    -d '{\"msg\": \"mesage2\"} -H 'Content-Type: application/json'`",
                value = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                    , "`userId` - invalid digit found in string (30a)"))
            )) ),
        ),
        (status = 417, body = [ApiError], description = format!("Validation error. 
            `curl -i -X PUT http://localhost:8080/api/chat_messages/123 
            -d '{{\"msg\": \"{}\"}}' -H 'Content-Type: application/json'`", message_max()),
            example = json!(ApiError::validations( (ModifyChatMessageDto { msg: message_max() }).validate().err().unwrap() ) )
        ),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("id", description = "Unique chat message ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[put("/api/chat_messages/{id}", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn put_chat_message(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<ModifyChatMessageDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();
    let mut user_id = user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = &format!("`{}` - {}", "id", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;
    
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let modify_chat_message_dto: ModifyChatMessageDto = json_body.into_inner();
    let msg = modify_chat_message_dto.msg.clone();
   
    let modify_chat_message = ModifyChatMessage::new(msg.clone());

    if user.role == UserRole::Admin {
        let query_params = Query::<HashMap<String, String>>::from_query(request.query_string()).unwrap();
        let user_id_str = query_params.get("userId").map(|v| v.clone()).unwrap_or("".to_string());
        if user_id_str.len() > 0 {
            user_id = parser::parse_i32(&user_id_str).map_err(|e| {
                let msg = &format!("`userId` - {}", &e);
                error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
                ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
            })?;    
        }
    }

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2
            .modify_chat_message(id, user_id.clone(), modify_chat_message)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string())
    })?;
    
    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));
    
    if let Some(timer) = timer {
        info!("put_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Ok().json(chat_message_dto)) // 200
    } else {
        let json = serde_json::json!({ "id": id, "user_id": user_id, "msg": msg });
        #[rustfmt::skip]
        let msg = format!("id: {}, user_id: {}, msg: \"{}\"", id, user_id, msg);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PARAMETER_UNACCEPTABLE, &msg);
        Err(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, &msg) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

/// delete_chat_message
///
/// Delete a message from a user with the specified ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/chat_messages/123
/// ```
///
/// Return the user message (`ChatMessageDto`) with status 200 or 406 if the user message is not found.
/// All active chat users receive information about the message deletion in real time.
///
/// The structure is returned:
/// ```text
/// {
///   id: Number,               // required - chat message ID;
///   date: DateTime<Utc>,      // required - date of the chat message;
///   member: String,           // required - nickname of the chat message user;
///   msg: String,              // required - chat message text;
///   dateEdt?: DateTime<Utc>,  // optional - the date the chat message text was last edited;
///   dateRmv?: DateTime<Utc>,  // optional - date the chat message was deleted;
/// }
/// ```
/// 
/// Date and time are transmitted in ISO8601 format ("2020-01-20T20:10:57.000Z").
/// 
/// A user with administrator rights can delete messages from other chat users.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/chat_messages/123?userId=3
/// ```
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Delete a message from a user with the specified ID.", body = ChatMessageDto,
            examples(
            ("msg_current_user" = (summary = "Message of the current user.",
                description = "Delete the current user's message. `curl -i -X DELETE http://localhost:8080/api/chat_messages/123`",
                value = json!(ChatMessageDto::from(ChatMessage::new(123, 98, 37, "emma_johnson".to_string()
                    , None, Utc::now() + Duration::minutes(-10), None, Some(Utc::now())) ) )
            )),
            ("msg_some_other_user" = (summary = "Message of some other user. (Admin)",
                description = "Delete another user's message. `curl -i -X DELETE http://localhost:8080/api/chat_messages/123?userId=30`",
                value = json!(ChatMessageDto::from(ChatMessage::new(123, 98, 30, "robert_brown".to_string()
                    , None, Utc::now() + Duration::minutes(-10), None, Some(Utc::now())) ) )
            )) ),
        ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        
        (status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, "id: 123, user_id: 30, msg: \"message2\"")
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "id": 123, "user_id": 30,"msg": "message2" })) )),
        (status = 416, description = "Error parsing input parameter.", body = ApiError,
            examples(
            ("msg_current_user" = (summary = "Message of the current user.",
                description = "Error parsing input parameter. `curl -i -X DELETE http://localhost:8080/api/chat_messages/123a`",
                value = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                    , "`id` - invalid digit found in string (123a)"))    
            )),
            ("msg_some_other_user" = (summary = "Message of some other user. (Admin)", 
                description = "Error parsing input parameter. `curl -i -X DELETE http://localhost:8080/api/chat_messages/123?userId=30a`",
                value = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                    , "`userId` - invalid digit found in string (30a)"))
            )) ),
        ),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("id", description = "Unique chat message ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/chat_messages/{id}", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn delete_chat_message(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();
    let mut user_id = user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = &format!("`id` - {}", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;
    
    if user.role == UserRole::Admin {
        let query_params = Query::<HashMap<String, String>>::from_query(request.query_string()).unwrap();
        let user_id1 = query_params.get("userId").map(|v| v.clone()).unwrap_or("".to_string());
        if user_id1.len() > 0 {
            user_id = parser::parse_i32(&user_id1).map_err(|e| {
                let msg = &format!("`userId` - {}", &e);
                error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
                ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
            })?;    
        }
    }

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm2
            .delete_chat_message(id, user_id)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_chat_message_dto = res_chat_message?.map(|v| ChatMessageDto::from(v));

    if let Some(timer) = timer {
        info!("delete_chat_message() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(chat_message_dto) = opt_chat_message_dto {
        Ok(HttpResponse::Ok().json(chat_message_dto)) // 200
    } else {
        let json = serde_json::json!({ "id": id, "user_id": user_id });
        #[rustfmt::skip]
        let message = format!("id: {}, user_id: {}", id, user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PARAMETER_UNACCEPTABLE, &message);
        Err(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, &message) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

// ** Section: BlockedUsers **

/// get_blocked_users
///
/// Get a list of blocked users for the specified stream.
/// This method is called by the stream owner to get a list of blocked users.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/blocked_users
/// ```
/// 
/// Returns the found list of blocked users (Vec<`BlockedUserDto`>) with status 200.
///
/// The structure is returned:
/// ```text
/// [
///   {
///     id: Number,                // required - record ID;
///     userId: Number,            // required - user ID, stream owner;
///     blockedId: Number,         // required - blocked user ID;
///     blockedNickname: String,   // required - nickname of the blocked user;
///     blockDate: DateTime<Utc>,  // required - date and time the user was blocked;
///   }
/// ]
/// ```
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Get a list of blocked users for the specified stream.", body = Vec<BlockedUserDto>,
            examples(
            ("1_blocked_users_present" = (summary = "blocked users are present", description = "There are blocked users.",
                value = json!(vec![
                    BlockedUserDto::from(BlockedUser::new(1, 12, 42, "mary_williams".to_string()
                        , Some(Utc::now() + Duration::minutes(-30)))),
                    BlockedUserDto::from(BlockedUser::new(1, 12, 48, "ava_wilson".to_string()
                        , Some(Utc::now() + Duration::minutes(-145)))), ])
            )),
            ("2_blocked_users_absent" = (summary = "blocked users are absent", description = "There are no blocked users.",
                value = json!([])
            ))),
        ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/blocked_users", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn get_blocked_users(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();
    let user_id = user.id;

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_blocked_users = web::block(move || {
        // Get a list of blocked users.
        let res_chat_message1 = chat_message_orm2
            .get_blocked_users(user_id)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let blocked_user_vec = res_blocked_users?;
    let blocked_user_dto_vec: Vec<BlockedUserDto> = blocked_user_vec.iter().map(|v| BlockedUserDto::from(v.clone())).collect();

    if let Some(timer) = timer {
        info!("get_blocked_users() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Ok().json(blocked_user_dto_vec)) // 200
}

/// post_blocked_user
/// 
/// Add user to blocked list.
/// 
/// Request structure:
/// ```text
/// {
///   blockedId?: Number,        // optional - user id to block;
///   blockedNickname?: String,  // optional - "nickname" of the user to block;
/// }
/// ```
/// 
/// One of the parameters "blockedId", "blockedNickname" must be present.
/// 
/// Returns the blocked user record (`BlockedUserDto`) with status 200 or 204 (no content) if the user is not found.
/// 
/// The structure is returned:
/// ```text
/// {
///   id: Number,                // required - record ID;
///   userId: Number,            // required - user ID, stream owner;
///   blockedId: Number,         // required - blocked user ID;
///   blockedNickname: String,   // required - nickname of the blocked user;
///   blockDate: DateTime<Utc>,  // required - date and time the user was blocked;
/// }
/// ```
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/blocked_users/ \
/// -d '{"blockedId": 42}' \
/// -H 'Content-Type: application/json'
/// ```
/// Add user to blocked list by user ID.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/blocked_users/ \
/// -d '{"blockedNickname": "mary_williams"}' \
/// -H 'Content-Type: application/json'
/// ```
/// Add user to the blocked list by user "nickname".
/// 
#[utoipa::path(
    responses(
        (status = 201, description = "Add user to blocked list.", body = BlockedUserDto,
            examples(
            ("1_add_by_user_id" = (summary = "Add user by user ID", 
                description = "Add user to blocked list by user ID. 
                `curl -i -X POST http://localhost:8080/api/blocked_users 
                -d '{\"blockedId\": 42} -H 'Content-Type: application/json'`",
                value = json!(BlockedUserDto::from(BlockedUser::new(1, 12, 42, "mary_williams".to_string()
                    , Some(Utc::now() + Duration::minutes(-30))))
            ))),
            ("2_add_by_nickname" = (summary = "Add user by user \"nickname\"", 
                description = "Add user to the blocked list by user \"nickname\".
                `curl -i -X POST http://localhost:8080/api/blocked_users 
                -d '{\"blockedNickname\": \"mary_williams\"} -H 'Content-Type: application/json'`",
                value = json!(BlockedUserDto::from(BlockedUser::new(1, 12, 42, "mary_williams".to_string()
                , Some(Utc::now() + Duration::minutes(-30))))
            )))),
        ),
        (status = 204, description = "The user with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X POST http://localhost:8080/api/blocked_users 
            -d '{} -H 'Content-Type: application/json'`",
            example = json!(ApiError::validations(
                (CreateBlockedUserDto { blocked_id: None, blocked_nickname: None }).validate().err().unwrap() ) )
        ),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[post("/api/blocked_users", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn post_blocked_user(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateBlockedUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();
    let user_id = user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }
    
    let create_blocked_user_dto: CreateBlockedUserDto = json_body.into_inner();
    let blocked_id = create_blocked_user_dto.blocked_id;
    let blocked_nickname = create_blocked_user_dto.blocked_nickname.clone();

    let create_blocked_user = CreateBlockedUser::new(user_id, blocked_id, blocked_nickname);

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_blocked_user = web::block(move || {
        // Add a new entity (blocked_user).
        let res_blocked_user1 = chat_message_orm2.create_blocked_user(create_blocked_user).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_blocked_user1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_blocked_user_dto = res_blocked_user?.map(|v| BlockedUserDto::from(v));

    if let Some(timer) = timer {
        info!("post_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(blocked_user_dto) = opt_blocked_user_dto {
        Ok(HttpResponse::Created().json(blocked_user_dto)) // 201
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_blocked_user
///
/// Remove user from blocked list.
///
/// Request structure:
/// ```text
/// {
///   blockedId?: Number,        // optional - user id to block;
///   blockedNickname?: String,  // optional - "nickname" of the user to block
/// }
/// ```
/// 
/// One of the parameters "blockedId", "blockedNickname" must be present.
/// 
/// Returns the blocked user record (`BlockedUserDto`) with status 200 or 204 (no content) if the user is not found.
/// 
/// The structure is returned:
/// ```text
/// {
///   id: Number,                // required - record ID;
///   userId: Number,            // required - user ID, stream owner;
///   blockedId: Number,         // required - blocked user ID;
///   blockedNickname: String,   // required - nickname of the blocked user;
///   blockDate: DateTime<Utc>,  // required - date and time the user was blocked;
/// }
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/blocked_users/ \
/// -d '{"blockedId": 42}' \
/// -H 'Content-Type: application/json'
/// ```
/// Remove user from blocked list by user ID.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/blocked_users/ \
/// -d '{"blockedNickname": "mary_williams"}' \
/// -H 'Content-Type: application/json'
/// ```
/// Remove user from blocked list by user "nickname".
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Remove user from blocked list.", body = ChatMessageDto,
            examples(
            ("1_remove_by_user_id" = (summary = "Remove user by user ID", 
                description = "Remove user from blocked list by user ID. 
                `curl -i -X DELETE http://localhost:8080/api/blocked_users 
                -d '{\"blockedId\": 42} -H 'Content-Type: application/json'`",
                value = json!(BlockedUserDto::from(BlockedUser::new(1, 12, 42, "mary_williams".to_string()
                    , Some(Utc::now() + Duration::minutes(-30))))
            ))),
            ("2_remove_by_nickname" = (summary = "Remove user by user \"nickname\"", 
                description = "Remove user from the blocked list by user \"nickname\".
                `curl -i -X DELETE http://localhost:8080/api/blocked_users 
                -d '{\"blockedNickname\": \"mary_williams\"} -H 'Content-Type: application/json'`",
                value = json!(BlockedUserDto::from(BlockedUser::new(1, 12, 42, "mary_williams".to_string()
                , Some(Utc::now() + Duration::minutes(-30))))
            )))),
        ),
        (status = 204, description = "The user with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X POST http://localhost:8080/api/blocked_users 
            -d '{} -H 'Content-Type: application/json'`",
            example = json!(ApiError::validations(
                (DeleteBlockedUserDto { blocked_id: None, blocked_nickname: None }).validate().err().unwrap() ) )
        ),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/blocked_users", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
pub async fn delete_blocked_user(
    authenticated: Authenticated2,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<DeleteBlockedUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let user = authenticated.deref();
    let user_id = user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }
    
    let delete_blocked_user_dto: DeleteBlockedUserDto = json_body.into_inner();
    let blocked_id = delete_blocked_user_dto.blocked_id;
    let blocked_nickname = delete_blocked_user_dto.blocked_nickname.clone();

    let delete_blocked_user = DeleteBlockedUser::new(user_id, blocked_id, blocked_nickname);

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_blocked_user = web::block(move || {
        // Add a new entity (blocked_user).
        let res_blocked_user1 = chat_message_orm2.delete_blocked_user(delete_blocked_user).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_blocked_user1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_blocked_user_dto = res_blocked_user?.map(|v| BlockedUserDto::from(v));

    if let Some(timer) = timer {
        info!("delete_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(blocked_user_dto) = opt_blocked_user_dto {
        Ok(HttpResponse::Ok().json(blocked_user_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::http;
    use vrb_common::api_error::ApiError;
    use vrb_tools::token_data::BEARER;

    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    pub fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
}

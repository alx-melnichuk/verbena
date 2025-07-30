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
use utoipa;
use vrb_dbase::db_enums::UserRole;
use vrb_tools::{
    api_error::{code_to_str, ApiError},
    err, parser,
    validators::{msg_validation, Validator},
};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::{
    chat_message_models::{
        BlockedUserDto, ChatMessageDto, CreateBlockedUser, CreateBlockedUserDto, CreateChatMessage, CreateChatMessageDto,
        DeleteBlockedUser, DeleteBlockedUserDto, ModifyChatMessage, ModifyChatMessageDto, SearchChatMessage, SearchChatMessageDto,
    },
    chat_message_orm::ChatMessageOrm,
};
use crate::extractors::authentication::{Authenticated, RequireAuth};

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

// ** Section: ChatMessage Get **

/// get_chat_message
///
/// Get a list of messages from a chat (page by page).
///
/// Request structure:
/// ```text
/// {
///   streamId: number,         // required
///   isSortDes?: boolean,      // optional
///   minDate?: DateTime<Utc>,  // optional
///   maxDate?: DateTime<Utc>,  // optional
///   limit?: number,           // optional
/// }
/// Where:
/// "streamId" - Chat ID (Stream ID).;
/// "isSortDes" - descending sorting flag (default false, i.e. default sorting is "ascending");
/// "minDate" - Minimum end date for chat message selection 
///             (result is strictly greater than the specified date);
/// "maxDate" - Maximum end date of selection of chat messages 
///             (result is strictly less than the specified date);
/// "limit" - number of records on the page (20 by default);
/// ```
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
#[utoipa::path(
    responses(
        (status = 200, description = "The result is an array of chat messages.", body = Vec<ChatMessageDto>,
        examples(
            ("sort_ascending_part1" = (description = "Chat messages are sorted in ascending order, number of entries 4. `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&limit=4`",
                summary = "sort ascending part1", value = json!(get_ch_msgs(0, 3))
            )),
            ("sort_ascending_part2" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 203 (part 2). `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&maxDate=2020-07-01T10:45:00.000Z&limit=4`",
                summary = "sort ascending part2", value = json!(get_ch_msgs(4, 7))
            )),
            ("sort_ascending_part3" = (description = "Chat messages are sorted in ascending order, number of entries 4, starting with ID > 207 (part 3). `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=false&maxDate=2020-07-01T11:05:00.000Z&limit=4`",
                summary = "sort ascending part3", value = json!(get_ch_msgs(8, 11))
            )),
            ("sort_descending_part1" = (description = "Chat messages are sorted in descending order. `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&limit=4`",
                summary = "sort descending part1", value = json!(get_ch_msgs(11, 8))
            )),
            ("sort_descending_part2" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 2). `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&minDate=2020-07-01T11:10:00.000Z&limit=4`",
                summary = "sort descending part2", value = json!(get_ch_msgs(7, 4))
            )),
            ("sort_descending_part3" = (description = "Chat messages are sorted in descending order, number of entries 4, starting with ID 203 (part 3). `curl -i -X GET http://localhost:8080/api/chat_messages?streamId=1&isSortDes=true&minDate=2020-07-01T10:50:00.000Z&limit=4`",
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
#[get("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
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

#[rustfmt::skip]
#[post("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateChatMessageDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let create_chat_message_dto: CreateChatMessageDto = json_body.into_inner();

    let stream_id = create_chat_message_dto.stream_id;
    let msg = create_chat_message_dto.msg.clone();
    
    let create_chat_message = CreateChatMessage::new(stream_id, user_id, &msg);

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

// PUT /api/chat_messages/{id}
#[rustfmt::skip]
#[put("/api/chat_messages/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<ModifyChatMessageDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let mut user_id = profile.user_id;

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

    if profile.role == UserRole::Admin {
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
            .modify_chat_message(id, user_id, modify_chat_message)
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
        let json = serde_json::json!({ "id": id, "user_id": profile.user_id, "msg": msg });
        #[rustfmt::skip]
        let msg = format!("id: {}, user_id: {}, msg: \"{}\"", id, profile.user_id, msg);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PARAMETER_UNACCEPTABLE, &msg);
        Err(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, &msg) // 406
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
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let mut user_id = profile.user_id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = &format!("`id` - {}", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;
    
    if profile.role == UserRole::Admin {
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
        let json = serde_json::json!({ "id": id, "user_id": profile.user_id });
        #[rustfmt::skip]
        let message = format!("id: {}, user_id: {}", id, profile.user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PARAMETER_UNACCEPTABLE, &message);
        Err(ApiError::create(406, err::MSG_PARAMETER_UNACCEPTABLE, &message) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json))
    }
}

#[rustfmt::skip]
#[get("/api/blocked_users/{stream_id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_blocked_users(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Get data from request.
    let stream_id_str = request.match_info().query("stream_id").to_string();
    let stream_id = parser::parse_i32(&stream_id_str).map_err(|e| {
        let msg = &format!("`stream_id` - {}", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;

    let chat_message_orm2 = chat_message_orm.get_ref().clone();
    let res_blocked_users = web::block(move || {
        // Get a list of blocked users.
        let res_chat_message1 = chat_message_orm2
            .get_blocked_users(user_id, stream_id)
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

#[rustfmt::skip]
#[post("/api/blocked_users", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_blocked_user(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateBlockedUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

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

#[rustfmt::skip]
#[delete("/api/blocked_users", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_blocked_user(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<DeleteBlockedUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

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

    use actix_web::{http, web};
    use chrono::{DateTime, Duration, Utc};
    use vrb_dbase::db_enums::UserRole;
    use vrb_tools::{api_error::ApiError, token_coding, token_data::BEARER};

    use crate::chats::{
        chat_message_models::{BlockedUser, ChatMessage, ChatMessageLog},
        chat_message_orm::tests::{ChatMessageOrmApp, ChatMsgTest, UserMini},
    };
    use crate::profiles::{
        config_jwt,
        profile_models::{Profile, Session},
        profile_orm::tests::ProfileOrmApp,
        profile_orm::tests::PROFILE_USER_ID as PROFILE_ID,
    };

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
    pub fn create_chat_message(id: i32, stream_id: i32, user_id: i32, msg: &str, date_created: DateTime<Utc>) -> ChatMessage {
        #[rustfmt::skip]
        let stream_id = if stream_id > 0 { stream_id } else { ChatMsgTest::stream_ids().get(0).unwrap().clone() };
        let user_ids = ChatMsgTest::user_ids().clone();
        #[rustfmt::skip]
        let user_id1 = if user_id > 0 { user_id } else { user_ids.get(0).unwrap().clone() };
        let idx_user_id = user_ids.iter().position(|&u| u == user_id1).unwrap();
        let user_id = user_ids.get(idx_user_id).unwrap().clone();
        let user_name = ChatMsgTest::user_names().get(idx_user_id).unwrap().clone();
        let msg = Some(msg.to_string());
        ChatMessage::new(id, stream_id, user_id, user_name, msg, date_created, None, None)
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
        let token = token_coding::encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
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
            session_vec.push(Session { user_id: profile.user_id, num_token: Some(get_num_token(profile.user_id)) });
        }
        (profile_vec, session_vec)
    }
    pub fn get_chat_messages(count_msg: i32) -> (Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>) {
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_ids = ChatMsgTest::user_ids();
        let user_id1 = user_ids.get(0).unwrap().clone();
        let user_id2 = user_ids.get(1).unwrap().clone();

        let mut date_created: DateTime<Utc> = Utc::now() - Duration::minutes(i64::try_from(count_msg + 2).unwrap());

        let mut chat_message_list: Vec<ChatMessage> = Vec::new();
        chat_message_list.push(create_chat_message(1, stream_id, user_id1, "msg101", date_created));
        date_created = date_created + Duration::minutes(1);
        chat_message_list.push(create_chat_message(2, stream_id, user_id2, "msg201", date_created));
        date_created = date_created + Duration::minutes(1);

        let chat_message_log_list: Vec<ChatMessageLog> = Vec::new();
        let blocked_user_list: Vec<BlockedUser> = ChatMsgTest::get_blocked_user_vec();

        let user_mini_vec: Vec<UserMini> = ChatMsgTest::get_user_mini();

        if count_msg > 2 {
            for idx in 1..=count_msg {
                let s1 = if idx < 10 { "0" } else { "" };
                let idx_s = format!("{}{}", s1, idx + 1);
                let msg = format!("msg1{}", &idx_s);
                let ch_msg = create_chat_message(idx, stream_id, user_id1, &msg, date_created);
                // eprintln!("ch_msg: {:#?}", ch_msg.clone());
                chat_message_list.push(ch_msg);
                date_created = date_created + Duration::minutes(1);
            }
        }

        let chat_message_orm = ChatMessageOrmApp::create(&chat_message_list, &chat_message_log_list, &blocked_user_list, &user_mini_vec);

        let chat_message_vec = chat_message_orm.chat_message_vec.clone();
        let mut chat_message_log_vec: Vec<ChatMessageLog> = Vec::new();

        for (_key, value_vec) in chat_message_orm.chat_message_log_map.iter() {
            for chat_message_log in value_vec {
                chat_message_log_vec.push(chat_message_log.clone());
            }
        }
        let blocked_user_vec = (*chat_message_orm.blocked_user_vec).borrow().clone();

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
        // mode: 3, 4
        if mode > 2 {
            let count_msg = if mode == 3 { 2 } else { 6 };
            let res_data = get_chat_messages(count_msg);
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
        data_c: (Vec<Profile>, Vec<Session>, Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>),
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            #[rustfmt::skip]
            let user_mini_vec: Vec<UserMini> = data_c.0.iter().map(|v| UserMini { id: v.user_id, name: v.nickname.clone() }).collect();
            let data_config_jwt = web::Data::new(cfg_c);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_chat_message_orm = web::Data::new(ChatMessageOrmApp::create(&data_c.2, &data_c.3, &data_c.4, &user_mini_vec.clone()));
            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_chat_message_orm));
        }
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

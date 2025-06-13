use std::{borrow::Cow, ops::Deref, time::Instant as tm};

use actix_files::NamedFile;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_actors::ws;
use log::{error, info, log_enabled, Level::Info};

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
        blocked_user_orm::tests::BlockedUserOrmApp,
        chat_message_models::{tests::ChatMessageTest, ChatMessage, ChatMessageLog},
        chat_message_orm::tests::ChatMessageOrmApp,
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
        let user_ids = ChatMessageTest::user_ids().clone();
        #[rustfmt::skip]
        let user_id1 = if user_id > 0 { user_id } else { user_ids.get(0).unwrap().clone() };
        let idx_user_id = user_ids.iter().position(|&u| u == user_id1).unwrap();
        let user_id = user_ids.get(idx_user_id).unwrap().clone();
        let nickname = ChatMessageTest::user_names().get(idx_user_id).unwrap().clone();
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
        let stream_id = if stream_id > 0 { stream_id } else { ChatMessageTest::stream_ids().get(0).unwrap().clone() };
        let user_ids = ChatMessageTest::user_ids().clone();
        #[rustfmt::skip]
        let user_id1 = if user_id > 0 { user_id } else { user_ids.get(0).unwrap().clone() };
        let idx_user_id = user_ids.iter().position(|&u| u == user_id1).unwrap();
        let user_id = user_ids.get(idx_user_id).unwrap().clone();
        let user_name = ChatMessageTest::user_names().get(idx_user_id).unwrap().clone();
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
    pub fn get_profiles() -> (Vec<Profile>, Vec<Session>) {
        let mut profile_vec: Vec<Profile> = Vec::new();
        let mut session_vec: Vec<Session> = Vec::new();

        let profile_list = &vec![create_profile(PROFILE_ID + 0), create_profile(PROFILE_ID + 1)];

        let profiles = ProfileOrmApp::create(&profile_list).profile_vec;

        // Create a profile for the 1st user.
        let profile1: Profile = profiles.get(0).unwrap().clone();
        #[rustfmt::skip]
        session_vec.push(SessionOrmApp::new_session(profile1.user_id, Some(get_num_token(profile1.user_id))));
        profile_vec.push(profile1);

        // Create a profile for the 2st user.
        let profile2: Profile = profiles.get(1).unwrap().clone();
        #[rustfmt::skip]
        session_vec.push(SessionOrmApp::new_session(profile2.user_id, Some(get_num_token(profile2.user_id))));
        profile_vec.push(profile2);

        (profile_vec, session_vec)
    }
    #[rustfmt::skip]
    pub fn get_cfg_data(mode: i32) -> (config_jwt::ConfigJwt
    , (Vec<Profile>, Vec<Session>, Vec<ChatMessage>, Vec<ChatMessageLog>, Vec<BlockedUser>), String) {
        let config_jwt = config_jwt::get_test_config();
        let mut token = "".to_string();
        let mut user_id1 = i32::default();
        let mut user_id2 = i32::default();
        
        let mut profile_vec: Vec<Profile> = Vec::new();
        let mut session_vec: Vec<Session> = Vec::new();
        let mut chat_message_vec: Vec<ChatMessage> = Vec::new();
        let mut chat_message_log_vec: Vec<ChatMessageLog> = Vec::new();
        let mut blocked_user_vec: Vec<BlockedUser> = Vec::new();
        // mode: 1
        if mode > 0 {
            let (profile_vec1, session_vec1) = get_profiles();
            profile_vec = profile_vec1;
            session_vec = session_vec1;
            
            user_id1 = profile_vec.get(0).unwrap().user_id;
            user_id2 = profile_vec.get(1).unwrap().user_id;
            
            token = get_token(config_jwt.clone(), user_id1);
        }
        // mode: 2
        if mode > 1 {
            let date_update: DateTime<Utc> = Utc::now();
            let chat_message1 = create_chat_message(1, -1, user_id1, "msg101", date_update);
            let chat_message2 = create_chat_message(2, -1, user_id2, "msg201", date_update);
    
            let chat_message_list: Vec<ChatMessage> = vec![chat_message1, chat_message2];
            let chat_message_log_list: Vec<ChatMessageLog> = Vec::new();
            let blocked_user_list: Vec<BlockedUser> = Vec::new();
    
            let chat_message_orm = ChatMessageOrmApp::create(&chat_message_list, &chat_message_log_list, &blocked_user_list);
    
            chat_message_vec = chat_message_orm.chat_message_vec.clone();
            
            for (_key, value_vec) in chat_message_orm.chat_message_log_map.iter() {
                for chat_message_log in value_vec {
                    chat_message_log_vec.push(chat_message_log.clone());
                }
            }
            blocked_user_vec = chat_message_orm.blocked_user_vec.clone();
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
            let data_config_jwt = web::Data::new(cfg_c);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_chat_message_orm = web::Data::new(ChatMessageOrmApp::create(&data_c.2, &data_c.3, &data_c.4));
            let data_blocked_user_orm = web::Data::new(BlockedUserOrmApp::create(&data_c.4, &[]));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_chat_message_orm))
                .app_data(web::Data::clone(&data_blocked_user_orm));
        }
    }
}

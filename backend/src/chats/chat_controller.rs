use std::ops::Deref;

use actix_files::NamedFile;
use actix_web::{get, post, put, web, HttpResponse, Responder};
use actix_web_actors::ws;

use crate::chats::chat_message_models::{
    ChatMessage, ChatMessageDto, CreateChatMessage, CreateChatMessageDto, ModifyChatMessage, ModifyChatMessageDto,
};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::chat_message_orm::ChatMessageOrm;
use crate::chats::chat_ws_session::ChatWsSession;
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
use crate::users::user_models::UserRole;
use crate::utils::parser;
use crate::utils::token::get_token_from_cookie_or_header;
use crate::validators::{msg_validation, Validator};

// 403 Access denied - insufficient user rights.
pub const MSG_MODIFY_ANOTHER_USERS_CHAT_MESSAGE: &str = "modify_another_users_chat_message";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /ws
            .service(get_ws_chat)
            // GET /chat
            .service(chat);
    }
}

#[get("/ws")]
pub async fn get_ws_chat(
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse<actix_web::body::BoxBody>, actix_web::Error> {
    // Attempt to extract token from cookie or authorization header
    let token = get_token_from_cookie_or_header(&request);

    eprintln!("~~get_ws_chat() jwt_token: {:?}", token);
    let chat_message_orm_app = chat_message_orm.get_ref().clone();

    let chat_ws_session = ChatWsSession::new(
        u64::default(),       // id: u64,
        Option::default(),    // room_id: Option<i32>,
        Option::default(),    // user_id: Option<i32>,
        Option::default(),    // user_name: Option<String>,
        bool::default(),      // is_blocked: bool,
        token,                // jwt_token: Option<String>,
        chat_message_orm_app, // chat_message_orm: ChatMessageOrmApp,
    );
    ws::start(chat_ws_session, &request, stream)
}

#[get("/chat")]
async fn chat() -> impl Responder {
    NamedFile::open_async("./static/chat_index.html").await.unwrap()
}

#[rustfmt::skip]
#[post("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    json_body: web::Json<CreateChatMessageDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;
    let _nickname = profile.nickname.clone();

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let create_chat_message_dto: CreateChatMessageDto = json_body.into_inner();

    let stream_id = create_chat_message_dto.stream_id;
    let msg = create_chat_message_dto.msg.clone();
    
    let create_chat_message = CreateChatMessage::new(stream_id, user_id, &msg);

    let chat_message2 = execute_create_chat_message(chat_message_orm.get_ref().clone(), create_chat_message).await?;

    let chat_message_dto = ChatMessageDto {
        id: chat_message2.id,
        stream_id: chat_message2.stream_id,
        user_id: chat_message2.user_id,
        msg: chat_message2.msg.unwrap_or("".to_string()),
        date_update: chat_message2.date_update.clone(),
        is_changed: chat_message2.is_changed,
        is_removed: chat_message2.is_removed,
        created_at: chat_message2.created_at.clone(),
        updated_at: chat_message2.updated_at.clone(),
    };

    Ok(HttpResponse::Created().json(chat_message_dto)) // 201
}

#[rustfmt::skip]
#[put("/api/chat_messages", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_chat_message(
    authenticated: Authenticated,
    chat_message_orm: web::Data<ChatMessageOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<ModifyChatMessageDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();
    let _nickname = profile.nickname.clone();

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;
    
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let modify_chat_message_dto: ModifyChatMessageDto = json_body.into_inner();

    let stream_id = modify_chat_message_dto.stream_id;
    let user_id = modify_chat_message_dto.user_id;
    let msg = modify_chat_message_dto.msg.clone();

    if let Some(user_id2) = user_id {
        if user_id2 != profile.user_id && profile.role != UserRole::Admin {
            let text = format!("curr_user_id: {}, user_id: {}", profile.user_id, user_id2);
            #[rustfmt::skip]
            let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_MODIFY_ANOTHER_USERS_CHAT_MESSAGE, &text);
            log::error!("{}: {}", err::CD_FORBIDDEN, &message);
            return Err(AppError::forbidden403(&message)); // 403
        }
    }
    
    let modify_chat_message = ModifyChatMessage::new(stream_id, user_id, msg);

    let opt_chat_message2 = execute_modify_chat_message(chat_message_orm.get_ref().clone(), id, modify_chat_message).await?;

    if let Some(chat_message2) = opt_chat_message2 {
        let chat_message_dto = ChatMessageDto {
            id: chat_message2.id,
            stream_id: chat_message2.stream_id,
            user_id: chat_message2.user_id,
            msg: chat_message2.msg.unwrap_or("".to_string()),
            date_update: chat_message2.date_update.clone(),
            is_changed: chat_message2.is_changed,
            is_removed: chat_message2.is_removed,
            created_at: chat_message2.created_at.clone(),
            updated_at: chat_message2.updated_at.clone(),
        };

        Ok(HttpResponse::Ok().json(chat_message_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204        
    }    
}

async fn execute_create_chat_message(
    chat_message_orm: ChatMessageOrmApp,
    create_chat_message: CreateChatMessage,
) -> Result<ChatMessage, AppError> {
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm.create_chat_message(create_chat_message).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let chat_message = res_chat_message?;

    Ok(chat_message)
}

async fn execute_modify_chat_message(
    chat_message_orm: ChatMessageOrmApp,
    id: i32,
    modify_chat_message: ModifyChatMessage,
) -> Result<Option<ChatMessage>, AppError> {
    let res_chat_message = web::block(move || {
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm.modify_chat_message(id, modify_chat_message).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_chat_message1
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_chat_message = res_chat_message?;

    Ok(opt_chat_message)
}

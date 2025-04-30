use actix_web::web;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;

use crate::chats::chat_message_models::{ChatMessage, CreateChatMessage};
use crate::chats::chat_message_orm::ChatMessageOrm;
use crate::errors::AppError;
use crate::settings::err;

pub async fn execute_create_chat_message(
    chat_message_orm: ChatMessageOrmApp,
    create_chat_message: CreateChatMessage,
) -> Result<ChatMessage, AppError> {
    eprintln!("@@@04 execute_create_chat_message()");
    let res_chat_message = web::block(move || {
        eprintln!("@@@04a execute_create_chat_message()");
        // Add a new entity (stream).
        let res_chat_message1 = chat_message_orm.create_chat_message(create_chat_message).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        eprintln!("@@@04b execute_create_chat_message()");
        res_chat_message1
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let chat_message = res_chat_message?;
    eprintln!("@@@04c execute_create_chat_message() chat_message: {:?}", chat_message);
    Ok(chat_message)
}

use actix_web::web::{Data, ServiceConfig};
use vrb_db::db::DbPool2;

pub mod chat_event_ws;
pub mod chat_message;
pub mod chat_message_controller;
pub mod chat_message_db;
pub mod chat_message_models;
pub mod chat_msg_blocked_test;
pub mod chat_msg_test_delete;
pub mod chat_msg_test_get;
pub mod chat_msg_test_post_put;
pub mod chat_ws_assistant;
pub mod chat_ws_async_result;
pub mod chat_ws_blck;
pub mod chat_ws_controller;
pub mod chat_ws_msg;
pub mod chat_ws_prm;
pub mod chat_ws_server;
pub mod chat_ws_session;
pub mod chat_ws_test_base;
pub mod chat_ws_test_blck;
pub mod chat_ws_test_msg;
pub mod chat_ws_test_prm;
pub mod chat_ws_tools;

/**
 * Adding data for the controller of this library.
 *   Addendum: ConfigJwt, ChatMessageDb.
 *   Use: ConfigJwt, UserDb.
 */
pub fn configure_data_and_server(db_pool: &DbPool2) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        // used: chat_message_controller, chat_ws_controller
        let chat_message_db = Data::new(chat_message_db::get_chat_message_db_app(db_pool));
        config.app_data(Data::clone(&chat_message_db));

        // Add configuration of internal services.
        config
            .configure(chat_message_controller::configure())
            .configure(chat_ws_controller::configure());
    }
}

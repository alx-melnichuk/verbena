use actix_files::NamedFile;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web_actors::ws;

use crate::chats::{chat_message_orm::impls::ChatMessageOrmApp, session::WsChatSession};

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
    let _chat_message_orm_app = chat_message_orm.get_ref().clone();
    let ws_chat_session = WsChatSession::new(
        u64::default(),
        String::default(),
        Option::default(),
        bool::default(),
        // Some(chat_message_orm_app),
    );

    ws::start(ws_chat_session, &request, stream)
}

#[get("/chat")]
async fn chat() -> impl Responder {
    NamedFile::open_async("./static/chat_index.html").await.unwrap()
}

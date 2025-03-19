use actix_files::NamedFile;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web_actors::ws;

use crate::chats::session::WsChatSession;

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
    request: actix_web::HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse<actix_web::body::BoxBody>, actix_web::Error> {
    ws::start(WsChatSession::default(), &request, stream)
    // WsChatSession::new(id: u64, room: String, name: Option<String>, is_blocked: bool)
}

#[get("/chat")]
async fn chat() -> impl Responder {
    NamedFile::open_async("./static/chat_index.html").await.unwrap()
}

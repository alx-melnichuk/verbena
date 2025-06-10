#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{self, /*body,*/ dev, test, App};
    // use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Timelike, Utc};
    // use serde_json;

    use crate::chats::chat_message_controller::{
        get_ws_chat,
        tests::{configure_chat_message, get_cfg_data, header_auth},
    };

    // ** get_stream_by_id **

    #[actix_web::test]
    async fn test_get_ws_chat_demo() {
        let (cfg_c, data_c, token) = get_cfg_data();

        // let stream_id = data_c.2.get(0).unwrap().id;
        // let stream_id_bad = format!("{}a", stream_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/ws")
            .insert_header(header_auth(&token)).to_request();
        let _resp: dev::ServiceResponse = test::call_service(&app, req).await;

        // let mut client = actix_web::client::ClientBuilder::new().uri("ws://localhost:8080/ws").unwrap();

        // let mut server = actix_web::test::run();
        // let mut client = client::ClientBuilder::new().uri("ws://localhost:8080/ws").unwrap();

        // get a client builder
        // let client = reqwest::Client::builder().default_headers(headers).build()?;
        // let res = client.get("https://www.rust-lang.org").send().await?;
    }
}

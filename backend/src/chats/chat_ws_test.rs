#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed

    use crate::chats::{
        /*blocked_user_models::BlockedUser,*/
        chat_message_controller::{
            get_ws_chat,
            tests::{configure_chat_message, get_cfg_data, get_profiles, get_token},
        },
        chat_message_orm::tests::ChatMsgTest,
        /*chat_message_models::{ChatMessage, ChatMessageLog},*/
    };
    /*use crate::profiles::profile_models::Profile;*/
    use crate::sessions::config_jwt;

    const URL_WS: &str = "/ws";

    // ** get_ws_chat **

    // * ews_echo *

    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(|| {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText("{ \"echo\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"err\":\"\\\"echo\\\" parameter not defined\"}")));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(|| {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText("{ \"echo\": \"text echo\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"echo\":\"text echo\"}")));
    }

    // * ews_name *

    #[actix_web::test]
    async fn test_get_ws_chat_ews_name_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText("{ \"name\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item,FrameText(Bytes::from_static(b"{\"err\":\"\\\"name\\\" parameter not defined\"}")));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_name() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText("{ \"name\": \"nickname\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item,FrameText(Bytes::from_static(b"{\"name\":\"nickname\",\"id\":0}")));
    }

    // * join *

    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_not_authorized_err_1() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText(format!("{{ \"join\": {} }}", i32::default()).into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item,FrameText(Bytes::from_static(b"{\"err\":\"\\\"join\\\" parameter not defined\"}")));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_not_authorized_err_2() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let stream_id = ChatMsgTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{} }}", stream_id).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item,FrameText(Bytes::from_static(b"{\"err\":\"This stream is not active.\"}")));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_not_authorized() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText("{ \"join\": 1 }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item,FrameText(Bytes::from_static(b"{\"join\":1,\"member\":\"\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}")));
    }
    //
    // #[actix_web::test]
    /*async fn test_get_ws_chat_ews_join_authorized() {
        let (profile_vec, _session_vec) = get_profiles();
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let config_jwt = config_jwt::get_test_config();
        let token = get_token(config_jwt, user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        eprintln!("stream_id1: {}", stream_id1);
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        eprintln!("item: {:?}", &item);
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
    }*/
}

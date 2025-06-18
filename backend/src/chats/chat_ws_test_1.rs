#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed

    use crate::sessions::config_jwt;
    use crate::{
        chats::{
            chat_message_controller::{
                get_ws_chat,
                tests::{configure_chat_message, get_cfg_data, get_profiles, get_token},
            },
            chat_message_orm::tests::ChatMsgTest,
            chat_ws_session as session,
        },
        utils::crypto::CRT_WRONG_STRING_BASE64URL,
    };

    const URL_WS: &str = "/ws";

    // ** get_ws_chat **

    // ** ews_echo **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo_err_not_defined() {
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
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "echo", session::PARAMETER_NOT_DEFINED))));
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

    // ** ews_name **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_name_err_not_defined() {
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
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "name", session::PARAMETER_NOT_DEFINED))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_name_ok() {
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
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"name\":\"nickname\"}")));
    }

    // ** ews_join **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_not_defined() {
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
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "join", session::PARAMETER_NOT_DEFINED))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_stream_not_active() {
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
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::STREAM_NOT_ACTIVE))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_not_auth_ok() {
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
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"join\":1,\"member\":\"\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}")));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_token() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        // let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}a\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err = format!("invalid_or_expired_token; {}", CRT_WRONG_STRING_BASE64URL);
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", err))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_dual_connect() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Attempt to dual connect.
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item2,FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_ALREADY_JOIN_TO_ROOM))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_stream_id_wrong() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id_wrong = ChatMsgTest::stream_ids().last().unwrap().clone() + 1;
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id_wrong, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::STREAM_WITH_SPECIFIED_ID_NOT_FOUND))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_err_session_not_found() {
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id3 = profile_vec.get(2).unwrap().user_id;
        // let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token3 = get_token(config_jwt::get_test_config(), user_id3);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, mut data_c, _token) = get_cfg_data(1);
            let (profile_vec, _session_vec) = get_profiles(4);
            data_c.0 = profile_vec;
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token3).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = format!(
            "{{\\\"code\\\":\\\"NotAcceptable\\\",\\\"message\\\":\\\"session_not_found; user_id: {}\\\"}}", user_id3);
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", err406))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ok_owner() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ok_not_owner() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":false}}", stream_id1, &member2);
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ok_blocked() {
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(2);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member4);
        assert_eq!(item, FrameText(Bytes::from(value)));
    }

    // ** ews_leave **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_leave_err_was_no_join() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText(format!("{{ \"leave\": 0 }}").into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_NO_JOIN))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_leave_ok() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave from the chat room.
        #[rustfmt::skip]
        let msg_text = MessageText("{ \"leave\": 0 }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item2, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":0}}", stream_id1, &member1))));
    }

    // ** ews_count **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_count_err_was_no_join() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_NO_JOIN))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_count_ok_one_user() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();
        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Leave user1.
        #[rustfmt::skip]
        let msg_text = MessageText("{ \"leave\": 0 }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item2, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":0}}", stream_id1, &member1))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_count_ok_two_users() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();
        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":false,\"is_blocked\":false}}", stream_id1, &member2);
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message for user 1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));

        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));

        // Message for user 1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));

        // Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":0}}", stream_id1, &member1))));
    }
}

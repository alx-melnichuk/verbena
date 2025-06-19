#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed

    use crate::sessions::config_jwt;
    use crate::settings::err;
    use crate::{
        chats::{
            chat_event_ws::ErrEWS,
            chat_message_controller::{
                get_ws_chat,
                tests::{configure_chat_message, get_cfg_data, get_profiles, get_token},
            },
            chat_message_orm::tests::ChatMsgTest,
            chat_ws_session::{get_err400, get_err401, get_err404, get_err406, get_err409},
        },
        utils::crypto::CRT_WRONG_STRING_BASE64URL,
    };

    const URL_WS: &str = "/ws";

    #[rustfmt::skip]
    pub fn err_str(err_ews: ErrEWS) -> String {
        format!("{{\"err\":{},\"code\":\"{}\",\"message\":\"{}\"}}", err_ews.err, err_ews.code, err_ews.message)
    }

    // ** get_ws_chat **

    // ** ews_echo, ews_name **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo_ews_name() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(|| {
            let (cfg_c, data_c, _token) = get_cfg_data(0);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "'echo' parameter not defined" --
        let msg_text = MessageText("{ \"echo\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = err_str(get_err400(&format!("'{}' {}", "echo", err::MSG_PARAMETER_NOT_DEFINED)));
        assert_eq!(item, FrameText(Bytes::from(err400)));

        // -- Test: "'echo' - ok" --
        let msg_text = MessageText("{ \"echo\": \"text echo\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"echo\":\"text echo\"}")));

        // -- Test: "'name' parameter not defined" --
        let msg_text = MessageText("{ \"name\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = err_str(get_err400(&format!("'{}' {}", "name", err::MSG_PARAMETER_NOT_DEFINED)));
        assert_eq!(item, FrameText(Bytes::from(err400)));

        // -- Test: "'name' - ok" --
        let msg_text = MessageText("{ \"name\": \"nickname\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"name\":\"nickname\"}")));
    }

    // ** ews_join, ews_leave **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ews_leave_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, mut data_c, _token) = get_cfg_data(1);
            let (profile_vec, _session_vec) = get_profiles(4);
            data_c.0 = profile_vec;
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "'join' parameter not defined" --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", i32::default()).into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = err_str(get_err400(&format!("'{}' {}", "join", err::MSG_PARAMETER_NOT_DEFINED)));
        assert_eq!(item, FrameText(Bytes::from(err400)));

        // -- Test: "This stream is not active." --
        let stream_id = ChatMsgTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{} }}", stream_id).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = err_str(get_err409(err::MSG_STREAM_NOT_ACTIVE));
        assert_eq!(item, FrameText(Bytes::from(err409)));

        let (profile_vec, _session_vec) = get_profiles(4);

        // -- Test: "Invalid token" --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}a\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401 = err_str(get_err401(&format!("invalid_or_expired_token; {}", CRT_WRONG_STRING_BASE64URL)));
        assert_eq!(item, FrameText(Bytes::from(err401)));

        // -- Test: "Stream with the specified id not found." --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id_wrong = ChatMsgTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id_wrong, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = err_str(get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong)));
        assert_eq!(item, FrameText(Bytes::from(err404)));

        // -- Test: "session_not_found" --
        let user_id3 = profile_vec.get(2).unwrap().user_id;
        let token3 = get_token(config_jwt::get_test_config(), user_id3);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();

        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token3).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = err_str(get_err406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user_id3)));
        assert_eq!(item, FrameText(Bytes::from(err406)));

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText(format!("{{ \"leave\": 0 }}").into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = err_str(get_err406(err::MSG_THERE_WAS_NO_JOIN));
        assert_eq!(item, FrameText(Bytes::from(err406)));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ews_leave_ok() {
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(2);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: Join user1 as owner. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: "There was already a 'join' to the room.". Trying to connect again. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = err_str(get_err409(err::MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM));
        assert_eq!(item, FrameText(Bytes::from(err409)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: Join user2 not authorized. --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream_id1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"\",\"count\":2,\"is_owner\":false,\"is_blocked\":true}}", stream_id1);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"\",\"count\":2}}", stream_id1))));
        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"\",\"count\":1}}", stream_id1))));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"\",\"count\":1}}", stream_id1))));

        // -- Test: Join user2 authorized. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":false,\"is_blocked\":false}}", stream_id1, &member2);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));
        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));

        // -- Test: Join user4 is blocked. --
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member4);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member4))));
        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member4))));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member4))));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":0}}", stream_id1, &member1))));
    }

    // ** ews_count **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_count() {
        let (profile_vec, _session_vec) = get_profiles(2);
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(1);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = format!(
            "{{\"err\":406,\"code\":\"NotAcceptable\",\"message\":\"{}\"}}", err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(err406)));

        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: Number of connected users. --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"count\":1}")));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user2.
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":false,\"is_blocked\":false}}", stream_id1, &member2);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));

        // -- Test: Number of connected users. --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"count\":2}")));

        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":1}}", stream_id1, &member2))));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"leave\":{},\"member\":\"{}\",\"count\":0}}", stream_id1, &member1))));
    }
}

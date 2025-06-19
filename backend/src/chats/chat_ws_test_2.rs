#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use chrono::{SecondsFormat, Utc};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed
    use serde_json;

    use crate::chats::{
        chat_event_ws::MsgEWS,
        chat_message_controller::{
            get_ws_chat,
            tests::{
                configure_chat_message, get_cfg_data, get_chat_messages, get_profiles, get_token, MSG_FAILED_DESER,
            },
        },
        chat_message_orm::tests::ChatMsgTest,
        chat_ws_session as session,
    };
    use crate::sessions::config_jwt;

    const URL_WS: &str = "/ws";
    const ERROR_PROCESSING_WS_FRAME_TEXT: &str = "Error processing websocket message Frame::Text(Bytes)";

    fn eq_msg_ews(msg_ews1: MsgEWS, msg_ews2: MsgEWS) -> bool {
        msg_ews1.msg == msg_ews2.msg
            && msg_ews1.id == msg_ews2.id
            && msg_ews1.member == msg_ews2.member
            && msg_ews1.date[0..20] == msg_ews2.date[0..20]
            && msg_ews1.is_edt == msg_ews2.is_edt
            && msg_ews1.is_rmv == msg_ews2.is_rmv
    }

    // ** get_ws_chat **

    // ** ews_msg **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(2);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "'msg' parameter not defined" --
        let msg_text = MessageText("{ \"msg\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "msg", session::PARAMETER_NOT_DEFINED))));

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msg\": \"text1\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_NO_JOIN))));

        // -- Test: "There is a block on sending messages." --
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member4);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msg\": \"text4\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_IS_BLOCK_ON_SEND))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_ok() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);

        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.last().unwrap().id + 1;
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
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

        // Message about join user 2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));

        // User1 sends a message.
        let msg = "text1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msg\": \"{}\" }}", &msg).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message from user1 to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, is_edt: false, is_rmv: false };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews2));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }

    // ** ews_msg_put **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_put_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "'msgPut' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "msgPut", session::PARAMETER_NOT_DEFINED))));

        // -- Test: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "id", session::PARAMETER_NOT_DEFINED))));

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_NO_JOIN))));

        // -- Test: "There is a block on sending messages." --
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member4);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgPut\": \"text4\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_IS_BLOCK_ON_SEND))));

        // -- Test: Editing another user's message. --
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.get(1).unwrap().id;

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text2\", \"id\": {} }}", ch_cmd_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}", session::MSG_CHAT_MESSAGE_NOT_FOUND, ch_cmd_id, user_id1);
        #[rustfmt::skip]
        let err404 = format!("{{\\\"code\\\":\\\"NotFound\\\",\\\"message\\\":\\\"{}\\\"}}", message);
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", err404))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_put_ok() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);

        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.first().unwrap().id;
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
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

        // Message about join user 2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));

        // User1 sends a message.
        let msg = "text2".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"{}\", \"id\": {} }}", &msg, ch_cmd_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message from user1 to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, is_edt: true, is_rmv: false };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews2));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }

    // ** ews_msg_cut **
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_cut_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: "'msgCut' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "id", session::PARAMETER_NOT_DEFINED))));

        // -- Test: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"'{}' {}\"}}", "id", session::PARAMETER_NOT_DEFINED))));

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_WAS_NO_JOIN))));

        // -- Test: "There is a block on sending messages." --
        let (profile_vec, _session_vec) = get_profiles(4);
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":1,\"is_owner\":false,\"is_blocked\":true}}", stream_id1, &member4);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", session::THERE_IS_BLOCK_ON_SEND))));

        // -- Test: Cutting another user's message. --
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.get(1).unwrap().id;

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = format!(
            "{{\"join\":{},\"member\":\"{}\",\"count\":2,\"is_owner\":true,\"is_blocked\":false}}", stream_id1, &member1);
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_cmd_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}", session::MSG_CHAT_MESSAGE_NOT_FOUND, ch_cmd_id, user_id1);
        #[rustfmt::skip]
        let err404 = format!("{{\\\"code\\\":\\\"NotFound\\\",\\\"message\\\":\\\"{}\\\"}}", message);
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"err\":\"{}\"}}", err404))));
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_cut_ok() {
        let (profile_vec, _session_vec) = get_profiles(2);
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);

        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.first().unwrap().id;
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
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

        // Message about join user 2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from(format!("{{\"join\":{},\"member\":\"{}\",\"count\":2}}", stream_id1, &member2))));

        // User1 sends a message.
        let msg = "".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_cmd_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message from user1 to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, is_edt: false, is_rmv: true };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = serde_json::from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews2));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use chrono::{SecondsFormat, Utc};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed
    use serde_json::{from_slice, to_string};

    use crate::chats::{
        chat_event_ws::{JoinEWS, MsgEWS},
        chat_message_controller::tests::{
            configure_chat_message, get_cfg_data, get_chat_messages, get_profiles, get_token, MSG_FAILED_DESER,
        },
        chat_message_orm::tests::ChatMsgTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_session::{get_err400, get_err403, get_err404, get_err406},
    };
    use crate::sessions::config_jwt;
    use crate::settings::err;

    const URL_WS: &str = "/ws";
    const ERROR_PROCESSING_WS_FRAME_TEXT: &str = "Error processing websocket message Frame::Text(Bytes)";

    fn eq_msg_ews(msg_ews1: MsgEWS, msg_ews2: MsgEWS) -> bool {
        msg_ews1.msg == msg_ews2.msg
            && msg_ews1.id == msg_ews2.id
            && msg_ews1.member == msg_ews2.member
            && msg_ews1.date[0..20] == msg_ews2.date[0..20]
            && msg_ews1.date_edt == msg_ews2.date_edt
            && msg_ews1.date_rmv == msg_ews2.date_rmv
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
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msg"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msg\": \"text1\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msg\": \"text4\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden
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

        let ch_cmd_id = get_chat_messages().0.last().unwrap().id + 1;
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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message about join user2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // User1 sends a message.
        let msg = "text1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msg\": \"{}\" }}", &msg).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, date_edt: None, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
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
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msgPut"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let (profile_vec, _session_vec) = get_profiles(4);

        // -- Test: "There is a block on sending messages." --
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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgPut\": \"text4\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: Editing another user's message. --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.get(1).unwrap().id; // Message user2.

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 2, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text2\", \"id\": {} }}", ch_cmd_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_cmd_id, user_id1));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_put_ok() {
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.first().unwrap().id;
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });

        let (profile_vec, _session_vec) = get_profiles(2);

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message about join user 2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream_id1, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // User1 sends a message.
        let msg = "text2".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"{}\", \"id\": {} }}", &msg, ch_cmd_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let date_edt = Some(date.clone());
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, date_edt, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
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

        // -- Test: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let (profile_vec, _session_vec) = get_profiles(4);

        // -- Test: "There is a block on sending messages." --
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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: Cutting another user's message. --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.get(1).unwrap().id; // Message user2.

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 2, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_cmd_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_cmd_id, user_id1));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_cut_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(3);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });

        let (profile_vec, _session_vec) = get_profiles(2);

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

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
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        let ch_msgs = get_chat_messages();
        let ch_cmd_id = ch_msgs.0.first().unwrap().id;

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message about join user2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream_id1, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // User1 sends a message.
        let msg = "".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_cmd_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let date_rmv = Some(date.clone());
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_cmd_id, member: member1.clone(), date, date_edt: None, date_rmv };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert!(eq_msg_ews(msg_ews_res, msg_ews2));
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }
}

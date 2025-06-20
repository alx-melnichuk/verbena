#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for send method in Framed
    use serde_json::to_string;

    use crate::chats::chat_event_ws::LeaveEWS;
    use crate::sessions::config_jwt;
    use crate::settings::err;
    use crate::{
        chats::{
            chat_event_ws::{CountEWS, JoinEWS},
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
        let err400 = get_err400(&format!("'{}' {}", "echo", err::MSG_PARAMETER_NOT_DEFINED));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

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
        let err400 = get_err400(&format!("'{}' {}", "name", err::MSG_PARAMETER_NOT_DEFINED));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

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

        let (profile_vec, _session_vec) = get_profiles(4);

        // -- Test: "'join' parameter not defined" --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", i32::default()).into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("'{}' {}", "join", err::MSG_PARAMETER_NOT_DEFINED));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: "Stream with the specified id not found." (unauthorized) --
        let stream_id_wrong = ChatMsgTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream_id_wrong).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: "Stream with the specified id not found." (authorized) --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id_wrong = ChatMsgTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id_wrong, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: "This stream is not active." (unauthorized) --
        let stream_id = ChatMsgTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{} }}", stream_id).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: "This stream is not active." (authorized) --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id = ChatMsgTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{}, \"access\": \"{}\"  }}", stream_id, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: "Invalid token" --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}a\" }}", stream_id1, token1).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401 = get_err401(&format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, CRT_WRONG_STRING_BASE64URL));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401).unwrap()))); // 401:Unauthorized

        // -- Test: "session_not_found" --
        let user_id3 = profile_vec.get(2).unwrap().user_id;
        let token3 = get_token(config_jwt::get_test_config(), user_id3);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token3).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = get_err406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user_id3));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: "There was no 'join' command." --
        let msg_text = MessageText(format!("{{ \"leave\": 0 }}").into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ews_leave_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let (cfg_c, data_c, _token) = get_cfg_data(2);
            App::new().service(get_ws_chat).configure(configure_chat_message(cfg_c, data_c))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = get_profiles(4);
        let stream_id1 = ChatMsgTest::stream_ids().get(0).unwrap().clone();

        // -- Test: Join user1 as owner. --
        let user_id1 = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = get_token(config_jwt::get_test_config(), user_id1);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: "There was already a 'join' to the room.". Trying to connect again. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: Join user2 unauthorized. --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream_id1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: "".into(), count: 2, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: "".into(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2. (Test: Leave unauthorized.)
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: "".into(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: "".into(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: Join user2 authorized. --
        let user_id2 = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = get_token(config_jwt::get_test_config(), user_id2);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2. (Test: Leave authorized.)
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: Join user4 is blocked. --
        let user_id4 = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = get_token(config_jwt::get_test_config(), user_id4);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id1, token4).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member4.clone(), count: 2, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream_id1, member: member4.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: member4.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: member1.clone(), count: 0 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
    }

    // ** ews_count **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_count() {
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
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let (profile_vec, _session_vec) = get_profiles(2);

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

        // -- Test: Number of connected users. --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&CountEWS { count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

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
        let value = to_string(&JoinEWS {
            join: stream_id1, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream_id1, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: Number of connected users. --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&CountEWS { count: 2 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream_id1, member: member1.clone(), count: 0 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
}

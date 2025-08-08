#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed.
    use serde_json::to_string;
    use vrb_common::crypto::CRT_WRONG_STRING_BASE64URL;
    use vrb_tools::err;

    use crate::chats::chat_event_ws::LeaveEWS;
    use crate::chats::{
        chat_event_ws::{CountEWS, JoinEWS},
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_session::{get_err400, get_err401, get_err404, get_err406, get_err409},
    };
    use crate::profiles::{
        config_jwt,
        profile_orm::tests::{ProfileOrmTest as ProflTest, USER, USER100_ID_NO_SESSION},
    };

    const URL_WS: &str = "/ws";

    // ** get_ws_chat **

    // ** ews_echo, ews_name **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo_ews_name() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(|| {
            let data_p = ProflTest::profiles(&[]);
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: 1. "'echo' parameter not defined" --
        let msg_text = MessageText("{ \"echo\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "echo"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "'echo' - ok" --
        let msg_text = MessageText("{ \"echo\": \"text echo\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        assert_eq!(item, FrameText(Bytes::from_static(b"{\"echo\":\"text echo\"}")));

        // -- Test: 3. "'name' parameter not defined" --
        let msg_text = MessageText("{ \"name\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "name"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 4. "'name' - ok" --
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
            let mut data_p = ProflTest::profiles(&[USER, USER]);
            let profile2 = data_p.0.get_mut(1).unwrap();
            profile2.user_id = USER100_ID_NO_SESSION;
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (mut profile_vec, _session_vec) = ProflTest::profiles(&[USER, USER]);
        let profile2 = profile_vec.get_mut(1).unwrap();
        profile2.user_id = USER100_ID_NO_SESSION;
        let user1_id = profile_vec.get(0).unwrap().user_id;
        let token1 = ProflTest::get_token(user1_id);

        // -- Test: 1. "'join' parameter not defined" --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", i32::default()).into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "join"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "Stream with the specified id not found." (unauthorized) --
        let stream_id_wrong = ChMesTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream_id_wrong).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 3. "Stream with the specified id not found." (authorized) --
        let stream_id_wrong = ChMesTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id_wrong, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 4. "This stream is not active." (unauthorized) --
        let stream3_id = ChMesTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{} }}", stream3_id).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: 5. "This stream is not active." (authorized) --
        let stream3_id = ChMesTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{}, \"access\": \"{}\"  }}", stream3_id, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: 6. "Invalid token" --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}a\" }}", stream1_id, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401 = get_err401(&format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, CRT_WRONG_STRING_BASE64URL));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401).unwrap()))); // 401:Unauthorized

        // -- Test: 7. "session_not_found" --
        let user2_id = profile_vec.get(1).unwrap().user_id;
        let token2 = ProflTest::get_token(user2_id);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = get_err406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user2_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 8. "There was no 'join' command." --
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
            let mut data_p = ProflTest::profiles(&[USER, USER, USER, USER]);
            let user2_id = data_p.0.get(1).unwrap().user_id;
            // Add session (num_token) for user2.
            data_p.1.get_mut(1).unwrap().num_token = Some(ProflTest::get_num_token(user2_id));
            let user4_id = data_p.0.get(3).unwrap().user_id;
            // Add session (num_token) for user4.
            data_p.1.get_mut(3).unwrap().num_token = Some(ProflTest::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = ProflTest::profiles(&[USER, USER, USER, USER]);

        // -- Test: 1. "Join user1 as owner."" --
        let user1_id = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = ProflTest::get_token(user1_id);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 2. "There was already a 'join' to the room.". Trying to connect again. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: 3. "Join user2 unauthorized."" --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream1_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 2, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2. (Test: Leave unauthorized.)
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: "".into(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: "".into(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 4. "Join user2 authorized."" --
        let user2_id = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = ProflTest::get_token(user2_id);
        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2. (Test: Leave authorized.)
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 5. "Join user4 is blocked."" --
        let user4_id = profile_vec.get(3).unwrap().user_id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = ProflTest::get_token(user4_id);
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member4.clone(), count: 2, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member4.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member4.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member1.clone(), count: 0 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
    }

    // ** ews_count **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_count() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_p = ProflTest::profiles(&[USER, USER]);
            let user2_id = data_p.0.get(1).unwrap().user_id;
            // Add session (num_token) for user2.
            data_p.1.get_mut(1).unwrap().num_token = Some(ProflTest::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: 1. "There was no 'join' command." --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = ProflTest::profiles(&[USER, USER]);

        let user1_id = profile_vec.get(0).unwrap().user_id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = ProflTest::get_token(user1_id);
        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 2. "Number of connected users."" --
        let msg_text = MessageText(format!("{{ \"count\": 0 }}").into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&CountEWS { count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        let user2_id = profile_vec.get(1).unwrap().user_id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = ProflTest::get_token(user2_id);
        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 3. "Number of connected users."" --
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
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Finally: Leave user1.
        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member1.clone(), count: 0 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
}

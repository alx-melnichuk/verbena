#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed.
    use serde_json::to_string;
    use vrb_authent::{
        config_jwt,
        user_mock::{UserMock, USER, USER1_ID},
        user_models::Session,
        user_orm::tests::UserOrmTest as User_Test,
    };
    use vrb_common::{crypto::CRT_WRONG_STRING_BASE64URL, err};
    use vrb_tools::token_coding;

    use crate::{
        chat_event_ws::{CountEWS, JoinEWS, LeaveEWS},
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_session::{get_err400, get_err401, get_err404, get_err406, get_err409},
    };

    const URL_WS: &str = "/ws";

    // ** get_ws_chat **

    // ** ews_echo, ews_name **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_echo_ews_name() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(|| {
            let data_u = UserMock::users(&[]);
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
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
            let mut data_u = UserMock::users(&[USER, USER]);
            let session1 = data_u.1.get(0).unwrap().clone();
            let user3_id = USER1_ID + 2;
            let session3 = Session::new(user3_id, Some(UserMock::get_num_token(user3_id + 1)));
            data_u.1 = vec![session1, session3];
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true

        let token1 = User_Test::get_token(USER1_ID);

        // -- Test: 1. "There was no 'join' command." --
        let msg_text = MessageText(format!("{{ \"leave\": 0 }}").into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 2. "'join' parameter not defined" --
        let msg_text = MessageText(format!("{{ \"join\": {} }}", i32::default()).into()); // 0
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "join"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3. "Stream with the specified id not found." (unauthorized) --
        let stream_id_wrong = ChMesTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream_id_wrong).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 4. "Stream with the specified id not found." (authorized) --
        let stream_id_wrong2 = ChMesTest::stream_ids().last().unwrap().clone() + 1;
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream_id_wrong2, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id_wrong2));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 5. "This stream is not active." (unauthorized) --
        let stream3a_id = ChMesTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{} }}", stream3a_id).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: 6. "This stream is not active." (authorized) --
        let stream3b_id = ChMesTest::stream_ids().get(2).unwrap().clone(); // live: false
        let msg_text = MessageText(format!("{{ \"join\":{}, \"access\": \"{}\"  }}", stream3b_id, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err409 = get_err409(err::MSG_STREAM_NOT_ACTIVE);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err409).unwrap()))); // 409:Conflict

        // -- Test: 7. "Invalid token" --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}a\" }}", stream1_id, token1.clone()).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401b = get_err401(&format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, CRT_WRONG_STRING_BASE64URL));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401b).unwrap()))); // 401(b):Unauthorized

        // -- Test: 8. "expired_token" --
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let num_token1b = UserMock::get_num_token(USER1_ID);
        let token1b = token_coding::encode_token(USER1_ID, num_token1b, &jwt_secret, -config_jwt.jwt_access).unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1b).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401b = get_err401(&format!("{}; ExpiredSignature", err::MSG_INVALID_OR_EXPIRED_TOKEN));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401b).unwrap()))); // 401(b):Unauthorized

        // -- Test: 9. "session_not_found" (session_non_exist) --
        let user2a_id = USER1_ID + 1;
        let token2a = User_Test::get_token(user2a_id);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2a).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err406 = get_err406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user2a_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 10. "unacceptable_token_num" (session found, but 'num_token' does not match) --
        let user3a_id = USER1_ID + 2;
        let token3a = User_Test::get_token(user3a_id);
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token3a).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401c = get_err401(&format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user3a_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401c).unwrap()))); // 401(c):Unauthorized

        // -- Test: 11. "unacceptable_token_id" (session found, and 'num_token' does match, but user exist) --
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let user3b_id = USER1_ID + 2;
        let num_token3b = UserMock::get_num_token(user3b_id + 1);
        let token3b = token_coding::encode_token(user3b_id, num_token3b, &jwt_secret, config_jwt.jwt_access).unwrap();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token3b).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err401d = get_err401(&format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_ID, user3b_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err401d).unwrap()))); // 401(d):Unauthorized
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_join_ews_leave_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user4.
            data_u.1.get_mut(3).unwrap().num_token = Some(UserMock::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER, USER, USER]);

        // -- Test: 1. "Join user1 as owner."" --
        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = User_Test::get_token(user1_id);
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
        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = User_Test::get_token(user2_id);
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
        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = User_Test::get_token(user4_id);
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
            let mut data_u = UserMock::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
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
        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER]);

        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = User_Test::get_token(user1_id);
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

        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = User_Test::get_token(user2_id);
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

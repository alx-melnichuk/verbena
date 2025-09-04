#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use chrono::{SecondsFormat, Utc};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed
    use serde_json::{from_slice, to_string};
    use vrb_authent::{
        config_jwt,
        user_mock::{UserMock, USER},
        user_orm::tests::UserOrmTest as User_Test,
    };
    use vrb_common::err;

    use crate::{
        chat_event_ws::{JoinEWS, MsgEWS},
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_session::{get_err400, get_err403, get_err404, get_err406},
    };

    const URL_WS: &str = "/ws";
    const ERROR_PROCESSING_WS_FRAME_TEXT: &str = "Error processing websocket message Frame::Text(Bytes)";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** get_ws_chat **

    // ** ews_msg **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER, USER, USER]);
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
        let mut framed = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER, USER, USER]);

        // -- Test: 1. "'msg' parameter not defined" --
        let msg_text = MessageText("{ \"msg\": \"\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msg"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "There was no 'join' command." --
        let msg_text = MessageText("{ \"msg\": \"text1\" }".into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true

        // -- Test: 3. "There is a block on sending messages." --
        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = User_Test::get_token(user4_id);

        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
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
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChMesTest::chat_messages(2);
        let ch_msg_id = data_cm.0.last().unwrap().id + 1;

        // -- Test: 1. "Add message from user1" --
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

        // Message about join user2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
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
        let msg_ews = MsgEWS { msg, id: ch_msg_id, member: member1.clone(), date, date_edt: None, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews2.msg);
            assert_eq!(msg_ews_res.id, msg_ews2.id);
            assert_eq!(msg_ews_res.member, msg_ews2.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews2.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }

    // ** ews_msg_put **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_put_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            data_u.1.get_mut(3).unwrap().num_token = Some(UserMock::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(2);
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
        let data_cm = ChMesTest::chat_messages(2);

        // -- Test: 1. "'msgPut' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msgPut"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3. "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgPut\": \"text1\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 4. "There is a block on sending messages." --
        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = User_Test::get_token(user4_id);
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgPut\": \"text4\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 5. Editing another user's message. --
        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = User_Test::get_token(user1_id);
        let ch_msg_id = data_cm.0.get(1).unwrap().id; // Message user2.

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member1.clone(), count: 2, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text2\", \"id\": {} }}", ch_msg_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id, user1_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_put_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChMesTest::chat_messages(2);
        let ch_msg1 = data_cm.0.first().unwrap().clone();

        // -- Test: 1. "Edit message user1." --
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

        // Message about join user 2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // User1 sends a message.
        let msg = "text2".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"{}\", \"id\": {} }}", &msg, ch_msg1.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = ch_msg1.date_created.to_rfc3339_opts(SecondsFormat::Millis, true);
        let date_edt = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg1.id, member: member1.clone(), date, date_edt, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[0..20], msg_ews.date_edt.unwrap()[0..20]);
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews2.msg);
            assert_eq!(msg_ews_res.id, msg_ews2.id);
            assert_eq!(msg_ews_res.member, msg_ews2.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews2.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews2.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[0..20], msg_ews2.date_edt.unwrap()[0..20]);
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }

    // ** ews_msg_cut **
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_cut_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            data_u.1.get_mut(3).unwrap().num_token = Some(UserMock::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER, USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChMesTest::chat_messages(2);

        // -- Test: 1. "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 3. "There is a block on sending messages." --
        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = User_Test::get_token(user4_id);
        // Join user4.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 4. Cutting another user's message. --
        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = User_Test::get_token(user1_id);
        let ch_msg_id = data_cm.0.get(1).unwrap().id; // Message user2.

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        // Join user1.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member1.clone(), count: 2, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));
        // Send message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_msg_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id, user1_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_cut_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserMock::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(UserMock::get_num_token(user2_id));
            data_u.1.get_mut(3).unwrap().num_token = Some(UserMock::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserMock::users(&[USER, USER, USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChMesTest::chat_messages(2);

        // -- Test: 1. "Cut message user1." --
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

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = User_Test::get_token(user2_id);
        let ch_msg = data_cm.0.first().unwrap().clone();

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Message about join user2.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // User1 sends a message.
        let msg = "".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_msg.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        let date = ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Millis, true);
        let date_rmv = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg.id, member: member1.clone(), date, date_edt: None, date_rmv };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[0..20], msg_ews.date_rmv.unwrap()[0..20]);
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews2.msg);
            assert_eq!(msg_ews_res.id, msg_ews2.id);
            assert_eq!(msg_ews_res.member, msg_ews2.member);
            assert_eq!(msg_ews_res.date[0..20], msg_ews2.date[0..20]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews2.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[0..20], msg_ews2.date_rmv.unwrap()[0..20]);
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }
    }
}

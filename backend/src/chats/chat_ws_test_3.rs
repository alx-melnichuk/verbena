#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{web::Bytes, App};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed
    use serde_json::to_string;
    use vrb_dbase::user_auth::{
        config_jwt,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, USER},
    };
    use vrb_tools::err;

    use crate::chats::{
        chat_event_ws::{BlockEWS, JoinEWS, LeaveEWS, MsgRmvEWS, UnblockEWS},
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_session::{get_err400, get_err403, get_err404, get_err406},
    };

    const URL_WS: &str = "/ws";

    // ** get_ws_chat **

    // ** ews_msg_rmv **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_rmv_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = User_Test::users(&[USER, USER, USER, USER]);
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user4.
            data_u.1.get_mut(3).unwrap().num_token = Some(User_Test::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = User_Test::users(&[USER, USER, USER, USER]);
        let data_cm = ChMesTest::chat_messages(2);

        // -- Test: 1. "'msgRmv' parameter not defined" --
        let msg_text = MessageText("{ \"msgRmv\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msgRmv"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgRmv\": 1 }".into());
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
        let msg_text = MessageText("{ \"msgRmv\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 4. Removing another user's message. --
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
        let msg_text = MessageText(format!("{{ \"msgRmv\": {} }}", ch_msg_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.

        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id, user1_id));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_rmv_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = User_Test::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(User_Test::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = User_Test::users(&[USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChMesTest::chat_messages(2);
        let ch_msg_id = data_cm.0.first().unwrap().id;

        // -- Test: 1. "Delete message user1."  --
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
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgRmv\": {} }}", ch_msg_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&MsgRmvEWS { msg_rmv: ch_msg_id }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));

        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));
    }

    // ** ews_block, ews_unblock **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_block_ews_unblock_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = User_Test::users(&[USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(User_Test::get_num_token(user2_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = User_Test::users(&[USER, USER, USER]);

        // -- Test: 1. "'id' parameter not defined" --
        let msg_text = MessageText("{ \"block\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "block"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        let msg_text = MessageText("{ \"unblock\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "unblock"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2. "There was no 'join' command." --
        let msg_text = MessageText("{ \"block\": \"user\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        let msg_text = MessageText("{ \"unblock\": \"user\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 3. "stream_owner_rights_missing" --
        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = User_Test::get_token(user2_id);

        // Join user2.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member2.clone(), count: 1, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Block user3.
        let member3 = profile_vec.get(2).unwrap().nickname.clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"block\": \"{}\" }}", member3.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_STREAM_OWNER_RIGHTS_MISSING);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // Unblock user3.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"unblock\": \"{}\" }}", member3.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_STREAM_OWNER_RIGHTS_MISSING);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_block_ews_unblock_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = User_Test::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(User_Test::get_num_token(user2_id));
            data_u.1.get_mut(3).unwrap().num_token = Some(User_Test::get_num_token(user4_id));
            let data_cm = ChMesTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = User_Test::users(&[USER, USER, USER, USER]);
        let stream1_id = ChMesTest::stream_ids().get(0).unwrap().clone(); // live: true

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

        // -- Test: 1. Unblocking user2 who has not blocked and is not in the chat. --
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"unblock\": \"{}\" }}", member2.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; blocked_nickname: '{}'", err::MSG_USER_NOT_FOUND, &member2));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 2. Blocking user2 who has not blocked and is not in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"block\": \"{}\" }}", member2.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&BlockEWS { block: member2.clone(), is_in_chat: false }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));

        // -- Test: 3. Unblocking user4 who was blocked and is not in the chat. --
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"unblock\": \"{}\" }}", member4.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&UnblockEWS { unblock: member4.clone(), is_in_chat: false }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));

        // -- Test: 4. Blocking user4 who was blocked and is not in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"block\": \"{}\" }}", member4.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&BlockEWS { block: member4.clone(), is_in_chat: false }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));

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

        // -- Test: 5. Unblocking user2 who has not blocked and is in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"unblock\": \"{}\" }}", member2.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; blocked_nickname: '{}'", err::MSG_USER_NOT_FOUND, &member2));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 6. Blocking user2 who has not blocked and is in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"block\": \"{}\" }}", member2.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&BlockEWS { block: member2.clone(), is_in_chat: true }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // Leave user2.
        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS { leave: stream1_id, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

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
        // Message to user1 about user4 joining.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member4.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 7. Unblocking user4 who was blocked and is in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"unblock\": \"{}\" }}", member4.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&UnblockEWS { unblock: member4.clone(), is_in_chat: true }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user4.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));

        // -- Test: 8. Blocking user4 who was blocked and is in the chat. --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"block\": \"{}\" }}", member4.clone()).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&BlockEWS { block: member4.clone(), is_in_chat: true }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message to user4.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
}

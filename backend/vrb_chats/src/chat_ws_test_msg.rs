#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{App, web::Bytes};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use chrono::{SecondsFormat, Utc};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed
    use serde_json::{from_slice, to_string};
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{USER, UserOrmTest},
    };
    use vrb_common::err;

    use crate::{
        chat_event_ws::{JoinEWS, LeaveEWS, MsgEWS, MsgRmvEWS },
        chat_message_orm::tests::ChatMessageOrmTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_tools::{get_err400, get_err403, get_err404, get_err406},
    };

    const URL_WS: &str = "/ws";
    const ERROR_PROCESSING_WS_FRAME_TEXT: &str = "Error processing websocket message Frame::Text(Bytes)";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** ews_msg, ews_msg_cut, ews_msg_put, ews_msg_rmv **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user2, user4.
            data_u.1.get_mut(1).unwrap().num_token = Some(config_jwt::tests::get_num_token(user2_id));
            data_u.1.get_mut(3).unwrap().num_token = Some(config_jwt::tests::get_num_token(user4_id));
            let data_cm = ChatMessageOrmTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        });

        let stream1_id = ChatMessageOrmTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        // -- Test: 1. ews_msg --

        // -- Test: 1.1. ews_msg: "'msg' parameter not defined" --
        let msg_text = MessageText("{ \"msg\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msg"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 1.2. ews_msg: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msg\": \"text_1_2\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 2. ews_msg_put --

        // -- Test: 2.1. ews_msg_put: "'msgPut' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msgPut"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.2. ews_msg_put: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text_2_2\", \"id\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.3. ews_msg_put: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text_2_2\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.4. ews_msg_put: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgPut\": \"text_2_2\", \"id\": -1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.5. ews_msg_put: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgPut\": \"text_2_4\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 3. ews_msg_cut --

        // -- Test: 3.1. ews_msg_cut: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"\", \"id\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3.2. ews_msg_cut: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"text_3_2\", \"id\": 0 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3.3. ews_msg_cut: "'id' parameter not defined" --
        let msg_text = MessageText("{ \"msgCut\": \"text_3_3\", \"id\": -1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "id"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3.4. ews_msg_cut: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgCut\": \"text_3_4\", \"id\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 4. ews_msg_rmv --

        // -- Test: 4.1. ews_msg_rmv: "'msgRmv' parameter not defined" --
        let msg_text = MessageText("{ \"msgRmv\": \"\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "msgRmv"));
        assert_eq!(item, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 4.2. ews_msg_rmv: "There was no 'join' command." --
        let msg_text = MessageText("{ \"msgRmv\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable


        // == Join user4 authorized (is blocked). ==

        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = config_jwt::tests::get_token(user4_id);
        let ch_msg_id_last = data_cm.0.last().unwrap().id;

        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value)));
        
        // -- Test: 1. ews_msg --

        // -- Test: 1.3. ews_msg: "Sending "msg" to authorized users (but blocked)." --
        let msg_text = MessageText("{ \"msg\": \"text_1_3\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 2. ews_msg_put --

        // -- Test: 2.6. ews_msg_put: "Sending "msgPut" to authorized users (but blocked)." --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text_2_6\", \"id\": {} }}", ch_msg_id_last).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 3. ews_msg_cut --

        // -- Test: 3.5. ews_msg_cut: "Sending "msgCut" to authorized users (but blocked)." --
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"text_3_5\", \"id\": {} }}", ch_msg_id_last).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 4. ews_msg_rmv --

        // -- Test: 4.3. ews_msg_rmv: "Sending "msgRmv" to authorized users (but blocked)." --
        let msg_text = MessageText("{ \"msgRmv\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden


        // == Leave user4 authorized (is blocked). ==

        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member4.clone(), count: 0 }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));

        // == Join user3 unauthorized. ==

        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream1_id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        // Message for user3.
        let item3 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item3, FrameText(Bytes::from(value)));

        // -- Test: 1. ews_msg --

        // -- Test: 1.4. ews_msg: "Sending "msg" is blocked for unauthorized users."
        let msg_text = MessageText("{ \"msg\": \"text_1_4\" }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item3 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item3, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden
    
        // -- Test: 2. ews_msg_put --

        // -- Test: 2.7. ews_msg_put: "Sending "msgPut" is blocked for unauthorized users." --
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text_2_7\", \"id\": {} }}", ch_msg_id_last).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item3 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item3, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 3. ews_msg_cut --

        // -- Test: 3.6. ews_msg_put: "Sending "msgCut" is blocked for unauthorized users." --
        let msg_text = MessageText(format!("{{ \"msgCut\": \"text_3_6\", \"id\": {} }}", ch_msg_id_last).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item3 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item3, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // -- Test: 4. ews_msg_rmv --

        // -- Test: 4.3. ews_msg_rmv: "Sending "msgRmv" is blocked for unauthorized users." --
        let msg_text = MessageText("{ \"msgRmv\": 1 }".into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden


        // == Leave user3 unauthorized. ==

        #[rustfmt::skip]
        framed1.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: "".into(), count: 0 }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));

        // == Join user2 authorized. (is not blocked) ==

        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = config_jwt::tests::get_token(user2_id);

        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member2.clone(), count: 1, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item2, FrameText(Bytes::from(value)));

        // -- Test: 2. ews_msg_put --

        // -- Test: 2.8. ews_msg_put: "Sending "msgPut" is not blocked for authorized users." --
        let ch_msg_id2 = ch_msg_id_last + 1;
        let msg_text = MessageText(format!("{{ \"msgPut\": \"text_2_8\", \"id\": {} }}", ch_msg_id2).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id2, user2_id));
        assert_eq!(item2, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 3. ews_msg_cut --

        // -- Test: 3.7. ews_msg_cut: "Sending "msgCut" is not blocked for authorized users." --
        let ch_msg_id3 = ch_msg_id_last + 1;
        let msg_text = MessageText(format!("{{ \"msgCut\": \"text_3_7\", \"id\": {} }}", ch_msg_id3).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id3, user2_id));
        assert_eq!(item2, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound

        // -- Test: 4. ews_msg_rmv --

        // -- Test: 4.3. ews_msg_rmv: "Sending "msgRmv" is not blocked for authorized users." --
        let ch_msg_id4 = ch_msg_id_last + 1;
        let msg_text = MessageText(format!("{{ \"msgRmv\": {} }}", ch_msg_id4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        let item2 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let err404 = get_err404(&format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, ch_msg_id4, user2_id));
        assert_eq!(item2, FrameText(Bytes::from(to_string(&err404).unwrap()))); // 404:NotFound
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_msg_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserOrmTest::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(config_jwt::tests::get_num_token(user2_id));
            let data_cm = ChatMessageOrmTest::chat_messages(2);
            App::new()
                .service(get_ws_chat)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        });

        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let (profile_vec, _session_vec) = UserOrmTest::users(&[USER, USER]);
        let stream1_id = ChatMessageOrmTest::stream_ids().get(0).unwrap().clone(); // live: true
        let data_cm = ChatMessageOrmTest::chat_messages(2);


        // == Join user1 authorized. (is not blocked) ==

        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = config_jwt::tests::get_token(user1_id);

        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        // Message for user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();


        // == Join user2 authorized. (is not blocked) ==

        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = config_jwt::tests::get_token(user2_id);

        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        // Message for user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member2.clone(), count: 2, is_owner: Some(false), is_blocked: Some(false) }).unwrap();
        assert_eq!(item2, FrameText(Bytes::from(value)));
        // Message about join user 2.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: member2.clone(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value)));

        // -- Test: 1.1. ews_msg: Send a message of type "msg". (authorized)  --
        
        let ch_msg_id1 = data_cm.0.last().unwrap().id + 1;

        // User1 sends a message.
        let msg = "text1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msg\": \"{}\" }}", &msg).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        let msg_ews1 = MsgEWS { msg, id: ch_msg_id1, member: member1.clone(), date, date_edt: None, date_rmv: None };

        let msg_ews2 = msg_ews1.clone();
        if let FrameText(buf) = item1 {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews1.msg);
            assert_eq!(msg_ews_res.id, msg_ews1.id);
            assert_eq!(msg_ews_res.member, msg_ews1.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews1.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews1.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews1.date_rmv.is_none());
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
            assert_eq!(msg_ews_res.date[0..19], msg_ews2.date[0..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 2.1. ews_msg: Send a message of type "msgPut". (authorized)  --

        let ch_msg1 = data_cm.0.first().unwrap().clone();        

        // User1 sends a message.
        let msg = "text_2_1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"{}\", \"id\": {} }}", &msg, ch_msg1.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = ch_msg1.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_edt = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg1.id, member: member1.clone(), date, date_edt, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[..19], msg_ews.date_edt.unwrap()[..19]);
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
            assert_eq!(msg_ews_res.date[0..19], msg_ews2.date[0..19]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews2.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[0..19], msg_ews2.date_edt.unwrap()[0..19]);
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 3.1. ews_msg: Send a message of type "msgCut". (authorized)  --

        let ch_msg2 = data_cm.0.first().unwrap().clone();        

        // User1 sends a message.
        let msg = "".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_msg2.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = ch_msg2.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_rmv = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg2.id, member: member1.clone(), date, date_edt: None, date_rmv };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[..19], msg_ews.date_rmv.unwrap()[..19]);
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
            assert_eq!(msg_ews_res.date[..19], msg_ews2.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews2.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[..19], msg_ews2.date_rmv.unwrap()[..19]);
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 4.1. ews_msg: Send a message of type "msgRmv". (authorized)  --

        let ch_msg_id2 = data_cm.0.first().unwrap().id;     

        // User1 sends a message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgRmv\": {} }}", ch_msg_id2).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.
        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&MsgRmvEWS { msg_rmv: ch_msg_id2 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));


        // == Leave user2 authorized (is not blocked). ==

        #[rustfmt::skip]
        framed2.send(MessageText("{ \"leave\": 0 }".into())).await.unwrap(); // Send a message to a websocket.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&LeaveEWS {leave: stream1_id, member: member2.clone(), count: 1 }).unwrap();
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message to user1 about user2 leaving.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));


        // == Join user3 unauthorized. (is not blocked) ==

        let msg_text = MessageText(format!("{{ \"join\": {} }}", stream1_id).into());
        framed2.send(msg_text).await.unwrap(); // Send a message to a websocket.
        // Message for user3.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 2, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item2, FrameText(Bytes::from(value)));
        // Message to user1 about user2 joining.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 2, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));

        // -- Test: 1.1. ews_msg: Send a message of type "msg". (unauthorized)  --

        let ch_msg_id3 = data_cm.0.last().unwrap().id + 1;

        // User1 sends a message.
        let msg = "text1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msg\": \"{}\" }}", &msg).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        let msg_ews1 = MsgEWS { msg, id: ch_msg_id3, member: member1.clone(), date, date_edt: None, date_rmv: None };
        let msg_ews2 = msg_ews1.clone();
        if let FrameText(buf) = item1 {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews1.msg);
            assert_eq!(msg_ews_res.id, msg_ews1.id);
            assert_eq!(msg_ews_res.member, msg_ews1.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews1.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews1.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews1.date_rmv.is_none());
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
            assert_eq!(msg_ews_res.date[0..19], msg_ews2.date[0..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 2.1. ews_msg: Send a message of type "msgPut". (unauthorized)  --

        let ch_msg3 = data_cm.0.first().unwrap().clone();        

        // User1 sends a message.
        let msg = "text_2_1".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgPut\": \"{}\", \"id\": {} }}", &msg, ch_msg3.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = ch_msg3.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_edt = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg3.id, member: member1.clone(), date, date_edt, date_rmv: None };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[..19], msg_ews.date_edt.unwrap()[..19]);
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
            assert_eq!(msg_ews_res.date[0..19], msg_ews2.date[0..19]);
            assert_eq!(msg_ews_res.date_edt.is_some(), msg_ews2.date_edt.is_some());
            assert_eq!(msg_ews_res.date_edt.unwrap()[0..19], msg_ews2.date_edt.unwrap()[0..19]);
            assert_eq!(msg_ews_res.date_rmv.is_none(), msg_ews2.date_rmv.is_none());
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 3.1. ews_msg: Send a message of type "msgCut". (unauthorized)  --

        let ch_msg4 = data_cm.0.first().unwrap().clone();        

        // User1 sends a message.
        let msg = "".to_string();
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgCut\": \"\", \"id\": {} }}", ch_msg4.id).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let date = ch_msg4.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_rmv = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        #[rustfmt::skip]
        let msg_ews = MsgEWS { msg, id: ch_msg4.id, member: member1.clone(), date, date_edt: None, date_rmv };
        let msg_ews2 = msg_ews.clone();
        if let FrameText(buf) = item {
            let msg_ews_res: MsgEWS = from_slice(&buf).expect(MSG_FAILED_DESER);
            assert_eq!(msg_ews_res.msg, msg_ews.msg);
            assert_eq!(msg_ews_res.id, msg_ews.id);
            assert_eq!(msg_ews_res.member, msg_ews.member);
            assert_eq!(msg_ews_res.date[..19], msg_ews.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[..19], msg_ews.date_rmv.unwrap()[..19]);
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
            assert_eq!(msg_ews_res.date[..19], msg_ews2.date[..19]);
            assert_eq!(msg_ews_res.date_edt.is_none(), msg_ews2.date_edt.is_none());
            assert_eq!(msg_ews_res.date_rmv.is_some(), msg_ews2.date_rmv.is_some());
            assert_eq!(msg_ews_res.date_rmv.unwrap()[..19], msg_ews2.date_rmv.unwrap()[..19]);
        } else {
            panic!("{}", ERROR_PROCESSING_WS_FRAME_TEXT);
        }

        // -- Test: 4.1. ews_msg: Send a message of type "msgRmv". (authorized)  --

        let ch_msg_id4 = data_cm.0.first().unwrap().id;     

        // User1 sends a message.
        #[rustfmt::skip]
        let msg_text = MessageText(format!("{{ \"msgRmv\": {} }}", ch_msg_id4).into());
        framed1.send(msg_text).await.unwrap(); // Send a message to a websocket.

        // Message to user1.
        let item = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let value = to_string(&MsgRmvEWS { msg_rmv: ch_msg_id4 }).unwrap();
        assert_eq!(item, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user2.
        let item = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item, FrameText(Bytes::from(value)));
    }
}
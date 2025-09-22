#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{App, web::Bytes};
    use actix_web_actors::ws::{Frame::Text as FrameText, Message::Text as MessageText};
    use futures_util::{SinkExt, StreamExt}; // this is needed for "send" method in Framed
    use serde_json::to_string;
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{USER, UserOrmTest},
    };
    use vrb_common::err;

    use crate::{
        chat_event_ws::{JoinEWS, PrmBoolEWS, PrmIntEWS, PrmStrEWS},
        chat_message_orm::tests::ChatMessageOrmTest,
        chat_ws_controller::get_ws_chat,
        chat_ws_tools::{get_err400, get_err403, get_err406},
    };

    const URL_WS: &str = "/ws";


    // ** ews_prm_bool, ews_prm_int, ews_prm_str **

    #[actix_web::test]
    async fn test_get_ws_chat_ews_prm_err() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
            let user4_id = data_u.0.get(3).unwrap().id;
            // Add session (num_token) for user4.
            data_u.1.get_mut(3).unwrap().num_token = Some(config_jwt::tests::get_num_token(user4_id));
            let data_cm = ChatMessageOrmTest::chat_messages(0);
            App::new()
                .service(get_ws_chat)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        });
        // Open a websocket connection to the test server.
        let mut framed1 = srv.ws_at(URL_WS).await.unwrap();

        let stream1_id = ChatMessageOrmTest::stream_ids().get(0).unwrap().clone(); // live: true
        let (profile_vec, _session_vec) = UserOrmTest::users(&[USER, USER, USER, USER]);

        // -- Test: 1.1. "'prmBool' parameter not defined" --
        let prm_text = MessageText("{ \"prmBool\": \"\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "prmBool"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 1.2. "'valBool' parameter not defined" --
        let prm_text = MessageText("{ \"prmBool\": \"param1_2\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "valBool"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 1.3. "There was no 'join' command." --
        let prm_text = MessageText("{ \"prmBool\": \"param1_3\", \"valBool\": \"\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "valBool"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 1.4. "There was no 'join' command." --
        let prm_text = MessageText("{ \"prmBool\": \"param1_4\", \"valBool\": false }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 2.1. "'prmInt' parameter not defined" --
        let prm_text = MessageText("{ \"prmInt\": \"\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "prmInt"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.2. "'valInt' parameter not defined" --
        let prm_text = MessageText("{ \"prmInt\": \"param2_2\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "valInt"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.3. "There was no 'join' command." --
        let prm_text = MessageText("{ \"prmInt\": \"param2_3\", \"valInt\": \"\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "valInt"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 2.4. "There was no 'join' command." --
        let prm_text = MessageText("{ \"prmInt\": \"param2_4\", \"valInt\": 1 }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err406 = get_err406(err::MSG_THERE_WAS_NO_JOIN);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err406).unwrap()))); // 406:NotAcceptable

        // -- Test: 3.1. "'prmStr' parameter not defined" --
        let prm_text = MessageText("{ \"prmStr\": \"\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "prmStr"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- Test: 3.2. "'valStr' parameter not defined" --
        let prm_text = MessageText("{ \"prmStr\": \"param3_2\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err400 = get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, "valStr"));
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err400).unwrap()))); // 400:BadRequest

        // -- "There is a block on sending messages." --

        let user4_id = profile_vec.get(3).unwrap().id;
        let member4 = profile_vec.get(3).unwrap().nickname.clone();
        let token4 = config_jwt::tests::get_token(user4_id);
        // Join user4.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token4).into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member4.clone(), count: 1, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value)));
        
        // Test: 1.5. Send message "prmBool".
        let prm_text = MessageText("{ \"prmBool\": \"param1_5\", \"valBool\": false }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // Test: 2.5. Send message "prmInt".
        let prm_text = MessageText("{ \"prmInt\": \"param2_5\", \"valInt\": 0 }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden

        // Test: 3.3. Send message "prmStr".
        let prm_text = MessageText("{ \"prmStr\": \"param3_3\", \"valStr\": \"text2\" }".into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        let err403 = get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES);
        assert_eq!(item1, FrameText(Bytes::from(to_string(&err403).unwrap()))); // 403:Forbidden
    }
    #[actix_web::test]
    async fn test_get_ws_chat_ews_prm_ok() {
        // Create a test server without listening on a port.
        let mut srv = actix_test::start(move || {
            let mut data_u = UserOrmTest::users(&[USER, USER]);
            let user2_id = data_u.0.get(1).unwrap().id;
            // Add session (num_token) for user2.
            data_u.1.get_mut(1).unwrap().num_token = Some(config_jwt::tests::get_num_token(user2_id));
            let data_cm = ChatMessageOrmTest::chat_messages(0);
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

        
        let user1_id = profile_vec.get(0).unwrap().id;
        let member1 = profile_vec.get(0).unwrap().nickname.clone();
        let token1 = config_jwt::tests::get_token(user1_id);
        // Join user1.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token1).into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.
        // Message for user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS { 
            join: stream1_id, member: member1.clone(), count: 1, is_owner: Some(true), is_blocked: Some(false) }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value)));

        // Open a websocket connection to the test server.
        let mut framed2 = srv.ws_at(URL_WS).await.unwrap();

        let user2_id = profile_vec.get(1).unwrap().id;
        let member2 = profile_vec.get(1).unwrap().nickname.clone();
        let token2 = config_jwt::tests::get_token(user2_id);
        // -- Join user2 authorized. --
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"join\": {}, \"access\": \"{}\" }}", stream1_id, token2).into());
        framed2.send(prm_text).await.unwrap(); // Send a message to a websocket.
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
        
        // Open a websocket connection to the test server.
        let mut framed3 = srv.ws_at(URL_WS).await.unwrap();

        // -- Join user3 unauthorized. --
        let prm_text = MessageText(format!("{{ \"join\": {} }}", stream1_id).into());
        framed3.send(prm_text).await.unwrap(); // Send a message to a websocket.
        // Message for user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 3, is_owner: Some(false), is_blocked: Some(true) }).unwrap();
        assert_eq!(item3, FrameText(Bytes::from(value)));
        // Message to user1 about user3 joining.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        #[rustfmt::skip]
        let value = to_string(&JoinEWS {
            join: stream1_id, member: "".into(), count: 3, is_owner: None, is_blocked: None }).unwrap();
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message to user2 about user3 joining.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value)));

        // -- Test: 1.1. "The chat owner sends the boolean parameter." -- 
        let prm_bool = "param1_1";
        let val_bool = false;
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmBool\": \"{}\", \"valBool\": {} }}", prm_bool, val_bool).into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let value = to_string(&PrmBoolEWS { prm_bool: prm_bool.into(), val_bool, is_owner: Some(true) }).unwrap();
        // Message to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));

        // -- Test: 1.2. "It is not the chat owner who sends the number parameter." --
        let prm_bool = "param2_2";
        let val_bool = true;
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmBool\": \"{}\", \"valBool\": {} }}", prm_bool, val_bool).into());
        framed2.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let prm_bool = prm_bool.to_owned();
        let value = to_string(&PrmBoolEWS { prm_bool, val_bool, is_owner: None }).unwrap();
        // Message from user2 to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user2 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));

        // -- Test: 2.1. "The chat owner sends the number parameter." -- 
        let prm_int = "param2_1";
        let val_int = 11;
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmInt\": \"{}\", \"valInt\": {} }}", prm_int, val_int).into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let value = to_string(&PrmIntEWS { prm_int: prm_int.into(), val_int, is_owner: Some(true) }).unwrap();
        // Message to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));
        
        // -- Test: 2.2. "It is not the chat owner who sends the number parameter." --
        let prm_int = "param2_2";
        let val_int = 22;
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmInt\": \"{}\", \"valInt\": {} }}", prm_int, val_int).into());
        framed2.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let prm_int = prm_int.to_owned();
        let value = to_string(&PrmIntEWS { prm_int, val_int, is_owner: None }).unwrap();
        // Message from user2 to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user2 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));

        // -- Test: 3.1. "The chat owner sends the string parameter." -- 
        let prm_str = "param3_1";
        let val_str = "text3_1";
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmStr\": \"{}\", \"valStr\": \"{}\" }}", prm_str, val_str).into());
        framed1.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let prm_str = prm_str.to_owned();
        let val_str = val_str.to_owned();
        let value = to_string(&PrmStrEWS { prm_str, val_str, is_owner: Some(true) }).unwrap();
        // Message to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user1 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));
        
        // -- Test: 3.2. "It is not the chat owner who sends the string parameter." --
        let prm_str = "param3_2";
        let val_str = "text3_2";
        // User1 sends a message.
        #[rustfmt::skip]
        let prm_text = MessageText(format!("{{ \"prmStr\": \"{}\", \"valStr\": \"{}\" }}", prm_str, val_str).into());
        framed2.send(prm_text).await.unwrap(); // Send a message to a websocket.

        let prm_str = prm_str.to_owned();
        let val_str = val_str.to_owned();
        let value = to_string(&PrmStrEWS { prm_str, val_str, is_owner: None }).unwrap();
        // Message from user2 to user1.
        let item1 = framed1.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item1, FrameText(Bytes::from(value.clone())));
        // Message to user2.
        let item2 = framed2.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item2, FrameText(Bytes::from(value.clone())));
        // Message from user2 to user3.
        let item3 = framed3.next().await.unwrap().unwrap(); // Receive a message from a websocket.
        assert_eq!(item3, FrameText(Bytes::from(value)));

    }

}
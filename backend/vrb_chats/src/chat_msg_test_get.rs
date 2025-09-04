#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::SecondsFormat;
    use serde_json;
    use vrb_authent::{
        config_jwt,
        user_mock::{UserMock, USER, USER1_ID},
        user_orm::tests::UserOrmTest as User_Test,
    };

    use crate::{
        chat_message_controller::{get_chat_message, tests as ChtCtTest},
        chat_message_models::ChatMessageDto,
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** get_chat_message **

    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg1_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}", stream_id))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true", stream_id))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg_dto_vec).to_string();
        let ch_msg_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg_vec_ser.len());
        assert_eq!(response, ch_msg_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_part1() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();
        let limit = 3;
        let ch_msg2_dto_vec = &ch_msg_dto_vec[0..limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&limit={}", stream_id, limit))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg2_dto_vec).to_string();
        let ch_msg2_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg2_vec_ser.len());
        assert_eq!(response, ch_msg2_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_part2() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();
        let limit = 3;
        let min_date = ch_msg_dto_vec.get(limit - 1).unwrap().date;
        let min_date_str = min_date.to_rfc3339_opts(SecondsFormat::Millis, true);
        let ch_msg2_dto_vec = &ch_msg_dto_vec[limit..2 * limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&minDate={}&limit={}", stream_id, min_date_str, limit))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg2_dto_vec).to_string();
        let ch_msg2_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg2_vec_ser.len());
        assert_eq!(response, ch_msg2_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des_part1() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();
        let limit = 3;
        let ch_msg2_dto_vec = &ch_msg_dto_vec[0..limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true&limit={}", stream_id, limit))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg2_dto_vec).to_string();
        let ch_msg2_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg2_vec_ser.len());
        assert_eq!(response, ch_msg2_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des_part2() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(6);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_cm.0
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();
        let limit = 3;
        let max_date = ch_msg_dto_vec.get(limit - 1).unwrap().date;
        let max_date_str = max_date.to_rfc3339_opts(SecondsFormat::Millis, true);
        let ch_msg2_dto_vec = &ch_msg_dto_vec[limit..2 * limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true&maxDate={}&limit={}", stream_id, max_date_str, limit))
            .insert_header(ChtCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg2_dto_vec).to_string();
        let ch_msg2_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg2_vec_ser.len());
        assert_eq!(response, ch_msg2_vec_ser);
    }
}

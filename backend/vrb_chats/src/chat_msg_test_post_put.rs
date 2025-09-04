#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use vrb_authent::{
        config_jwt,
        user_models::{UserMock, ADMIN, USER, USER1_ID},
        user_orm::tests::UserOrmTest as User_Test,
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        err,
    };

    use crate::{
        chat_message_controller::{post_chat_message, put_chat_message, tests as ChtCtTest},
        chat_message_models::{self, ChatMessageDto, ChatMessageMock as ChMsgMock, CreateChatMessageDto, ModifyChatMessageDto},
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
    };

    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    const MSG_JSON_MISSING_FIELD: &str = "Json deserialize error: missing field";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    // ** post_chat_message **

    #[actix_web::test]
    async fn test_post_chat_message_no_form() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_post_chat_message_empty_json() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_JSON_MISSING_FIELD));
    }
    #[actix_web::test]
    async fn test_post_chat_message_msg_min() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(CreateChatMessageDto { stream_id, msg: ChMsgMock::message_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_chat_message_msg_max() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(CreateChatMessageDto { stream_id, msg: ChMsgMock::message_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_chat_message_stream_id_wrong() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id_wrong = data_cm.0.get(0).unwrap().stream_id.clone() - 1;
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(CreateChatMessageDto { stream_id: stream_id_wrong, msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; stream_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, stream_id_wrong, &msg);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "stream_id": stream_id_wrong, "msg": &msg });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_chat_message_valid_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let user1_name = data_u.0.get(0).unwrap().nickname.clone();
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(CreateChatMessageDto { stream_id, msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let chat_message_dto_res: ChatMessageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(chat_message_dto_res.id, last_msg_id + 1);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let date_s = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Secs, true), date_s);
        assert_eq!(chat_message_dto_res.member, user1_name);
        assert_eq!(chat_message_dto_res.msg, msg);
        assert_eq!(chat_message_dto_res.date_edt, None);
        assert_eq!(chat_message_dto_res.date_rmv, None);
    }

    // ** put_chat_message **

    #[actix_web::test]
    async fn test_put_chat_message_no_form() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_put_chat_message_invald_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let ch_msg_id_bad = format!("{}a", last_msg_id);
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg_id_bad))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[actix_web::test]
    async fn test_put_chat_message_empty_json() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_JSON_MISSING_FIELD));
    }
    #[actix_web::test]
    async fn test_put_chat_message_msg_max() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let msg = ChMsgMock::message_max();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_chat_message_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let user_id1 = data_u.0.get(0).unwrap().id;
        let msg = ChMsgMock::message_norm();
        let id_wrong = last_msg_id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", id_wrong))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, id_wrong, user_id1, msg);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": id_wrong, "user_id": user_id1, "msg": &msg });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_chat_message_msg_another_user_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg_id = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().id.clone();
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, ch_msg_id, user_id1, msg);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": ch_msg_id, "user_id": user_id1, "msg": &msg });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_chat_message_valid_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let ch_msg = data_cm.0.get(0).unwrap().clone();
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let chat_message_dto_res: ChatMessageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(chat_message_dto_res.id, ch_msg.id);
        assert_eq!(chat_message_dto_res.member, ch_msg.user_name);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let date_crt = ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Secs, true), date_crt);
        assert_eq!(chat_message_dto_res.msg, msg);
        assert!(chat_message_dto_res.date_edt.is_some());
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let date_s = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_edt = chat_message_dto_res.date_edt.unwrap();
        assert_eq!(date_edt.to_rfc3339_opts(SecondsFormat::Secs, true), date_s);
        assert_eq!(chat_message_dto_res.date_rmv, None);
    }
    #[actix_web::test]
    async fn test_put_chat_message_admin_msg_another_invald_user_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[ADMIN]);
        let data_cm = ChMesTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}?userId={}a", ch_msg.id, ch_msg.user_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "userId", MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[actix_web::test]
    async fn test_put_chat_message_admin_msg_another_user_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[ADMIN, USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let user_id2 = data_u.0.get(1).unwrap().id;
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let id_wrong = last_msg_id + 1;
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}?userId={}", id_wrong, user_id2))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; id: {}, user_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, id_wrong, user_id2, msg);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": id_wrong, "user_id": user_id2, "msg": &msg });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_chat_message_admin_msg_another_user_valid_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserMock::users(&[ADMIN]);
        let data_cm = ChMesTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = ChMsgMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(User_Test::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}?userId={}", ch_msg.id, ch_msg.user_id))
            .insert_header(ChtCtTest::header_auth(&token1))
            .set_json(ModifyChatMessageDto { msg: msg.clone() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let chat_message_dto_res: ChatMessageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(chat_message_dto_res.id, ch_msg.id);
        assert_eq!(chat_message_dto_res.member, ch_msg.user_name);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let date_crt = ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Secs, true), date_crt);
        assert_eq!(chat_message_dto_res.msg, msg);
        assert!(chat_message_dto_res.date_edt.is_some());
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let date_s = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let date_edt = chat_message_dto_res.date_edt.unwrap();
        assert_eq!(date_edt.to_rfc3339_opts(SecondsFormat::Secs, true), date_s);
        assert_eq!(chat_message_dto_res.date_rmv, None);
    }
}

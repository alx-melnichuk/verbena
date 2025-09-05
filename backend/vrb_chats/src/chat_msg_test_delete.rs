#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::SecondsFormat;
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{UserOrmTest, ADMIN, USER, USER1_ID},
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        err,
    };

    use crate::{
        chat_message_controller::{delete_chat_message, tests as ChatMessageCtrlTest},
        chat_message_models::{ChatMessageDto, ChatMessageMock, ModifyChatMessageDto},
        chat_message_orm::tests::ChatMessageOrmTest,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    // ** delete_chat_message **

    #[actix_web::test]
    async fn test_delete_chat_message_invald_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let last_ch_msg_id = data_cm.0.last().unwrap().id.clone();
        let ch_msg_id_bad = format!("{}a", last_ch_msg_id);
        let msg = ChatMessageMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg_id_bad))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
    async fn test_delete_chat_message_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let last_ch_msg_id = data_cm.0.last().unwrap().id.clone();
        let msg = ChatMessageMock::message_norm();
        let id_wrong = last_ch_msg_id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", id_wrong))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
        let message = format!("{}; id: {}, user_id: {}", err::MSG_PARAMETER_UNACCEPTABLE, id_wrong, user_id1);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": id_wrong, "user_id": user_id1 });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_chat_message_msg_another_user_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = ChatMessageMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
        let message = format!("{}; id: {}, user_id: {}", err::MSG_PARAMETER_UNACCEPTABLE, ch_msg.id, user_id1);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": ch_msg.id, "user_id": user_id1 });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_chat_message_valid_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let ch_msg = data_cm.0.get(0).unwrap().clone();
        let msg = ChatMessageMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        let date_created = ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Millis, true)[..21], date_created[..21]);
        assert_eq!(chat_message_dto_res.msg, ch_msg.msg.unwrap());
        assert_eq!(chat_message_dto_res.date_edt, ch_msg.date_changed);
        assert_eq!(chat_message_dto_res.date_rmv, ch_msg.date_removed);
    }
    #[actix_web::test]
    async fn test_delete_chat_message_admin_msg_another_invald_user_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}?userId={}a", ch_msg.id, ch_msg.user_id))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
    async fn test_delete_chat_message_admin_msg_another_user_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let user_id2 = data_u.0.get(1).unwrap().id;
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let id_wrong = last_msg_id + 1;
        let msg = ChatMessageMock::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}?userId={}", id_wrong, user_id2))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
        let message = format!("{}; id: {}, user_id: {}", err::MSG_PARAMETER_UNACCEPTABLE, id_wrong, user_id2);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": id_wrong, "user_id": user_id2 });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_chat_message_admin_msg_another_user_valid_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN]);
        let data_cm = ChatMessageOrmTest::chat_messages(2);
        let user_id1 = data_u.0.get(0).unwrap().id;
        let ch_msg = data_cm.0.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}?userId={}", ch_msg.id, ch_msg.user_id))
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let chat_message_dto_res: ChatMessageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(chat_message_dto_res.id, ch_msg.id);
        assert_eq!(chat_message_dto_res.member, ch_msg.user_name);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        let date_created = ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Millis, true)[..21], date_created[..21]);
        assert_eq!(chat_message_dto_res.msg, ch_msg.msg.unwrap());
        assert_eq!(chat_message_dto_res.date_edt, ch_msg.date_changed);
        assert_eq!(chat_message_dto_res.date_rmv, ch_msg.date_removed);
    }
}

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
    use vrb_common::api_error::{code_to_str, ApiError};
    use vrb_dbase::db_enums::UserRole;
    use vrb_tools::err; 
    
    use crate::chats::{
        chat_message_controller::{
            delete_chat_message, post_chat_message, put_chat_message, tests as ChtCtTest,
            tests::{configure_chat_message, get_cfg_data, header_auth},
        },
        chat_message_models::{self, ChatMessageDto, CreateChatMessageDto, ModifyChatMessageDto, ChatMessageTest as MessgTest},
        chat_message_orm::tests::ChatMsgTest,
        chat_message_orm::tests::ChatMessageOrmTest as ChMesTest,
    };
    use crate::profiles::{
        config_jwt,
        profile_orm::tests::{ProfileOrmTest as ProflTest, USER},
    };

    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    const MSG_JSON_MISSING_FIELD: &str = "Json deserialize error: missing field";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    // ** post_chat_message **

    #[actix_web::test]
    async fn test_post_chat_message_no_form() {
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        // let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                // .configure(configure_chat_message(cfg_c, data_c))
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        // let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(1);
        // let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id, msg: MessgTest::message_min() })
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id, msg: MessgTest::message_max() })
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id_wrong = data_cm.0.get(0).unwrap().stream_id.clone() - 1;
        // let (cfg_c, data_c, token) = get_cfg_data(1);
        // let stream_id_wrong = ChatMsgTest::stream_ids().get(0).unwrap().clone() - 1;
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        let user1_name = data_p.0.get(0).unwrap().nickname.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let user1_name = data_c.0.get(0).unwrap().nickname.clone();
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        // let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        // let stream_id = data_cm.0.get(0).unwrap().stream_id.clone();
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        let ch_msg_id_bad = format!("{}a", last_msg_id);
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg_id_bad))
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        let msg = MessgTest::message_max();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_msg_id))
            .insert_header(header_auth(&token))
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
        let token = ProflTest::token1();
        let data_p = ProflTest::profiles(&[USER]);
        let data_cm = ChMesTest::chat_messages(2);
        let last_msg_id = data_cm.0.last().unwrap().id.clone();
        // let (cfg_c, data_c, token) = get_cfg_data(3);
        // let user_id1 = data_c.0.get(0).unwrap().user_id;
        let user_id1 = data_p.0.get(0).unwrap().user_id.clone();
        // let last_ch_msg = data_c.2.last().unwrap().clone();
        let msg = MessgTest::message_norm();
        let id_wrong = last_msg_id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(ChMesTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", id_wrong))
            .insert_header(header_auth(&token))
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
    async fn test_put_chat_message_msg_another_user() {
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(header_auth(&token))
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
        let message = format!("{}; id: {}, user_id: {}, msg: \"{}\"", err::MSG_PARAMETER_UNACCEPTABLE, ch_msg.id, user_id1, msg);
        assert_eq!(app_err.message, message);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": ch_msg.id, "user_id": user_id1, "msg": &msg });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_chat_message_valid_data() {
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let ch_msg = data_c.2.get(0).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(header_auth(&token))
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
        let (cfg_c, mut data_c, token) = get_cfg_data(3);
        data_c.0.get_mut(0).unwrap().role = UserRole::Admin;
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}?userId={}a", ch_msg.id, ch_msg.user_id))
            .insert_header(header_auth(&token))
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
    async fn test_put_chat_message_admin_msg_another_user() {
        let (cfg_c, mut data_c, token) = get_cfg_data(3);
        data_c.0.get_mut(0).unwrap().role = UserRole::Admin;
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}?userId={}", ch_msg.id, ch_msg.user_id))
            .insert_header(header_auth(&token))
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

    // ** delete_chat_message **

    #[actix_web::test]
    async fn test_delete_chat_message_invald_id() {
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let ch_msg_id_bad = format!("{}a", last_ch_msg.id);
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg_id_bad))
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let msg = MessgTest::message_norm();
        let id_wrong = last_ch_msg.id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", id_wrong))
            .insert_header(header_auth(&token))
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
    async fn test_delete_chat_message_msg_another_user() {
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        #[rustfmt::skip]
        let ch_msg = data_c.2.get(0).unwrap().clone();
        let msg = MessgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}", ch_msg.id))
            .insert_header(header_auth(&token))
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
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Millis, true)
            , ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Millis, true));
        assert_eq!(chat_message_dto_res.msg, ch_msg.msg.unwrap());
        assert_eq!(chat_message_dto_res.date_edt, ch_msg.date_changed);
        assert_eq!(chat_message_dto_res.date_rmv, ch_msg.date_removed);
    }
    #[actix_web::test]
    async fn test_delete_chat_message_admin_msg_another_invald_user_id() {
        let (cfg_c, mut data_c, token) = get_cfg_data(3);
        data_c.0.get_mut(0).unwrap().role = UserRole::Admin;
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}?userId={}a", ch_msg.id, ch_msg.user_id))
            .insert_header(header_auth(&token))
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
    async fn test_delete_chat_message_admin_msg_another_user() {
        let (cfg_c, mut data_c, token) = get_cfg_data(3);
        data_c.0.get_mut(0).unwrap().role = UserRole::Admin;
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let ch_msg = data_c.2.iter().find(|v| v.user_id != user_id1).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/chat_messages/{}?userId={}", ch_msg.id, ch_msg.user_id))
            .insert_header(header_auth(&token))
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
        #[rustfmt::skip]
        assert_eq!(chat_message_dto_res.date.to_rfc3339_opts(SecondsFormat::Millis, true)
            , ch_msg.date_created.to_rfc3339_opts(SecondsFormat::Millis, true));
        assert_eq!(chat_message_dto_res.msg, ch_msg.msg.unwrap());
        assert_eq!(chat_message_dto_res.date_edt, ch_msg.date_changed);
        assert_eq!(chat_message_dto_res.date_rmv, ch_msg.date_removed);
    }
}

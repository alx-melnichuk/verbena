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
    use vrb_dbase::db_enums::UserRole;
    use vrb_tools::{
        api_error::{code_to_str, ApiError},
        err,
    };

    use crate::chats::{
        chat_message_controller::{
            delete_chat_message, post_chat_message, put_chat_message,
            tests::{
                configure_chat_message, get_cfg_data, header_auth, MSG_CASTING_TO_TYPE, MSG_CONTENT_TYPE_ERROR, MSG_FAILED_DESER,
                MSG_JSON_MISSING_FIELD,
            },
        },
        chat_message_models::{self, ChatMessageDto, CreateChatMessageDto, ModifyChatMessageDto},
        chat_message_orm::tests::ChatMsgTest,
    };

    fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** post_chat_message **

    #[actix_web::test]
    async fn test_post_chat_message_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, token) = get_cfg_data(1);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id, msg: ChatMsgTest::message_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_chat_message_msg_max() {
        let (cfg_c, data_c, token) = get_cfg_data(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id: 1, msg: ChatMsgTest::message_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_chat_message_stream_id_wrong() {
        let (cfg_c, data_c, token) = get_cfg_data(1);
        let stream_id_wrong = ChatMsgTest::stream_ids().get(0).unwrap().clone() - 1;
        let msg = ChatMsgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let user1_name = data_c.0.get(0).unwrap().nickname.clone();
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let msg = ChatMsgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        assert_eq!(chat_message_dto_res.id, last_ch_msg.id + 1);
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let last_ch_msg = data_c.2.last().unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_ch_msg.id))
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let ch_msg_id_bad = format!("{}a", last_ch_msg.id);
        let msg = ChatMsgTest::message_norm();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let last_ch_msg = data_c.2.last().unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_ch_msg.id))
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
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let msg = ChatMsgTest::message_max();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/chat_messages/{}", last_ch_msg.id))
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_MESSAGE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_chat_message_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(3);
        let user_id1 = data_c.0.get(0).unwrap().user_id;
        let last_ch_msg = data_c.2.last().unwrap().clone();
        let msg = ChatMsgTest::message_norm();
        let id_wrong = last_ch_msg.id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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
        let msg = ChatMsgTest::message_norm();
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

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{SecondsFormat, Utc};
    use serde_json::{self, json};
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        validators,
    };
    use vrb_tools::err;

    use crate::chats::{
        chat_message_controller::{
            delete_blocked_user, get_blocked_users, post_blocked_user, tests as ChtCtTest,
            tests::{configure_chat_message, get_cfg_data, header_auth},
        },
        chat_message_models::{self, BlockedUserDto, ChatMessageTest, CreateBlockedUserDto, DeleteBlockedUserDto},
        chat_message_orm::tests::ChatMsgTest,
    };

    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    // ** get_blocked_users **

    #[actix_web::test]
    async fn test_get_blocked_users_by_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let stream_id_bad = format!("{}a", stream_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/blocked_users/{}", stream_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `stream_id` - {} ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, MSG_CASTING_TO_TYPE, stream_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_blocked_users_by_non_existent_stream_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone() + 99999;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/blocked_users/{}", stream_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_dto_vec: Vec<BlockedUserDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_dto_vec.len(), 0);
    }
    #[actix_web::test]
    async fn test_get_blocked_users_by_user_is_owner() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id.clone();
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/blocked_users/{}", stream_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_dto_vec: Vec<BlockedUserDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        let blocked_user_dto_ls: Vec<BlockedUserDto> = ChatMsgTest::get_blocked_user_vec()
            .iter().filter(|v| v.user_id == user_id).map(|v| BlockedUserDto::from(v.clone())).collect();
        assert_eq!(blocked_user_dto_vec.len(), blocked_user_dto_ls.len());
        let blocked_user_res = blocked_user_dto_vec.get(0).unwrap();
        let blocked_user_dto = blocked_user_dto_ls.get(0).unwrap();
        assert_eq!(blocked_user_res.id, blocked_user_dto.id);
        assert_eq!(blocked_user_res.user_id, blocked_user_dto.user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_user_dto.blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_user_dto.blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            blocked_user_dto.block_date.to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    #[actix_web::test]
    async fn test_get_blocked_users_by_user_is_not_owner() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(1).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/blocked_users/{}", stream_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_dto_vec: Vec<BlockedUserDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_dto_vec.len(), 0);
    }

    // ** post_blocked_user **

    #[actix_web::test]
    async fn test_post_blocked_user_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_post_blocked_user_empty_json() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        let status_code417 = code_to_str(StatusCode::EXPECTATION_FAILED);
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &status_code417, &[chat_message_models::MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "optionalFields": "blocked_id, blocked_nickname" });
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get(validators::NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT).unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_min_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let blocked_nickname = ChatMessageTest::blocked_nickname_min();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MIN_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MIN });
        assert_eq!(*app_err.params.get("minlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_max_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let blocked_nickname = ChatMessageTest::blocked_nickname_max();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MAX_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MAX });
        assert_eq!(*app_err.params.get("maxlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_invalid_blocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.last().unwrap().user_id + 9999;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_invalid_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let nickname = format!("{}a", data_c.0.last().unwrap().nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_new_blocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        let blocked_id = data_c.0.get(1).unwrap().user_id;
        let blocked_nickname = data_c.0.get(1).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: Some(blocked_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_new_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        let blocked_id = data_c.0.get(1).unwrap().user_id;
        let blocked_nickname = data_c.0.get(1).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_old_blocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked_user = ChatMsgTest::get_blocked_user_vec().iter().find(|v| v.user_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_id = blocked_user.blocked_id;
        let blocked_nickname = blocked_user.blocked_nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: Some(blocked_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_old_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked_user = ChatMsgTest::get_blocked_user_vec().iter().find(|v| v.user_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_id = blocked_user.blocked_id;
        let blocked_nickname = blocked_user.blocked_nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }

    // ** delete_blocked_user **

    #[actix_web::test]
    async fn test_delete_blocked_user_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_empty_json() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "optionalFields": "blocked_id, blocked_nickname" });
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get(validators::NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT).unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_min_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let blocked_nickname = ChatMessageTest::blocked_nickname_min();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MIN_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MIN });
        assert_eq!(*app_err.params.get("minlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_max_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let blocked_nickname = ChatMessageTest::blocked_nickname_max();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChtCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MAX_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MAX });
        assert_eq!(*app_err.params.get("maxlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_invalid_blocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.last().unwrap().user_id + 9999;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_invalid_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let nickname = format!("{}a", data_c.0.last().unwrap().nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_unblocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(1).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_unblocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let nickname = data_c.0.get(1).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_old_blocked_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked_user = ChatMsgTest::get_blocked_user_vec().iter().find(|v| v.user_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_id = blocked_user.blocked_id;
        let blocked_nickname = blocked_user.blocked_nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(blocked_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_old_blocked_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let user_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked_user = ChatMsgTest::get_blocked_user_vec().iter().find(|v| v.user_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_id = blocked_user.blocked_id;
        let blocked_nickname = blocked_user.blocked_nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(header_auth(&token))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_res: BlockedUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_res.user_id, user_id);
        assert_eq!(blocked_user_res.blocked_id, blocked_id);
        assert_eq!(blocked_user_res.blocked_nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        #[rustfmt::skip]
        assert_eq!(blocked_user_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true),
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    }
}

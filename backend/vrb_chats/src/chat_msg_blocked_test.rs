#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, App, body, dev,
        http::StatusCode,
        http::header::{CONTENT_TYPE, HeaderValue},
        test,
    };
    use chrono::{SecondsFormat, Utc};
    use serde_json::{self, json};
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{USER, USER1_ID, UserOrmTest},
    };
    use vrb_common::{
        api_error::{ApiError, code_to_str},
        validators,
    };

    use crate::{
        chat_message_controller::{delete_blocked_user, get_blocked_users, get_blocked_users_names, post_blocked_user, tests as ChatMessageCtrlTest},
        chat_message_models::{self, BlockedUser, BlockedUserDto, BlockedUserMiniDto, ChatMessageMock, CreateBlockedUserDto, DeleteBlockedUserDto},
        chat_message_orm::tests::{BlockedData, ChatMessageOrmTest},
    };

    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** get_blocked_users_names **

    #[actix_web::test]
    async fn test_get_blocked_users_names_exist_blocked_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let blocked_users: Vec<BlockedData> = data_cm.2.iter()
            .filter(|v| v.owner_id == user1_id).map(|v| v.clone()).collect();
        #[rustfmt::skip]
        let nickname_vec: Vec<String> = blocked_users.iter().map(|v| v.nickname.clone()).collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users_names)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/blocked_users/nicknames")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nicknames_res: Vec<String> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(nicknames_res.len(), nickname_vec.len());
        for (index, nickname1) in nicknames_res.iter().enumerate() {
            let nickname2 = nickname_vec.get(index).unwrap();
            assert_eq!(nickname1, nickname2);
        }
    }
    #[actix_web::test]
    async fn test_get_blocked_users_names_not_exist_blocked_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let mut data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let blocked_users: Vec<BlockedData> = data_cm.2.iter()
            .filter(|v| v.owner_id != user1_id).map(|v| v.clone()).collect();
        data_cm.2 = blocked_users;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users_names)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/blocked_users/nicknames")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nicknames_res: Vec<String> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(nicknames_res.len(), 0);
    }

    // ** get_blocked_users **

    #[actix_web::test]
    async fn test_get_blocked_users_exist_blocked_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let blocked_users_vec: Vec<BlockedUserDto> = data_cm.2.iter()
            .filter(|v| v.owner_id == user1_id)
            .map(|v| BlockedUserDto::from(BlockedUser::from(v.clone().into())))
            .collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_users_res: Vec<BlockedUserDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_users_res.len(), blocked_users_vec.len());
        for (index, blocked_user1) in blocked_users_res.iter().enumerate() {
            let blocked_user2 = blocked_users_vec.get(index).unwrap();
            assert_eq!(blocked_user1.id, blocked_user2.id);
            assert_eq!(blocked_user1.user_id, blocked_user2.user_id);
            assert_eq!(blocked_user1.nickname, blocked_user2.nickname);
            assert_eq!(blocked_user1.email, blocked_user2.email);
            // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
            // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
            let block_date = blocked_user2.block_date.to_rfc3339_opts(SecondsFormat::Secs, true);
            #[rustfmt::skip]
            assert_eq!(blocked_user1.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), block_date);
            assert_eq!(blocked_user1.avatar, blocked_user2.avatar);
        }
    }
    #[actix_web::test]
    async fn test_get_blocked_users_not_exist_blocked_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let mut data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let blocked_users: Vec<BlockedData> = data_cm.2.iter()
            .filter(|v| v.owner_id != user1_id).map(|v| v.clone()).collect();
        data_cm.2 = blocked_users;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_blocked_users)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
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
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
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
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
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
        ChatMessageCtrlTest::check_app_err(app_err_vec, &status_code417, &[chat_message_models::MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "optionalFields": "blocked_id, blocked_nickname" });
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get(validators::NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT).unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_min_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_nickname = ChatMessageMock::blocked_nickname_min();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChatMessageCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MIN_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MIN });
        assert_eq!(*app_err.params.get("minlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_max_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_nickname = ChatMessageMock::blocked_nickname_max();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChatMessageCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MAX_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MAX });
        assert_eq!(*app_err.params.get("maxlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_invalid_blocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.last().unwrap().id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_invalid_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let nickname = format!("{}a", data_u.0.last().unwrap().nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_new_blocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_id = data_u.0.get(1).unwrap().id;
        let blocked_nickname = data_u.0.get(1).unwrap().nickname.clone();
        let blocked_last_id = data_cm.2.last().unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: Some(blocked_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked_last_id + 1);
        assert_eq!(blocked_user_mini_res.user_id, blocked_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_new_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_id = data_u.0.get(1).unwrap().id;
        let blocked_nickname = data_u.0.get(1).unwrap().nickname.clone();
        let blocked_last_id = data_cm.2.last().unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked_last_id + 1);
        assert_eq!(blocked_user_mini_res.user_id, blocked_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked_nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_old_blocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.get(0).unwrap().id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked = data_cm.2.iter().find(|v| v.owner_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_id = blocked.user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: Some(blocked_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked.id);
        assert_eq!(blocked_user_mini_res.user_id, blocked.user_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked.nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }
    #[actix_web::test]
    async fn test_post_blocked_user_by_old_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.get(0).unwrap().id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked = data_cm.2.iter().find(|v| v.owner_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_nickname = blocked.nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(CreateBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked.id);
        assert_eq!(blocked_user_mini_res.user_id, blocked.user_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked.nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }

    // ** delete_blocked_user **

    #[actix_web::test]
    async fn test_delete_blocked_user_no_form() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_empty_json() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChatMessageCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_ONE_OPTIONAL_MUST_PRESENT]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "optionalFields": "blocked_id, blocked_nickname" });
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get(validators::NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT).unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_min_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_nickname = ChatMessageMock::blocked_nickname_min();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChatMessageCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MIN_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MIN });
        assert_eq!(*app_err.params.get("minlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_max_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let blocked_nickname = ChatMessageMock::blocked_nickname_max();
        let len1 = blocked_nickname.len();
        let blocked_nickname = Some(blocked_nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        ChatMessageCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[chat_message_models::MSG_BLOCKED_NICKNAME_MAX_LENGTH]);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualLength": len1, "requiredLength": chat_message_models::BLOCKED_NICKNAME_MAX });
        assert_eq!(*app_err.params.get("maxlength").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_invalid_blocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.last().unwrap().id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_invalid_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let nickname = format!("{}a", data_u.0.last().unwrap().nickname);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_unblocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.get(1).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_unblocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let nickname = data_u.0.get(1).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(nickname) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_old_blocked_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.get(0).unwrap().id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked = data_cm.2.iter().find(|v| v.owner_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_user_id = blocked.user_id;
        // let blocked_nickname = blocked.nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: Some(blocked_user_id), blocked_nickname: None })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked.id);
        assert_eq!(blocked_user_mini_res.user_id, blocked.user_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked.nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }
    #[actix_web::test]
    async fn test_delete_blocked_user_by_old_blocked_nickname() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER, USER, USER]);
        let data_cm = ChatMessageOrmTest::chat_messages(1);
        let user_id = data_u.0.get(0).unwrap().id;
        #[rustfmt::skip] // Find a user who is already blocked for user1.
        let blocked = data_cm.2.iter().find(|v| v.owner_id == user_id).map(|v| v.clone()).unwrap();
        let blocked_nickname = blocked.nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_blocked_user)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ChatMessageOrmTest::cfg_chat_message_orm(data_cm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/blocked_users")
            .insert_header(ChatMessageCtrlTest::header_auth(&token1))
            .set_json(DeleteBlockedUserDto { blocked_id: None, blocked_nickname: Some(blocked_nickname.clone()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let blocked_user_mini_res: BlockedUserMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(blocked_user_mini_res.id, blocked.id);
        assert_eq!(blocked_user_mini_res.user_id, blocked.user_id);
        assert_eq!(blocked_user_mini_res.nickname, blocked.nickname);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(blocked_user_mini_res.block_date.to_rfc3339_opts(SecondsFormat::Secs, true), now_date);
    }
}

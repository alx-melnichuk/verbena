#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body,
        cookie::time::Duration as ActixWebDuration,
        dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        err,
    };

    use crate::{
        config_jwt,
        user_authent_controller::{users_uniqueness, logout, tests as AthCtTest,},
        user_authent_models::UserUniquenessResponseDto,
        user_models::Session,
        user_orm::tests::{UserOrmTest as User_Test, USER, USER1_ID},
        user_registr_orm::tests::UserRegistrOrmTest as RegisTest,
    };

    const MSG_ERROR_WAS_EXPECTED: &str = "Service call succeeded, but an error was expected.";
    const MSG_FAILED_TO_DESER: &str = "Failed to deserialize JSON string";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** users_uniqueness **

    #[actix_web::test]
    async fn test_users_uniqueness_by_non_params() {
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/users_uniqueness")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_nickname_empty() {
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/users_uniqueness?nickname=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_email_empty() {
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/users_uniqueness?email=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_nickname_profile() {
        let data_u = User_Test::users(&[USER]);
        let nickname = data_u.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(false);
        assert_eq!(response1_dto, response2_dto);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_email_profile() {
        let data_u = User_Test::users(&[USER]);
        let email = data_u.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(false);
        assert_eq!(response1_dto, response2_dto);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_nickname_registr() {
        let data_u = User_Test::users(&[USER]);
        let registr = RegisTest::registrs(true);
        let nickname = registr.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(false);
        assert_eq!(response1_dto, response2_dto);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_email_registr() {
        let data_u = User_Test::users(&[USER]);
        let registr = RegisTest::registrs(true);
        let email = registr.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(false);
        assert_eq!(response1_dto, response2_dto);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_new_nickname() {
        let data_u = User_Test::users(&[USER]);
        let nickname = format!("a{}", data_u.0.get(0).unwrap().nickname.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(true);
        assert_eq!(response1_dto, response2_dto);
    }
    #[actix_web::test]
    async fn test_users_uniqueness_by_new_email() {
        let data_u = User_Test::users(&[USER]);
        let email = format!("a{}", data_u.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(users_uniqueness)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response2_dto: UserUniquenessResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let response1_dto = UserUniquenessResponseDto::new(true);
        assert_eq!(response1_dto, response2_dto);
    }

    // ** logout **

    #[actix_web::test]
    async fn test_logout_missing_token() {
        let data_u = User_Test::users(&[]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout").to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(a)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, err::MSG_MISSING_TOKEN);
    }
    #[actix_web::test]
    async fn test_logout_invalid_token() {
        let data_u = User_Test::users(&[]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(AthCtTest::header_auth("invalid_token123"))
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401b

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert!(api_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_logout_valid_token_session_non_exist() {
        let data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(AthCtTest::header_auth(&token2))
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::NOT_ACCEPTABLE); // 406

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user2_id));
    }
    #[actix_web::test]
    async fn test_logout_valid_token_non_existent_user() {
        let mut data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        data_u.1 = vec![Session::new(user2_id, Some(User_Test::get_num_token(user2_id)))];
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(AthCtTest::header_auth(&token2))
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(d)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_ID, user2_id));
    }
    #[actix_web::test]
    async fn test_logout_valid_token_non_existent_num() {
        let mut data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        data_u.1 = vec![Session::new(user2_id, Some(User_Test::get_num_token(USER1_ID)))];
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(AthCtTest::header_auth(&token2))
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(c)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user2_id));
    }
    #[actix_web::test]
    async fn test_logout_valid_token() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(AthCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        let token_cookie_opt = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie_opt.is_some());
        let token = token_cookie_opt.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() == 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(0, 0));
        assert_eq!(true, token.http_only().unwrap());

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(body_str, "");
    }

}

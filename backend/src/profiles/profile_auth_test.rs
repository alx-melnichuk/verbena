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
    use serde_json::json;
    use vrb_authent::{
        config_jwt,
        user_auth_models::Session,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, USER, USER1_ID},
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        err,
    };
    use vrb_tools::{hash_tools, token_coding, token_data::TOKEN_NAME};

    use crate::profiles::{
        profile_auth_controller::{login, logout, tests as AthCtTest, update_token},
        profile_models::{self, LoginProfileDto, LoginProfileResponseDto, ProfileDto, ProfileTest, TokenDto},
        profile_orm::tests::ProfileOrmTest as ProflTest,
    };

    const MSG_ERROR_WAS_EXPECTED: &str = "Service call succeeded, but an error was expected.";
    const MSG_FAILED_TO_DESER: &str = "Failed to deserialize JSON string";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** login **

    #[actix_web::test]
    async fn test_login_no_data() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))               
        ).await;
        let req = test::TestRequest::post().uri("/api/login").to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_login_empty_json_object() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        let req = test::TestRequest::post().uri("/api/login").set_json(json!({})).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_empty() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: "".to_string(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_min() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::nickname_min(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_max() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::nickname_max(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_wrong() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::nickname_wrong(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_min() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::email_min(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_max() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::email_max(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_wrong() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: ProfileTest::email_wrong(), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_empty() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: "James_Smith".to_string(), password: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_min() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: "James_Smith".to_string(), password: ProfileTest::password_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_max() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: "James_Smith".to_string(), password: ProfileTest::password_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_wrong() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: "James_Smith".to_string(), password: ProfileTest::password_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        AthCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_if_nickname_not_exist() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        let nickname = profiles.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: format!("a{}", nickname), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401(f)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_email_not_exist() {
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        let email = profiles.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: format!("a{}", email), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401(f)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_password_invalid_hash() {
        let data_u = User_Test::users(&[USER]);
        let mut profiles = ProflTest::profiles(&data_u.0);
        let profile1 = profiles.get_mut(0).unwrap();
        let password = "hash_password_R2B2";
        profile1.password = password.to_string();
        let nickname = profile1.nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert!(app_err.message.starts_with(err::MSG_INVALID_HASH));
    }
    #[actix_web::test]
    async fn test_login_if_password_incorrect() {
        let data_u = User_Test::users(&[USER]);
        let mut profiles = ProflTest::profiles(&data_u.0);
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = profiles.get_mut(0).unwrap();
        profile1.nickname = nickname.clone().to_lowercase();
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: format!("{}b", password) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401(g)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_login_err_jsonwebtoken_encode() {
        let data_u = User_Test::users(&[USER]);
        let mut profiles = ProflTest::profiles(&data_u.0);
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = profiles.get_mut(0).unwrap();
        profile1.nickname = nickname.clone().to_lowercase();
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY); // 422

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNPROCESSABLE_ENTITY));
        assert_eq!(app_err.message, format!("{}; InvalidKeyFormat", err::MSG_JSON_WEB_TOKEN_ENCODE));
    }
    #[actix_web::test]
    async fn test_login_if_session_not_exist() {
        let mut data_u = User_Test::users(&[USER]);
        let mut profiles = ProflTest::profiles(&data_u.0);
        let profile1 = profiles.last_mut().unwrap();
        // Change ID, reset connection with session.
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1_id = profile1.user_id;
        // profile1.user_id = profile1_id;
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let session1 = data_u.1.get_mut(0).unwrap();
        session1.user_id = session1.user_id + 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile1_id));
    }
    #[actix_web::test]
    async fn test_login_valid_credentials() {
        let data_u = User_Test::users(&[USER]);
        let mut profiles = ProflTest::profiles(&data_u.0);
        let profile1 = profiles.get_mut(0).unwrap();
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let profile1_dto = ProfileDto::from(profile1.clone());
        let jwt_access = config_jwt::get_test_config().jwt_access;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let token_cookie_opt = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie_opt.is_some());

        let token = token_cookie_opt.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() > 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(jwt_access, 0));
        assert_eq!(true, token.http_only().unwrap());

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let login_resp: LoginProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = login_resp.profile_tokens_dto.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = login_resp.profile_tokens_dto.refresh_token;
        assert!(refresh_token.len() > 0);

        let profile_dto_res = login_resp.profile_dto;
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }

    // ** logout **

    #[actix_web::test]
    async fn test_logout_missing_token() {
        let data_u = User_Test::users(&[]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
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
                .configure(User_Test::cfg_user_auth_orm(data_u))
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
                .configure(User_Test::cfg_user_auth_orm(data_u))
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
                .configure(User_Test::cfg_user_auth_orm(data_u))
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
                .configure(User_Test::cfg_user_auth_orm(data_u))
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
                .configure(User_Test::cfg_user_auth_orm(data_u))
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

    // ** update_token **

    #[actix_web::test]
    async fn test_update_token_no_data() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_update_token_empty_json_object() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_update_token_invalid_dto_token_empty() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(TokenDto { token: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidSubject"));
    }
    #[actix_web::test]
    async fn test_update_token_invalid_dto_token_invalid() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(TokenDto { token: "invalid_token".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_update_token_unacceptable_token_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        let profile1_id = profiles.get(0).unwrap().user_id;
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret = config_jwt.jwt_secret.as_bytes();
        let profile_id_bad = profile1_id + 1;
        let num_token = data_u.1.get(0).unwrap().num_token.unwrap();
        let token_bad = token_coding::encode_token(profile_id_bad, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(TokenDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile_id_bad));
    }
    #[actix_web::test]
    async fn test_update_token_unacceptable_token_num() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        let profile1_id = profiles.get(0).unwrap().user_id;
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret = config_jwt.jwt_secret.as_bytes();
        let num_token = data_u.1.get(0).unwrap().num_token.unwrap();
        let token_bad = token_coding::encode_token(profile1_id, num_token + 1, &jwt_secret, config_jwt.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(TokenDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, profile1_id));
    }
    #[actix_web::test]
    async fn test_update_token_valid_dto_token() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let profiles = ProflTest::profiles(&data_u.0);
        let profile1_id = profiles.get(0).unwrap().user_id;
        let config_jwt = config_jwt::get_test_config();
        let jwt_access = config_jwt.jwt_access;
        let jwt_secret = config_jwt.jwt_secret.as_bytes();
        let num_token = data_u.1.get(0).unwrap().num_token.unwrap();
        let token_refresh = token_coding::encode_token(profile1_id, num_token, &jwt_secret, config_jwt.jwt_refresh).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(ProflTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(AthCtTest::header_auth(&token1))
            .set_json(TokenDto { token: token_refresh })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        let opt_token_cookie = resp.response().cookies().find(|cookie| cookie.name() == TOKEN_NAME);
        assert!(opt_token_cookie.is_some());

        let token = opt_token_cookie.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() > 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(jwt_access, 0));
        assert_eq!(true, token.http_only().unwrap());

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_token_resp: profile_models::ProfileTokensDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let access_token: String = profile_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = profile_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);

        assert_eq!(token_value, access_token);
    }
}

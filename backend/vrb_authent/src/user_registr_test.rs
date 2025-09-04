#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::env;

    use actix_web::{
        body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{Duration, SecondsFormat, Utc};
    use serde_json::json;
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        consts, err, user_validations,
    };
    use vrb_tools::{config_app, send_email::config_smtp, token_coding};

    use crate::{
        config_jwt,
        user_mock::UserMock,
        user_orm::tests::{UserOrmTest as User_Test, ADMIN, USER, USER1_ID},
        user_registr_controller::{
            confirm_registration, registration, registration_clear_for_expired, tests as RgsCtTest, MSG_REGISTR_NOT_FOUND,
        },
        user_registr_models::{
            ConfirmRegistrUserResponseDto, RegistrUserDto, RegistrUserResponseDto, RegistrationClearForExpiredResponseDto,
        },
        user_registr_orm::tests::UserRegistrOrmTest as RegisTest,
    };

    const TEST_PATH_TEMPLATE: &str = "../templates";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** registration **

    #[actix_web::test]
    async fn test_registration_no_data() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration").to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Content type error"));
    }
    #[actix_web::test]
    async fn test_registration_empty_json_object() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration").set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Json deserialize error: missing field"));
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_empty() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_min() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: UserMock::nickname_min(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_max() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: UserMock::nickname_max(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: UserMock::nickname_wrong(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_empty() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_min() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserMock::email_min(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_max() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserMock::email_max(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_wrong() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserMock::email_wrong(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_empty() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_min() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserMock::password_min(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_max() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserMock::password_max(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_wrong() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserMock::password_wrong(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RgsCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_users() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        let nickname1 = data_u.0.get(0).unwrap().nickname.clone();
        let email1 = data_u.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: nickname1, email: format!("A{}", email1), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert_eq!(app_err.message, err::MSG_NICKNAME_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_email_exists_in_users() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        let nickname1 = data_u.0.get(0).unwrap().nickname.clone();
        let email1 = data_u.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: format!("A{}", nickname1), email: email1, password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert_eq!(app_err.message, err::MSG_EMAIL_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_registr() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        let registrs = RegisTest::registrs(true);
        let nickname1 = registrs.get(0).unwrap().nickname.clone();
        let email1 = registrs.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: nickname1, email: format!("A{}", email1), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert_eq!(app_err.message, err::MSG_NICKNAME_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_email_exists_in_registr() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        let registrs = RegisTest::registrs(true);
        let nickname1 = registrs.get(0).unwrap().nickname.clone();
        let email1 = registrs.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: format!("A{}", nickname1), email: email1, password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert_eq!(app_err.message, err::MSG_EMAIL_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_err_jsonwebtoken_encode() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserMock::users(&[USER]);
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let nickname = "Mary_Williams".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: nickname.clone(), email: format!("{}@gmail.com", nickname), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY); // 422

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNPROCESSABLE_ENTITY));
        assert!(app_err.message.starts_with(&format!("{};", err::MSG_JSON_WEB_TOKEN_ENCODE)));
    }
    #[actix_web::test]
    async fn test_registration_new_user() {
        env::set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let registrs = RegisTest::registrs(true);
        let user_registr1 = registrs.get(0).unwrap().clone();
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RgsCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RgsCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrUserDto {
                nickname: user_registr1.nickname.clone(),
                email: user_registr1.email.clone(),
                password: user_registr1.password.clone(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let registr_profile_resp: RegistrUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(user_registr1.nickname, registr_profile_resp.nickname);
        assert_eq!(user_registr1.email, registr_profile_resp.email);

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let (user_registr_id, _) = token_coding::decode_token(&registr_profile_resp.registr_token, jwt_secret).unwrap();
        assert_eq!(user_registr1.id, user_registr_id);
    }

    // ** confirm_registration **

    #[actix_web::test]
    async fn test_confirm_registration_invalid_registr_token() {
        let data_u = UserMock::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", "invalid_registr_token"))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_confirm_registration_final_date_has_expired() {
        let data_u = UserMock::users(&[USER]);
        let registrs = RegisTest::registrs(true);
        let user_reg1 = registrs.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = token_coding::encode_token(user_reg1_id, num_token1, jwt_secret, -reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(registrs))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "ExpiredSignature"));
    }
    #[actix_web::test]
    async fn test_confirm_registration_no_exists_in_user_regist() {
        let data_u = UserMock::users(&[USER]);
        let registrs = RegisTest::registrs(true);
        let user_reg1 = registrs.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let user_reg_id = user_reg1_id + 1;
        let registr_token = token_coding::encode_token(user_reg_id, num_token1, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(registrs))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_FOUND));
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; user_registr_id: {}", MSG_REGISTR_NOT_FOUND, user_reg_id));
    }
    #[actix_web::test]
    async fn test_confirm_registration_exists_in_user_regist() {
        let data_u = UserMock::users(&[USER]);
        let registrs = RegisTest::registrs(true);
        let user_reg1 = registrs.get(0).unwrap().clone();
        let nickname = user_reg1.nickname.to_string();
        let email = user_reg1.email.to_string();
        let last_user_id = data_u.0.last().unwrap().id;

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = token_coding::encode_token(user_reg1.id, num_token1, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(registrs))
                .configure(User_Test::cfg_user_orm(data_u))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto_res: ConfirmRegistrUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let now_str = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        assert_eq!(response_dto_res.id, last_user_id + 1);
        assert_eq!(response_dto_res.nickname, nickname);
        assert_eq!(response_dto_res.email, email);
        assert_eq!(response_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Millis, true), now_str);
    }

    // ** registration_clear_for_expired **

    #[actix_web::test]
    async fn test_registration_clear_for_expired() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = UserMock::users(&[ADMIN]);
        let config_app = config_app::get_test_config();

        let registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
        let mut registrs = RegisTest::registrs(true);
        let registr1 = registrs.get_mut(0).unwrap();
        registr1.final_date = Utc::now() - Duration::seconds(registr_duration);

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration_clear_for_expired)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&"/api/registration/clear_for_expired")
            .insert_header(RgsCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto: RegistrationClearForExpiredResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto.count_inactive_registr, 1);
    }
}

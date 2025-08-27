#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{Duration, Utc};
    use serde_json::json;
    use vrb_authent::{
        config_jwt,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, ADMIN, USER, USER1_ID},
        user_recovery_orm::tests::UserRecoveryOrmTest as RecovTest,
        user_registr_orm::tests::UserRegistrOrmTest as RegisTest,
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        err, user_validations,
    };
    use vrb_dbase::enm_user_role::UserRole;
    use vrb_tools::{config_app, send_email::config_smtp, token_coding};

    use crate::{
        profile_models::{
            ClearForExpiredResponseDto, Profile, ProfileDto, ProfileTest, RecoveryDataDto, RecoveryProfileDto, RecoveryProfileResponseDto,
            RegistrProfileDto, RegistrProfileResponseDto,
        },
        profile_orm::tests::ProfileOrmTest as ProflTest,
        profile_registr_controller::{
            clear_for_expired, confirm_recovery, confirm_registration, recovery, registration, tests as RegCtTest, MSG_RECOVERY_NOT_FOUND,
            MSG_REGISTR_NOT_FOUND, MSG_USER_NOT_FOUND,
        },
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn get_profiles(role_idx: u8) -> Vec<Profile> {
        let nickname = "Oliver_Taylor".to_lowercase();
        let email = format!("{}@gmail.com", nickname).to_lowercase();
        #[rustfmt::skip]
        let role = if role_idx == ADMIN { UserRole::Admin } else { UserRole::User };
        let profile = Profile::new2(USER1_ID, &nickname, &email, "", role, None, None, None, None);
        vec![profile]
    }

    // ** registration **

    #[actix_web::test]
    async fn test_registration_no_data() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
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
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
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
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_min() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_min(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_max() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_max(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_wrong(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_empty() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_min() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_min(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_max() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_max(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_wrong() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_wrong(),
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_empty() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_min() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_min(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_max() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_max(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_wrong() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_wrong(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_users() {
        let profiles = get_profiles(USER);
        let nickname1 = profiles.get(0).unwrap().nickname.clone();
        let email1 = profiles.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        let profiles = get_profiles(USER);
        let nickname1 = profiles.get(0).unwrap().nickname.clone();
        let email1 = profiles.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        let profiles = get_profiles(USER);
        let registrs = RegisTest::registrs(true);
        let nickname1 = registrs.get(0).unwrap().nickname.clone();
        let email1 = registrs.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        let profiles = get_profiles(USER);
        let registrs = RegisTest::registrs(true);
        let nickname1 = registrs.get(0).unwrap().nickname.clone();
        let email1 = registrs.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        let profiles = get_profiles(USER);
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let nickname = "Mary_Williams".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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
        let registrs = RegisTest::registrs(true);
        let user_registr1 = registrs.get(0).unwrap().clone();
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
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

        let registr_profile_resp: RegistrProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
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
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
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
        let profiles = get_profiles(USER);
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
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegisTest::cfg_registr_orm(registrs))
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
        let profiles = get_profiles(USER);
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
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegisTest::cfg_registr_orm(registrs))
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
        let profiles = get_profiles(USER);
        let registrs = RegisTest::registrs(true);
        let user_reg1 = registrs.get(0).unwrap().clone();
        let nickname = user_reg1.nickname.to_string();
        let email = user_reg1.email.to_string();
        let last_user_id = profiles.last().unwrap().user_id;

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
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegisTest::cfg_registr_orm(registrs))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, last_user_id + 1);
        assert_eq!(profile_dto_res.nickname, nickname);
        assert_eq!(profile_dto_res.email, email);
        assert_eq!(profile_dto_res.role, UserRole::User);
    }

    // ** recovery **

    #[actix_web::test]
    async fn test_recovery_no_data() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_recovery_empty_json_object() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery").set_json(json!({}))
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
    async fn test_recovery_invalid_dto_email_empty() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_min() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_max() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_wrong() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_with_email_not_exist() {
        let profiles = get_profiles(USER);
        let email = format!("A{}", profiles.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_FOUND));
        assert_eq!(app_err.message, format!("{}; email: {}", MSG_USER_NOT_FOUND, &email.to_lowercase()));
    }
    #[actix_web::test]
    async fn test_recovery_if_user_recovery_not_exist() {
        let profiles = get_profiles(USER);
        let user1_id = profiles.get(0).unwrap().user_id;
        let user1_email = profiles.get(0).unwrap().email.clone();
        let recoveries = RecovTest::recoveries(Some(user1_id));
        let user_recovery1_id = recoveries.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = token_coding::decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery_id, user_recovery1_id);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_recovery_already_exists() {
        let profiles = get_profiles(USER);
        let user1_id = profiles.get(0).unwrap().user_id;
        let user1_email = profiles.get(0).unwrap().email.clone();
        let recoveries = RecovTest::recoveries(Some(user1_id));
        let user_recovery1_id = recoveries.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = token_coding::decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery1_id, user_recovery_id);
    }
    #[actix_web::test]
    async fn test_recovery_err_jsonwebtoken_encode() {
        let profiles = get_profiles(USER);
        let user1_email = profiles.get(0).unwrap().email.clone();
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
            .configure(RegCtTest::cfg_config_app(config_app::get_test_config()))
            .configure(User_Test::cfg_config_jwt(config_jwt))
            .configure(ProflTest::cfg_profile_orm(profiles))
            .configure(RegCtTest::cfg_mailer(config_smtp::get_test_config()))
            .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
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

    // ** confirm_recovery **

    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_empty() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_min() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_max() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_wrong() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        RegCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_validations::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_recovery_token() {
        let profiles = get_profiles(USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(RecovTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "invalid_recovery_token"))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
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
    async fn test_confirm_recovery_final_date_has_expired() {
        let profiles = get_profiles(USER);
        let user1_id = profiles.get(0).unwrap().user_id;
        let recoveries = RecovTest::recoveries(Some(user1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone();

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, -recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
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
    async fn test_confirm_recovery_no_exists_in_user_recovery() {
        let profiles = get_profiles(USER);
        let user1_id = profiles.get(0).unwrap().user_id;
        let recoveries = RecovTest::recoveries(Some(user1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone() + 1;

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_FOUND));
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, recovery1_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user() {
        let profiles = get_profiles(USER);
        let user1_id = profiles.get(0).unwrap().user_id + 1;
        let mut recoveries = RecovTest::recoveries(Some(user1_id));
        let recovery1 = recoveries.get_mut(0).unwrap();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
        recovery1.final_date = final_date_utc;

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(recovery1.id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_FOUND));
        assert_eq!(app_err.message, format!("{}; user_id: {}", MSG_USER_NOT_FOUND, user1_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_success() {
        let profiles = get_profiles(USER);
        let profile1 = profiles.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile1_dto = ProfileDto::from(profile1);

        let recoveries = RecovTest::recoveries(Some(profile1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone();
        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();

        let num_token1 = User_Test::get_num_token(USER1_ID);
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(profiles))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile1_dto_ser.id);
        assert_eq!(profile_dto_res.nickname, profile1_dto_ser.nickname);
        assert_eq!(profile_dto_res.email, profile1_dto_ser.email);
        assert_eq!(profile_dto_res.role, profile1_dto_ser.role);
        assert_eq!(profile_dto_res.avatar, profile1_dto_ser.avatar);
        assert_eq!(profile_dto_res.descript, profile1_dto_ser.descript);
        assert_eq!(profile_dto_res.theme, profile1_dto_ser.theme);
        assert_eq!(profile_dto_res.created_at, profile1_dto_ser.created_at);
    }

    // ** clear_for_expired **

    #[actix_web::test]
    async fn test_clear_for_expired_user_recovery() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN]);
        let user1_id = data_u.0.get(0).unwrap().id;

        let config_app = config_app::get_test_config();

        let registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
        let mut registrs = RegisTest::registrs(true);
        let registr1 = registrs.get_mut(0).unwrap();
        registr1.final_date = Utc::now() - Duration::seconds(registr_duration);

        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let mut recoveries = RecovTest::recoveries(Some(user1_id));
        let recovery1 = recoveries.get_mut(0).unwrap();
        recovery1.final_date = Utc::now() - Duration::seconds(recovery_duration);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(clear_for_expired)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(RegisTest::cfg_registr_orm(registrs))
                .configure(RecovTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&"/api/clear_for_expired")
            .insert_header(RegCtTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto: ClearForExpiredResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto.count_inactive_registr, 1);
        assert_eq!(response_dto.count_inactive_recover, 1);
    }
}

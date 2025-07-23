#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, web, App,
    };
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;
    use vrb_tools::{
        api_error::{code_to_str, ApiError},
        send_email::{config_smtp, mailer::tests::MailerApp},
        token_data::BEARER,
        token_coding,
    };

    use crate::profiles::{
        profile_err as p_err,
        profile_models::{
            self, ClearForExpiredResponseDto, Profile, ProfileDto, ProfileTest, RecoveryDataDto, RecoveryProfileDto,
            RecoveryProfileResponseDto, RegistrProfileDto, RegistrProfileResponseDto,
        },
        profile_orm::tests::ProfileOrmApp,
        profile_registr_controller::{
            clear_for_expired, confirm_recovery, confirm_registration, recovery, registration, MSG_RECOVERY_NOT_FOUND,
            MSG_REGISTR_NOT_FOUND, MSG_USER_NOT_FOUND,
        },
    };
    use crate::sessions::{
        config_jwt,
        session_models::Session,
        session_orm::tests::SessionOrmApp,
    };
    use crate::settings::{config_app, err};
    use crate::users::{
        user_models::{UserRecovery, UserRegistr, UserRole},
        user_recovery_orm::tests::UserRecoveryOrmApp,
        user_registr_orm::tests::UserRegistrOrmApp,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_profile() -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role)
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_user_registr() -> UserRegistr {
        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);

        let user_registr = UserRegistrOrmApp::new_user_registr(1, "Robert_Brown", "Robert_Brown@gmail.com", "passwdR2B2", final_date);
        user_registr
    }
    fn user_registr_with_id(user_registr: UserRegistr) -> UserRegistr {
        let user_reg_orm = UserRegistrOrmApp::create(&vec![user_registr]);
        user_reg_orm.user_registr_vec.get(0).unwrap().clone()
    }
    fn create_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
        UserRecoveryOrmApp::new_user_recovery(id, user_id, final_date)
    }
    fn create_user_recovery_with_id(user_recovery: UserRecovery) -> UserRecovery {
        let user_recovery_orm = UserRecoveryOrmApp::create(&vec![user_recovery]);
        user_recovery_orm.user_recovery_vec.get(0).unwrap().clone()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data(is_registr: bool, opt_recovery_duration: Option<i64>) -> (
        (config_app::ConfigApp, config_jwt::ConfigJwt), 
        (Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<UserRecovery>),
        String
    ) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile());
        let user_id = profile1.user_id;
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = token_coding::encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let user_registr_vec:Vec<UserRegistr> = if is_registr {
            vec![user_registr_with_id(create_user_registr())]
        } else { vec![] };

        let user_recovery_vec:Vec<UserRecovery> = if let Some(recovery_duration) = opt_recovery_duration {
            let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
            let user_recovery = UserRecoveryOrmApp::new_user_recovery(1, user_id, final_date_utc);
            UserRecoveryOrmApp::create(&vec![user_recovery]).user_recovery_vec
        } else { vec![] };

        let config_app = config_app::get_test_config();
        let cfg_c = (config_app, config_jwt);
        let data_c = (vec![profile1], vec![session1], user_registr_vec,  user_recovery_vec);

        (cfg_c, data_c, token)
    }
    fn configure_reg(
        cfg_c: (config_app::ConfigApp, config_jwt::ConfigJwt), // cortege of configurations
        data_c: (Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<UserRecovery>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_app = web::Data::new(cfg_c.0);
            let data_config_jwt = web::Data::new(cfg_c.1);
            let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));

            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.2));
            let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(&data_c.3));

            config
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm));
        }
    }
    fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** registration **
    #[actix_web::test]
    async fn test_registration_no_data() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_users() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let nickname1 = data_c.0.get(0).unwrap().nickname.clone();
        let email1 = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let nickname1 = data_c.0.get(0).unwrap().nickname.clone();
        let email1 = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let nickname1 = data_c.2.get(0).unwrap().nickname.clone();
        let email1 = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let nickname1 = data_c.2.get(0).unwrap().nickname.clone();
        let email1 = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let mut config_jwt = cfg_c.1;
        config_jwt.jwt_secret = "".to_string();
        let cfg_c = (cfg_c.0, config_jwt);
        let nickname = "Mary_Williams".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        assert!(app_err.message.starts_with(&format!("{};", p_err::MSG_JSON_WEB_TOKEN_ENCODE)));
    }
    #[actix_web::test]
    async fn test_registration_new_user() {
        let user_registr1 = user_registr_with_id(create_user_registr());
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = token_coding::encode_token(user_reg1_id, num_token, jwt_secret, -reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let user_reg_id = user_reg1_id + 1;
        let registr_token = token_coding::encode_token(user_reg_id, num_token, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let last_user_id = data_c.0.last().unwrap().user_id;
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let nickname = user_reg1.nickname.to_string();
        let email = user_reg1.email.to_string();

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = token_coding::encode_token(user_reg1.id, num_token, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_with_email_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let email = format!("A{}", data_c.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let user1_id = data_c.0.get(0).unwrap().user_id;
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let final_date_utc = Utc::now() + Duration::seconds(600);
        let user_recovery1 = create_user_recovery_with_id(create_user_recovery(0, user1_id, final_date_utc));
        let user_recovery1_id = user_recovery1.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let user_recovery1_id = user_recovery1.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let mut config_jwt = cfg_c.1;
        config_jwt.jwt_secret = "".to_string();
        let cfg_c = (cfg_c.0, config_jwt);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        assert!(app_err.message.starts_with(&format!("{};", p_err::MSG_JSON_WEB_TOKEN_ENCODE)));
    }

    // ** confirm_recovery **
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_recovery_token() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.3.get(0).unwrap().clone();

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(user_recovery1.id, num_token, jwt_secret, -recovery_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let user_recovery_id = user_recovery1.id + 1;
        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(user_recovery_id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        assert_eq!(app_err.message, format!("{}; user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, user_recovery_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let user_id = data_c.0.get(0).unwrap().user_id + 1;

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
        let user_recovery1 = create_user_recovery_with_id(create_user_recovery(0, user_id, final_date_utc));
        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();

        let data_c = (data_c.0, data_c.1, data_c.2, vec![user_recovery1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        assert_eq!(app_err.message, format!("{}; user_id: {}", MSG_USER_NOT_FOUND, user_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_success() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let profile1_dto = ProfileDto::from(data_c.0.get(0).unwrap().clone());
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();

        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
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
        let (cfg_c, data_c, token) = get_cfg_data(true, Some(600));

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;

        let registr_duration: i64 = cfg_c.0.app_registr_duration.try_into().unwrap();
        let final_date_registr = Utc::now() - Duration::seconds(registr_duration);
        let mut user_registr1 = data_c.2.get(0).unwrap().clone();
        user_registr1.final_date = final_date_registr;

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_recovery = Utc::now() - Duration::seconds(recovery_duration);
        let mut user_recovery1 = data_c.3.get(0).unwrap().clone();
        user_recovery1.final_date = final_date_recovery;
        #[rustfmt::skip]
        let data_c = (vec![profile1], data_c.1, vec![user_registr1], vec![user_recovery1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(clear_for_expired).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&"/api/clear_for_expired")
            .insert_header(header_auth(&token))
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

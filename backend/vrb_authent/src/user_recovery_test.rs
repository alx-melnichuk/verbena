#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{Duration, SecondsFormat, Utc};
    use serde_json::json;
    use vrb_common::{
        api_error::{code_to_str, ApiError}, consts, env_var, err
    };
    use vrb_tools::{config_app, send_email::config_smtp, token_coding};

    use crate::{
        config_jwt,
        user_models::{self, UserMock},
        user_orm::tests::{UserOrmTest, ADMIN, USER, USER1_ID},
        user_recovery_controller::{
            confirm_recovery, recovery, recovery_clear_for_expired, tests as UserRecoveryCtrlTest, MSG_RECOVERY_NOT_FOUND,
            MSG_USER_NOT_FOUND,
        },
        user_recovery_models::{
            ConfirmRecoveryUserResponseDto, RecoveryClearForExpiredResponseDto, RecoveryDataDto, RecoveryUserDto, RecoveryUserResponseDto,
        },
        user_recovery_orm::tests::UserRecoveryOrmTest,
    };

    const TEST_PATH_TEMPLATE: &str = "../templates";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** recovery **

    #[actix_web::test]
    async fn test_recovery_no_data() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
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
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
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
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_min() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: UserMock::email_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_max() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: UserMock::email_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_wrong() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: UserMock::email_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_with_email_not_exist() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        let email = format!("A{}", data_u.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: email.to_string() })
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
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user1_email = data_u.0.get(0).unwrap().email.clone();
        let recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let user_recovery1_id = recoveries.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = token_coding::decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery_id, user_recovery1_id);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_recovery_already_exists() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user1_email = data_u.0.get(0).unwrap().email.clone();
        let recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let user_recovery1_id = recoveries.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
                .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = token_coding::decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery1_id, user_recovery_id);
    }
    #[actix_web::test]
    async fn test_recovery_err_jsonwebtoken_encode() {
        env_var::env_set_var(consts::SMTP_PATH_TEMPLATE, TEST_PATH_TEMPLATE);
        let data_u = UserOrmTest::users(&[USER]);
        let user1_email = data_u.0.get(0).unwrap().email.clone();
        let mut config_jwt = config_jwt::tests::get_config();
        config_jwt.jwt_secret = "".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery)
            .configure(UserRecoveryCtrlTest::cfg_config_app(config_app::get_test_config()))
            .configure(config_jwt::tests::cfg_config_jwt(config_jwt))
            .configure(UserRecoveryCtrlTest::cfg_mailer(config_smtp::get_test_config()))
            .configure(UserOrmTest::cfg_user_orm(data_u))
            .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryUserDto { email: user1_email.to_string() })
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
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
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
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_min() {
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: UserMock::password_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_max() {
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: UserMock::password_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_wrong() {
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: UserMock::password_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        UserRecoveryCtrlTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_recovery_token() {
        let data_u = UserOrmTest::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(UserRecoveryOrmTest::recoveries(None)))
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
        let data_u = UserOrmTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone();

        let num_token1 = config_jwt::tests::get_num_token(USER1_ID);
        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, -recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
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
        let data_u = UserOrmTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone() + 1;

        let num_token1 = config_jwt::tests::get_num_token(USER1_ID);
        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
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
        let data_u = UserOrmTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id + 1;
        let mut recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let recovery1 = recoveries.get_mut(0).unwrap();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
        recovery1.final_date = final_date_utc;

        let num_token1 = config_jwt::tests::get_num_token(USER1_ID);
        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(recovery1.id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
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
        let data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get(0).unwrap();
        let user1_id = user1.id.clone();
        let nickname = user1.nickname.clone();
        let email = user1.email.clone();
        let user1_created_at = user1.created_at.clone();

        let recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let recovery1_id = recoveries.get(0).unwrap().id.clone();
        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();

        let num_token1 = config_jwt::tests::get_num_token(USER1_ID);
        let config_jwt = config_jwt::tests::get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = token_coding::encode_token(recovery1_id, num_token1, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
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
        let response_dto_res: ConfirmRecoveryUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto_res.id, user1_id);
        assert_eq!(response_dto_res.nickname, nickname);
        assert_eq!(response_dto_res.email, email);
        assert_eq!(response_dto_res.created_at, user1_created_at);
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let now_str = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(response_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true), now_str);
    }

    // ** recovery_clear_for_expired **

    #[actix_web::test]
    async fn test_recovery_clear_for_expired() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN]);
        let user1_id = data_u.0.get(0).unwrap().id;

        let config_app = config_app::get_test_config();

        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let mut recoveries = UserRecoveryOrmTest::recoveries(Some(user1_id));
        let recovery1 = recoveries.get_mut(0).unwrap();
        recovery1.final_date = Utc::now() - Duration::seconds(recovery_duration);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery_clear_for_expired)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(UserRecoveryOrmTest::cfg_recovery_orm(recoveries))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/recovery/clear_for_expired")
            .insert_header(UserRecoveryCtrlTest::header_auth(&token1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto: RecoveryClearForExpiredResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto.count_inactive_recover, 1);
    }
}

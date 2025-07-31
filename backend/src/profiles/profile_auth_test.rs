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
    use vrb_common::api_error::{code_to_str, ApiError, check_app_err};
    use vrb_tools::{
        err, hash_tools,
        token_coding,
        token_data::TOKEN_NAME,
    };

    use crate::profiles::{
        profile_auth_controller::{login, logout, update_token},
        profile_models::{self, LoginProfileDto, LoginProfileResponseDto, ProfileDto, ProfileTest, TokenDto},
        profile_orm::tests::{ProfileOrmTest as PrfTest, PROFILE_USER_ID_NO_SESSION, USER},
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** login **
    #[actix_web::test]
    async fn test_login_no_data() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_min() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_max() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_wrong() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_min() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_max() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_wrong() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_empty() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_min() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_max() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_wrong() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_if_nickname_not_exist() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let nickname = data_p.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: format!("a{}", nickname), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401 (A)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_email_not_exist() {
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let email = data_p.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: format!("a{}", email), password: "passwordD1T1".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401 (A)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_password_invalid_hash() {
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        let password = "hash_password_R2B2";
        profile1.password = password.to_string();
        let nickname = profile1.nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.nickname = nickname.clone().to_lowercase();
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: format!("{}b", password) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401 (B)

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_login_err_jsonwebtoken_encode() {
        let (mut cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.nickname = nickname.clone().to_lowercase();
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        cfg_p.0.jwt_secret = "".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1 = data_p.0.last_mut().unwrap();
        // Change ID, reset connection with session.
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1_id = PROFILE_USER_ID_NO_SESSION;
        profile1.user_id = profile1_id;
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        profile1.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let profile1_dto = ProfileDto::from(profile1.clone());
        let jwt_access = cfg_p.0.jwt_access;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(PrfTest::config(cfg_p, data_p))).await;
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
    async fn test_logout_valid_token() {
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1_id = data_p.0.get(0).unwrap().user_id;
        let jwt_secret = cfg_p.0.jwt_secret.as_bytes();
        let profile_id_bad = profile1_id + 1;
        let num_token = data_p.1.get(0).unwrap().num_token.unwrap();
        let token_bad = token_coding::encode_token(profile_id_bad, num_token, &jwt_secret, cfg_p.0.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_p))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_c) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let jwt_secret = cfg_p.0.jwt_secret.as_bytes();
        let num_token = data_c.1.get(0).unwrap().num_token.unwrap();
        let token_bad = token_coding::encode_token(profile1_id, num_token + 1, &jwt_secret, cfg_p.0.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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
        let token = PrfTest::token1();
        let (cfg_p, data_c) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let jwt_access = cfg_p.0.jwt_access;
        let jwt_secret = cfg_p.0.jwt_secret.as_bytes();
        let num_token = data_c.1.get(0).unwrap().num_token.unwrap();
        let token_refresh = token_coding::encode_token(profile1_id, num_token, &jwt_secret, cfg_p.0.jwt_refresh).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(PrfTest::config(cfg_p, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(PrfTest::header_auth(&token))
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

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        cookie::Cookie,
        dev, get,
        http::{header, StatusCode},
        test, App, HttpResponse,
    };
    use vrb_common::api_error::{code_to_str, ApiError};
    use vrb_tools::{err, token_coding, token_data};

    use crate::extractors::authentication2::RequireAuth2;
    use vrb_dbase::user_auth::{
        config_jwt,
        user_auth_models::Session,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, ADMIN, USER, USER1_ID},
    };

    const MSG_ERROR_WAS_EXPECTED: &str = "Service call succeeded, but an error was expected.";
    const MSG_FAILED_TO_DESER: &str = "Failed to deserialize JSON string";

    #[get("/", wrap = "RequireAuth2::allowed_roles(RequireAuth2::all_roles())")]
    async fn handler_with_auth() -> HttpResponse {
        HttpResponse::Ok().into()
    }
    #[get("/", wrap = "RequireAuth2::allowed_roles(RequireAuth2::admin_role())")]
    async fn handler_with_require_only_admin() -> HttpResponse {
        HttpResponse::Ok().into()
    }
    fn header_auth(token: &str) -> (header::HeaderName, header::HeaderValue) {
        let header_value = header::HeaderValue::from_str(&format!("{}{}", token_data::BEARER, token)).unwrap();
        (header::AUTHORIZATION, header_value)
    }

    #[actix_web::test]
    async fn test_authentication_middelware_valid_token() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK); // 200
    }
    #[actix_web::test]
    async fn test_authentication_middelware_valid_token_with_cookie() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().cookie(Cookie::new("token", token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK); // 200
    }
    #[actix_web::test]
    async fn test_authentication_middleware_access_admin_only_endpoint_success() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_require_only_admin)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK); // 200
    }
    #[actix_web::test]
    async fn test_authentication_middleware_missing_token() {
        let data_u = User_Test::users(&[]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(a)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, err::MSG_MISSING_TOKEN);
    }
    #[actix_web::test]
    async fn test_authentication_middleware_invalid_token() {
        let data_u = User_Test::users(&[]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth("invalid_token123")).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401b

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert!(api_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_authentication_middelware_expired_token() {
        let data_u = User_Test::users(&[USER]);
        let config_jwt = config_jwt::get_test_config();
        let user1_id = data_u.0.get(0).unwrap().id;
        let num_token1 = data_u.1.get(0).unwrap().num_token.unwrap();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token1 = token_coding::encode_token(user1_id, num_token1, &jwt_secret, -config_jwt.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token1)).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(b)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, format!("{}: ExpiredSignature", err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_authentication_middelware_valid_token_session_non_exist() {
        let data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token2)).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::NOT_ACCEPTABLE); // 406

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user2_id));
    }
    #[actix_web::test]
    async fn test_authentication_middelware_valid_token_non_existent_user() {
        let mut data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        data_u.1 = vec![Session::new(user2_id, Some(User_Test::get_num_token(user2_id)))];
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token2)).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(d)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_ID, user2_id));
    }
    #[actix_web::test]
    async fn test_authentication_middelware_valid_token_non_existent_num() {
        let mut data_u = User_Test::users(&[USER]);
        let user2_id = USER1_ID + 1;
        data_u.1 = vec![Session::new(user2_id, Some(User_Test::get_num_token(USER1_ID)))];
        let token2 = User_Test::get_token(user2_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_auth)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token2)).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::UNAUTHORIZED); // 401(c)

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user2_id));
    }
    #[actix_web::test]
    async fn test_authentication_middleware_failure_access_only_admin() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(handler_with_require_only_admin)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
        ).await;
        let req = test::TestRequest::get().insert_header(header_auth(&token1)).to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect(MSG_ERROR_WAS_EXPECTED);

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, StatusCode::FORBIDDEN); // 403

        let api_err: ApiError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_TO_DESER);
        assert_eq!(api_err.code, code_to_str(StatusCode::FORBIDDEN));
        assert_eq!(api_err.message, err::MSG_ACCESS_DENIED);
    }
}

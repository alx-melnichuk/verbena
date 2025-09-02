#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
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
        user_authent_controller::users_uniqueness,
        user_authent_models::UserUniquenessResponseDto,
        user_orm::tests::{UserOrmTest as User_Test, USER},
        user_registr_orm::tests::UserRegistrOrmTest as RegisTest,
    };

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
}

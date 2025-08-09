#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use serde_json;
    use vrb_common::api_error::{code_to_str, ApiError};
    use vrb_tools::err;

    use crate::profiles::{
        config_jwt, config_prfl,
        profile_controller::{get_profile_by_id, get_profile_config, get_profile_current, tests as RrfCtTest, uniqueness_check},
        profile_models::{ProfileConfigDto, ProfileDto},
        profile_orm::tests::{ProfileOrmTest as ProflTest, ADMIN, USER, USER1_ID},
    };
    use crate::users::user_registr_orm::tests::UserRegistrOrmTest as RegisTest;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** get_profile_by_id **

    #[actix_web::test]
    async fn test_get_profile_by_id_invalid_id() {
        let token1 = ProflTest::get_token(USER1_ID);
        let data_p = ProflTest::profiles(&[ADMIN]);
        let user_id = data_p.0.get(0).unwrap().user_id;
        let user_id_bad = format!("{}a", user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", user_id_bad))
            .insert_header(RrfCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, user_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_profile_by_id_valid_id() {
        let token1 = ProflTest::get_token(USER1_ID);
        let data_p = ProflTest::profiles(&[ADMIN, USER]);
        let profile2_dto = ProfileDto::from(data_p.0.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", &profile2_id))
            .insert_header(RrfCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile2_dto).to_string();
        let profile2b_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile2b_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_profile_by_id_non_existent_id() {
        let token1 = ProflTest::get_token(USER1_ID);
        let data_p = ProflTest::profiles(&[ADMIN, USER]);
        let profile2_dto = ProfileDto::from(data_p.0.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", profile2_id + 1))
            .insert_header(RrfCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }

    // ** get_profile_config **

    #[actix_web::test]
    async fn test_get_profile_config_data() {
        let token1 = ProflTest::get_token(USER1_ID);
        let data_p = ProflTest::profiles(&[USER]);
        let cfg_prfl = config_prfl::get_test_config();
        #[rustfmt::skip]
        let profile_config_dto = ProfileConfigDto::new(
            if cfg_prfl.prfl_avatar_max_size > 0 { Some(cfg_prfl.prfl_avatar_max_size) } else { None },
            cfg_prfl.prfl_avatar_valid_types.clone(),
            cfg_prfl.prfl_avatar_ext.clone(),
            if cfg_prfl.prfl_avatar_max_width > 0 { Some(cfg_prfl.prfl_avatar_max_width) } else { None },
            if cfg_prfl.prfl_avatar_max_height > 0 { Some(cfg_prfl.prfl_avatar_max_height) } else { None },
        );
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_config)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_config")
            .insert_header(RrfCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_config_dto_res: ProfileConfigDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(profile_config_dto_res, profile_config_dto);
    }

    // ** get_profile_current **

    #[actix_web::test]
    async fn test_get_profile_current_valid_token() {
        let token1 = ProflTest::get_token(USER1_ID);
        let data_p = ProflTest::profiles(&[USER]);
        let profile1_dto = ProfileDto::from(data_p.0.get(0).unwrap().clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_current)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_current")
            .insert_header(RrfCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
    }

    // ** uniqueness_check **

    #[actix_web::test]
    async fn test_uniqueness_check_by_non_params() {
        let data_p = ProflTest::profiles(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness")
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
    async fn test_uniqueness_check_by_nickname_empty() {
        let data_p = ProflTest::profiles(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness?nickname=")
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
    async fn test_uniqueness_check_by_email_empty() {
        let data_p = ProflTest::profiles(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness?email=")
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
    async fn test_uniqueness_check_by_nickname_profile() {
        let data_p = ProflTest::profiles(&[USER]);
        let nickname = data_p.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_email_profile() {
        let data_p = ProflTest::profiles(&[USER]);
        let email = data_p.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_nickname_registr() {
        let data_p = ProflTest::profiles(&[USER]);
        let registr = RegisTest::registrs(true);
        let nickname = registr.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_email_registr() {
        let data_p = ProflTest::profiles(&[USER]);
        let registr = RegisTest::registrs(true);
        let email = registr.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_new_nickname() {
        let data_p = ProflTest::profiles(&[USER]);
        let nickname = format!("a{}", data_p.0.get(0).unwrap().nickname.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":true}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_new_email() {
        let data_p = ProflTest::profiles(&[USER]);
        let email = format!("a{}", data_p.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check)
                .configure(ProflTest::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(ProflTest::cfg_profile_orm(data_p))
                .configure(RegisTest::cfg_registr_orm(RegisTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":true}");
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        App, body, dev,
        http::StatusCode,
        http::header::{CONTENT_TYPE, HeaderValue},
        test,
    };
    use serde_json;
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{ADMIN, USER, USER1_ID, UserOrmTest},
    };
    use vrb_common::{
        api_error::{ApiError, code_to_str},
        err,
    };

    use crate::{
        config_prfl,
        profile_controller::{get_profile_by_id, get_profile_config, get_profile_current, get_profile_mini_by_id, tests as ProfileCtrlTest},
        profile_models::{ProfileConfigDto, UserProfileDto, UserProfileMiniDto},
        profile_orm::tests::ProfileOrmTest,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** get_profile_by_id **

    #[actix_web::test]
    async fn test_get_profile_by_id_invalid_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let user_id = data_u.0.get(0).unwrap().id;
        let user_id_bad = format!("{}a", user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", user_id_bad))
            .insert_header(ProfileCtrlTest::header_auth(&token1)).to_request();
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
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN, USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile2_dto = UserProfileDto::from(profiles.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", &profile2_id))
            .insert_header(ProfileCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile2_dto).to_string();
        let profile2b_dto_ser: UserProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile2b_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_profile_by_id_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[ADMIN, USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile2_id = profiles.get(1).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", profile2_id + 1))
            .insert_header(ProfileCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }

    // ** get_profile_mini_by_id (Without authorization.) ** 

    #[actix_web::test]
    async fn test_get_profile_mini_by_id_invalid_id() {
        let data_u = UserOrmTest::users(&[ADMIN]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let user_id = data_u.0.get(0).unwrap().id;
        let user_id_bad = format!("{}a", user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_mini_by_id)
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_mini/{}", user_id_bad))
            .to_request();
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
    async fn test_get_profile_mini_by_id_valid_id() {
        let data_u = UserOrmTest::users(&[ADMIN, USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile2_mini_dto = UserProfileMiniDto::from(profiles.get(1).unwrap().clone());
        let profile2_id = profile2_mini_dto.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_mini_by_id)
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_mini/{}", &profile2_id))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_mini_dto_res: UserProfileMiniDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile2_mini_dto).to_string();
        let profile2b_mini_dto_ser: UserProfileMiniDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_mini_dto_res, profile2b_mini_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_profile_mini_by_id_non_existent_id() {
        let data_u = UserOrmTest::users(&[ADMIN, USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile2_id = profiles.get(1).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_mini_by_id)
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_mini/{}", profile2_id + 1))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }

    // ** get_profile_config **

    #[actix_web::test]
    async fn test_get_profile_config_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
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
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_config")
            .insert_header(ProfileCtrlTest::header_auth(&token1)).to_request();
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
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1_dto = UserProfileDto::from(profiles.get(0).unwrap().clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_current)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_current")
            .insert_header(ProfileCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: UserProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
    }
}

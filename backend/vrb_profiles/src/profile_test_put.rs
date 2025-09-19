#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{borrow::Cow, fs, path, thread::sleep, time::Duration};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        App, body, dev,
        http::StatusCode,
        http::header::{CONTENT_TYPE, HeaderValue},
        test,
    };
    use chrono::{SecondsFormat, Utc};
    use serde_json;
    use vrb_authent::{
        config_jwt, user_models,
        user_models::UserMock,
        user_orm::tests::{USER, USER1_ID, UserOrmTest},
        user_registr_orm::tests::UserRegistrOrmTest,
    };
    use vrb_common::{
        api_error::{ApiError, code_to_str},
        consts, err, profile, validators,
    };
    use vrb_dbase::enm_user_role::UserRole;
    use vrb_tools::{cdis::coding, hash_tools, png_files};

    use crate::{
        config_prfl,
        profile_controller::{
            put_profile, put_profile_new_password,
            tests::{self as ProfileCtrlTest, check_app_err},
        },
        profile_models::{ModifyUserProfileDto, NewPasswordUserProfileDto, ProfileMock, UserProfile, UserProfileDto},
        profile_orm::tests::ProfileOrmTest,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_MULTIPART_STREAM_INCOMPLETE: &str = "Multipart stream is incomplete";
    const MSG_CONTENT_TYPE_ERROR: &str = "Could not find Content-Type header";

    const DELAY_IN_MILLISECS: u64 = 30;

    fn sleep_by_milli_secs(milli_secs: u64) {
        let three_secs = Duration::from_millis(milli_secs);
        sleep(three_secs);
    }

    // ** put_profile **

    #[actix_web::test]
    async fn test_put_profile_no_form() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_put_profile_empty_form() {
        let (header, body) = MultiPartFormDataBuilder::new().build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_MULTIPART_STREAM_INCOMPLETE));
    }
    #[actix_web::test]
    async fn test_put_profile_form_with_invalid_name() {
        let name1_file = "test_put_profile_form_with_invalid_name.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 2).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile1", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.message, err::MSG_NO_FIELDS_TO_UPDATE);
        let key = Cow::Borrowed(validators::NM_NO_FIELDS_TO_UPDATE);
        #[rustfmt::skip]
        let names1 = app_err.params.get(&key).unwrap().get("validNames").unwrap().as_str().unwrap();
        let names2 = [ModifyUserProfileDto::valid_names(), vec!["avatarfile"]].concat().join(",");
        assert_eq!(names1, &names2);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", UserMock::nickname_min()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", UserMock::nickname_max()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", UserMock::nickname_wrong()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", UserMock::email_min()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", UserMock::email_max()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", UserMock::email_wrong()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_put_profile_role_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("role", UserMock::role_wrong()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_USER_ROLE_INVALID_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("descript", ProfileMock::descript_min()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("descript", ProfileMock::descript_max()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("theme", ProfileMock::theme_min()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_THEME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("theme", ProfileMock::theme_max()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_THEME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_locale_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("locale", ProfileMock::locale_min()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_LOCALE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_locale_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("locale", ProfileMock::locale_max()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile::MSG_LOCALE_MAX_LENGTH]);
    }

    #[actix_web::test]
    async fn test_put_profile_if_nickname_exists_in_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let nickname1 = data_u.0.get(0).unwrap().nickname.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", nickname1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_if_email_exists_in_users() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let email1 = data_u.0.get(0).unwrap().email.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", email1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(true)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_if_nickname_exists_in_registr() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let registr = UserRegistrOrmTest::registrs(true);
        let nickname1 = registr.get(0).unwrap().nickname.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", nickname1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_if_email_exists_in_registr() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let registr = UserRegistrOrmTest::registrs(true);
        let email1 = registr.get(0).unwrap().email.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", email1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(registr))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_invalid_file_size() {
        let name1_file = "test_put_profile_invalid_file_size.png";
        let path_name1_file = format!("./{}", &name1_file);
        let (size, _name) = png_files::save_file_png(&path_name1_file, 2).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let mut config_prfl = config_prfl::get_test_config();
        let prfl_avatar_max_size = 160;
        config_prfl.prfl_avatar_max_size = prfl_avatar_max_size;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::PAYLOAD_TOO_LARGE));
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_SIZE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileSize": size, "maxFileSize": prfl_avatar_max_size });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_profile_invalid_file_type() {
        let name1_file = "test_put_profile_invalid_file_type.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 1).unwrap();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/bmp", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNSUPPORTED_MEDIA_TYPE));
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_TYPE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileType": "image/bmp", "validFileType": "image/jpeg,image/png" });
        assert_eq!(*app_err.params.get("invalidFileType").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_profile_valid_data_without_file() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);

        let profile = profiles.get(0).unwrap().clone();
        let nickname_s = format!("{}_a", profile.nickname.clone());
        let email_s = format!("{}_a", profile.email.clone());
        let user_role = UserRole::Admin;
        let descript_s = format!("{}_a", profile.descript.clone().unwrap_or("default".to_string()));
        let theme_s = format!("{}_a", profile.theme.clone().unwrap_or("default".to_string()));

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", &nickname_s)
            .with_text("email", &email_s)
            .with_text("role", &user_role.to_string())
            .with_text("descript", &descript_s)
            .with_text("theme", &theme_s)
            .build();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile.user_id);
        assert_eq!(profile_dto_res.nickname, nickname_s);
        assert_eq!(profile_dto_res.email, email_s);
        assert_eq!(profile_dto_res.role, user_role);
        assert_eq!(profile_dto_res.descript, Some(descript_s));
        assert_eq!(profile_dto_res.theme, Some(theme_s));
        // DateTime.to_rfc3339_opts(SecondsFormat::Millis, true) => "2018-01-26T18:30:09.113Z"
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true)   => "2018-01-26T18:30:09Z"
        let old_created_at = profile.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(profile_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true), old_created_at);
        let old_updated_at = profile.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        #[rustfmt::skip]
        assert_eq!(profile_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true), old_updated_at);
    }
    #[actix_web::test]
    async fn test_put_profile_a_with_old0_new1() {
        let name1_file = "test_put_profile_a_with_old0_new1.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 1).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1_id = profiles.get(0).unwrap().user_id;
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(consts::ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(consts::ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_b_with_old0_new1_convert() {
        let name1_file = "test_put_profile_b_with_old0_new1_convert.png";
        let path_name1_file = format!("./{}", name1_file);
        png_files::save_file_png(&path_name1_file, 3).unwrap();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);

        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1_id = profiles.get(0).unwrap().user_id;
        let file_ext = "jpeg".to_string();
        let mut config_prfl = config_prfl::get_test_config();
        config_prfl.prfl_avatar_ext = Some(file_ext.clone());
        config_prfl.prfl_avatar_max_width = 18;
        config_prfl.prfl_avatar_max_height = 18;
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(consts::ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let path = path::Path::new(&img_name_full_path);
        let receiver_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
        let is_exists_img_new = path.exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert_eq!(file_ext, receiver_ext);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(consts::ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_c_with_old1_new1() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_c_with_old1_new1.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", consts::ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_c_with_old1_new1_new.png";
        let path_name1_file = format!("./{}", name1_file);
        png_files::save_file_png(&path_name1_file, 1).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let mut profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1 = profiles.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias.clone());
        let profile1_id = profile1.user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(consts::ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(consts::ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());

        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_d_with_old1_new0() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_d_with_old1_new0.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&path_name0_file, 1).unwrap();
        let path_name0_alias = format!("{}/{}", consts::ALIAS_AVATAR_FILES_DIR, name0_file);

        let (header, body) = MultiPartFormDataBuilder::new().with_text("descript", "descript1".to_string()).build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let mut profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1 = profiles.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(path_name0_file.clone());
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(consts::ALIAS_AVATAR_FILES_DIR));
        assert_eq!(&path_name0_alias, &profile_dto_res_img);
    }
    #[actix_web::test]
    async fn test_put_profile_e_with_old_1_new_size0() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_e_with_old_1_new_size0.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", consts::ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_e_with_old_1_new_size0_new.png";
        let path_name1_file = format!("./{}", name1_file);
        png_files::save_empty_file(&path_name1_file).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let mut profiles = ProfileOrmTest::profiles(&data_u.0);
        let profile1 = profiles.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
    }
    #[actix_web::test]
    async fn test_put_profile_f_with_old0_new_size0() {
        let name1_file = "test_put_profile_f_with_old0_new_size0.png";
        let path_name1_file = format!("./{}", name1_file);
        png_files::save_empty_file(&path_name1_file).unwrap();
        sleep_by_milli_secs(DELAY_IN_MILLISECS);

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
                .configure(ProfileOrmTest::cfg_config_prfl(config_prfl::get_test_config()))
                .configure(UserRegistrOrmTest::cfg_registr_orm(UserRegistrOrmTest::registrs(false)))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
    }

    // ** put_profile_new_password **

    #[actix_web::test]
    async fn test_put_profile_new_password_no_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
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
    async fn test_put_profile_new_password_empty_json_object() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(serde_json::json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_password_empty() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: "".to_string(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_password_min() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: UserMock::password_min(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_password_max() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: UserMock::password_max(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED),
            &[user_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_password_wrong() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: UserMock::password_wrong(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_empty() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password, new_password: "".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NEW_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_min() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password, new_password: UserMock::password_min()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NEW_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_max() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password, new_password: UserMock::password_max()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NEW_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_wrong() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password, new_password: UserMock::password_wrong()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NEW_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_equal_old_value() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password.clone(), new_password: old_password
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[user_models::MSG_NEW_PASSWORD_EQUAL_OLD_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_hash_password() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = "invali_hash_password".to_string();
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password.to_string(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert!(app_err.message.starts_with(err::MSG_INVALID_HASH));
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_password() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: format!("{}a", old_password), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_valid_data() {
        let old_password = "passwdP1C1".to_string();
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.password = hash_tools::encode_hash(old_password.clone()).unwrap(); // hashed
        let user1_profile = UserProfile::from(user1.clone());
        let user1_profile_dto = UserProfileDto::from(user1_profile);
        let profiles = ProfileOrmTest::profiles(&data_u.0);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(ProfileOrmTest::cfg_profile_orm(profiles))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(ProfileCtrlTest::header_auth(&token1))
            .set_json(NewPasswordUserProfileDto {
                password: old_password.to_string(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: UserProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(user1_profile_dto).to_string();
        let profile_dto_ser: UserProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile_dto_ser.id);
        assert_eq!(profile_dto_res.nickname, profile_dto_ser.nickname);
        assert_eq!(profile_dto_res.email, profile_dto_ser.email);
        assert_eq!(profile_dto_res.role, profile_dto_ser.role);
        assert_eq!(profile_dto_res.avatar, profile_dto_ser.avatar);
        assert_eq!(profile_dto_res.descript, profile_dto_ser.descript);
        assert_eq!(profile_dto_res.theme, profile_dto_ser.theme);
        assert_eq!(profile_dto_res.created_at, profile_dto_ser.created_at);
    }
}

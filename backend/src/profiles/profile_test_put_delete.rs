#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{borrow::Cow, fs, path};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        body, dev, http::{header::{HeaderValue, CONTENT_TYPE}, StatusCode}, test, App,
    };
    use chrono::{SecondsFormat, Utc};
    use serde_json;
    use vrb_tools::{api_error::{ApiError, code_to_str}, cdis::coding, validators};

    use crate::profiles::{
        config_prfl,
        profile_controller::{
            delete_profile, delete_profile_current, put_profile, put_profile_new_password,
            tests::{
                check_app_err, configure_profile, create_profile, get_cfg_data, header_auth, save_empty_file,
                save_file_png, ADMIN, MSG_CONTENT_TYPE_ERROR, MSG_FAILED_DESER, MSG_MULTIPART_STREAM_INCOMPLETE, USER,
            },
            ALIAS_AVATAR_FILES_DIR,
        },
        profile_models::{self, ModifyProfileDto, NewPasswordProfileDto, Profile, ProfileDto, ProfileTest},
    };
    use crate::settings::err;
    use crate::users::user_models::UserRole;

    // ** put_profile **

    #[actix_web::test]
    async fn test_put_profile_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile1", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let names2 = [ModifyProfileDto::valid_names(), vec!["avatarfile"]].concat().join(",");
        assert_eq!(names1, &names2);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_nickname_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_nickname_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_email_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_email_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

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
    async fn test_put_profile_role_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("role", ProfileTest::role_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_USER_ROLE_INVALID_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", ProfileTest::descript_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", ProfileTest::descript_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("theme", ProfileTest::theme_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_THEME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("theme", ProfileTest::theme_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_THEME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_locale_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("locale", ProfileTest::locale_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_LOCALE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_locale_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("locale", ProfileTest::locale_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_LOCALE_MAX_LENGTH]);
    }

    #[actix_web::test]
    async fn test_put_profile_if_nickname_exists_in_users() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let nickname1 = data_c.0.get(0).unwrap().nickname.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", nickname1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(true, USER);
        let email1 = data_c.0.get(0).unwrap().email.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", email1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(true, USER);
        let nickname1 = data_c.2.get(0).unwrap().nickname.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("nickname", nickname1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(true, USER);
        let email1 = data_c.2.get(0).unwrap().email.clone();
        let (header, body) = MultiPartFormDataBuilder::new().with_text("email", email1).build();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let (size, _name) = save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let mut config_prfl = config_prfl::get_test_config();
        let prfl_avatar_max_size = 160;
        config_prfl.prfl_avatar_max_size = prfl_avatar_max_size;
        let cfg_c = (cfg_c.0, config_prfl, cfg_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        save_file_png(&path_name1_file, 1).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/bmp", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let profile = data_c.0.get(0).unwrap().clone();
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
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile.user_id);
        assert_eq!(profile_dto_res.nickname, nickname_s);
        assert_eq!(profile_dto_res.email, email_s);
        assert_eq!(profile_dto_res.role, user_role);
        assert_eq!(profile_dto_res.descript, Some(descript_s));
        assert_eq!(profile_dto_res.theme, Some(theme_s));
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = profile_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_created_at = profile.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, old_created_at);
        let res_updated_at = profile_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_updated_at = profile.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, old_updated_at);
    }
    #[actix_web::test]
    async fn test_put_profile_a_with_old0_new1() {
        let name1_file = "test_put_profile_a_with_old0_new1.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let prfl_avatar_files_dir = cfg_c.1.prfl_avatar_files_dir.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
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
        save_file_png(&path_name1_file, 3).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let mut config_prfl = cfg_c.1.clone();
        let file_ext = "jpeg".to_string();
        config_prfl.prfl_avatar_ext = Some(file_ext.clone());
        config_prfl.prfl_avatar_max_width = 18;
        config_prfl.prfl_avatar_max_height = 18;
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();

        let cfg_c = (cfg_c.0, config_prfl, cfg_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let path = path::Path::new(&img_name_full_path);
        let receiver_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
        let is_exists_img_new = path.exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert_eq!(file_ext, receiver_ext);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
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
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_c_with_old1_new1_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
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
        save_file_png(&path_name0_file, 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", "descript1".to_string())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(path_name0_file.clone());
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
        assert_eq!(&path_name0_alias, &profile_dto_res_img);
    }
    #[actix_web::test]
    async fn test_put_profile_e_with_old_1_new_size0() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_e_with_old_1_new_size0.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_e_with_old_1_new_size0_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
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
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
    }
    #[actix_web::test]
    async fn test_put_profile_f_with_old0_new_size0() {
        let name1_file = "test_put_profile_f_with_old0_new_size0.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
    }

    // ** put_profile_new_password **

    #[actix_web::test]
    async fn test_put_profile_new_password_no_data() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
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
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_password_min() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_min(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_password_max() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_max(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_password_wrong() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_wrong(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_new_password_empty() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NEW_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_min() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_min()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NEW_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_max() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_max()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NEW_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_wrong() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_wrong()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NEW_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_equal_old_value() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
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
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[profile_models::MSG_NEW_PASSWORD_EQUAL_OLD_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_hash_password() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER, None);
        profile1.password = "invali_hash_password".to_string();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;

        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
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
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
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
        let mut profile1: Profile = create_profile(USER, Some(&old_password));
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let profile1_dto = ProfileDto::from(profile1.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2, data_c.3);

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password.to_string(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile_dto_ser.id);
        assert_eq!(profile_dto_res.nickname, profile_dto_ser.nickname);
        assert_eq!(profile_dto_res.email, profile_dto_ser.email);
        assert_eq!(profile_dto_res.role, profile_dto_ser.role);
        assert_eq!(profile_dto_res.avatar, profile_dto_ser.avatar);
        assert_eq!(profile_dto_res.descript, profile_dto_ser.descript);
        assert_eq!(profile_dto_res.theme, profile_dto_ser.theme);
        assert_eq!(profile_dto_res.created_at, profile_dto_ser.created_at);
    }

    // ** delete_profile **

    #[actix_web::test]
    async fn test_delete_profile_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile_id_bad = format!("{}a", data_c.0.get(0).unwrap().user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, profile_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_delete_profile_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_profile_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }

    // ** delete_profile_current **
    #[actix_web::test]
    async fn test_delete_profile_current_without_img() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_current_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_current_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
}

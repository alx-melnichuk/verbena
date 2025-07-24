#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{borrow::Cow, fs, path};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{Duration, SecondsFormat, Utc};
    use serde_json;
    use vrb_dbase::{db_enums::{UserRole, StreamState}};
    use vrb_tools::{
        api_error::{code_to_str, ApiError},
        cdis::coding,
        err, validators,
    };

    use crate::profiles::profile_orm::tests::ProfileOrmApp;
    use crate::streams::{
        config_strm,
        stream_controller::{
            put_stream, put_toggle_state,
            tests::{
                check_app_err, configure_stream, create_stream, get_cfg_data, header_auth, save_empty_file, save_file_png,
                MSG_CASTING_TO_TYPE, MSG_CONTENT_TYPE_NOT_FOUND, MSG_FAILED_DESER, MSG_MULTIPART_STREAM_INCOMPLETE,
            },
            ALIAS_LOGO_FILES_DIR, MSG_EXIST_IS_ACTIVE_STREAM, MSG_INVALID_FIELD_TAG, MSG_INVALID_STREAM_STATE,
        },
        stream_models::{self, ModifyStreamInfoDto, StreamInfoDto, StreamModelsTest, ToggleStreamStateDto},
        stream_orm::tests::StreamOrmApp,
    };

    // ** put_stream **

    #[actix_web::test]
    async fn test_put_stream_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/1"))
            .insert_header(header_auth(&token))
            .to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_NOT_FOUND));
    }
    #[actix_web::test]
    async fn test_put_stream_empty_form() {
        let (header, body) = MultiPartFormDataBuilder::new().build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/1"))
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
    async fn test_put_stream_invalid_name() {
        let name1_file = "test_put_stream_invalid_name.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile1", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1")
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
        let names2 = [ModifyStreamInfoDto::valid_names(), vec!["logofile"]].concat().join(",");
        assert_eq!(names1, &names2);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_id() {
        let stream_id_bad = "100a".to_string();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", "".to_string()).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", &stream_id_bad))
            .insert_header(header_auth(&token)).insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        let error = format!("{} ({})", "invalid digit found in string", stream_id_bad);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &error));
    }
    #[actix_web::test]
    async fn test_put_stream_title_min() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("title", StreamModelsTest::title_min()).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TITLE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_title_max() {
        let (header, body) = MultiPartFormDataBuilder::new().with_text("title", StreamModelsTest::title_max()).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TITLE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_descript_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", StreamModelsTest::descript_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_descript_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", StreamModelsTest::descript_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_starttime_now() {
        let starttime_s = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("starttime", starttime_s).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_MIN_VALID_STARTTIME]);
    }
    #[actix_web::test]
    async fn test_put_stream_source_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("source", StreamModelsTest::source_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_SOURCE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_source_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("source", StreamModelsTest::source_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_SOURCE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_tags_min_amount() {
        let tags = StreamModelsTest::tag_names_min();
        if tags.len() <= 0 {
            return;
        }
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MIN_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_put_stream_tags_max_amount() {
        let tags = StreamModelsTest::tag_names_max();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MAX_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_put_stream_tag_name_min() {
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_min()];
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_tag_name_max() {
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_max()];
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_tag() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", "aaa").build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; {}", MSG_INVALID_FIELD_TAG, "expected value at line 1 column 1");
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_tag_vec() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", "[\"tag\"").build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        let message = format!("{}; {}", MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6");
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_file_size() {
        let name1_file = "test_put_stream_invalid_file_size.png";
        let path_name1_file = format!("./{}", &name1_file);
        let (size, _name) = save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut config_strm = config_strm::get_test_config();
        config_strm.strm_logo_max_size = 160;
        let cfg_c = (cfg_c.0, config_strm);
        let strm_logo_max_size = cfg_c.1.strm_logo_max_size;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1")
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
        let json = serde_json::json!({ "actualFileSize": size, "maxFileSize": strm_logo_max_size });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_file_type() {
        let name1_file = "test_put_stream_invalid_file_type.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/bmp", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let valid_file_types: Vec<String> = cfg_c.1.strm_logo_valid_types.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::UNSUPPORTED_MEDIA_TYPE));
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_TYPE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileType": "image/bmp", "validFileType": &valid_file_types.join(",") });
        assert_eq!(*app_err.params.get("invalidFileType").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_stream_non_existent_id() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", format!("{}a", StreamModelsTest::title_min()))
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_stream_another_user() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", format!("{}a", StreamModelsTest::title_min()))
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut profile_vec = data_c.0;
        #[rustfmt::skip]
        profile_vec.push(ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User));
        let profile2 = ProfileOrmApp::create(&profile_vec).profile_vec.get(1).unwrap().clone();

        let stream2 = create_stream(1, profile2.user_id, "title2", "tag01,tag02", Utc::now());
        let mut stream_vec = data_c.2;
        let stream2_id = stream2.id;
        stream_vec.push(stream2);

        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream2_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_stream_another_user_by_admin() {
        let new_title = format!("{}b", StreamModelsTest::title_min());
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &new_title)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut profile_vec = data_c.0;
        let profile1 = profile_vec.get_mut(0).unwrap();
        profile1.role = UserRole::Admin;
        #[rustfmt::skip]
        profile_vec.push(ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User));
        let profile2 = ProfileOrmApp::create(&profile_vec).profile_vec.get(1).unwrap().clone();

        let stream2 = create_stream(1, profile2.user_id, "title2", "tag01,tag02", Utc::now());
        let mut stream_vec = data_c.2;
        stream_vec.push(stream2.clone());
        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res.user_id, stream2.user_id);
        assert_eq!(stream_dto_res.title, new_title);
        assert_eq!(stream_dto_res.descript, stream2.descript);
        assert_eq!(stream_dto_res.logo, stream2.logo);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_without_file() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let stream = data_c.2.get(0).unwrap().clone();
        let user_id = stream.user_id;
        let title_s = format!("{}_a", stream.title.clone());
        let descript_s = format!("{}_a", stream.descript.clone());
        let logo = stream.logo.clone();
        let starttime = stream.starttime.clone() + Duration::days(1);
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let source_s = format!("{}_a", stream.source.to_string());
        let tags: Vec<String> = stream.tags.clone().iter().map(|v| format!("{}_a", v)).collect();
        let tags_s = serde_json::to_string(&tags).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", title_s.clone())
            .with_text("descript", descript_s.clone())
            .with_text("starttime", starttime_s.clone())
            .with_text("source", source_s.clone())
            .with_text("tags", tags_s.clone())
            .build();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream.id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream.id);
        assert_eq!(stream_dto_res.user_id, user_id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, descript_s);
        assert_eq!(stream_dto_res.logo, logo);
        #[rustfmt::skip]
        assert_eq!(stream_dto_res.starttime.to_rfc3339_opts(SecondsFormat::Millis, true), starttime_s);
        assert_eq!(stream_dto_res.live, stream.live);
        assert_eq!(stream_dto_res.state, stream.state);
        assert_eq!(stream_dto_res.started, stream.started);
        assert_eq!(stream_dto_res.stopped, stream.stopped);
        assert_eq!(stream_dto_res.source, source_s);
        assert_eq!(stream_dto_res.tags, tags);
        assert_eq!(stream_dto_res.is_my_stream, stream.is_my_stream);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = stream_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_created_at = stream.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, old_created_at);
        let res_updated_at = stream_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_updated_at = stream.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, old_updated_at);
    }
    #[actix_web::test]
    async fn test_put_stream_a_with_old0_new1() {
        let name1_file = "test_put_stream_a_with_old0_new1.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let strm_logo_files_dir = cfg_c.1.strm_logo_files_dir.clone();
        let stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        let stream_orm = StreamOrmApp::create(&[stream.clone()]);

        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
                .insert_header(header_auth(&token))
                .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let stream_dto_res_img = stream_dto_res.logo.unwrap_or("".to_string());
        let img_name_full_path = stream_dto_res_img.replacen(ALIAS_LOGO_FILES_DIR, &strm_logo_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(stream_dto_res_img.len() > 0);
        assert!(stream_dto_res_img.starts_with(ALIAS_LOGO_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(stream_dto_res_img);
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
    async fn test_put_stream_b_with_old0_new1_convert() {
        let name1_file = "test_put_stream_b_with_old0_new1_convert.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 3).unwrap();

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut config_strm = config_strm::get_test_config();
        let file_ext = "jpeg".to_string();
        config_strm.strm_logo_ext = Some(file_ext.clone());
        config_strm.strm_logo_max_width = 18;
        config_strm.strm_logo_max_height = 18;
        let strm_logo_files_dir = config_strm.strm_logo_files_dir.clone();
        let cfg_c = (cfg_c.0, config_strm);

        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
                .insert_header(header_auth(&token))
                .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let stream_dto_res_img = stream_dto_res.logo.unwrap_or("".to_string());
        let img_name_full_path = stream_dto_res_img.replacen(ALIAS_LOGO_FILES_DIR, &strm_logo_files_dir, 1);
        let path = path::Path::new(&img_name_full_path);
        let receiver_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
        let is_exists_img_new = path.exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert_eq!(file_ext, receiver_ext);
        assert!(stream_dto_res_img.len() > 0);
        assert!(stream_dto_res_img.starts_with(ALIAS_LOGO_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(stream_dto_res_img);
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
    async fn test_put_stream_c_with_old1_new1() {
        let strm_logo_files_dir = config_strm::get_test_config().strm_logo_files_dir;

        let name0_file = "test_put_stream_c_with_old1_new1.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file);

        let name1_file = "test_put_stream_c_with_old1_new1_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let mut stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_alias);
        let stream_id = stream.id;
        let data_c = (data_c.0, data_c.1, vec![stream]);
        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
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
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let stream_dto_res_img = stream_dto_res.logo.unwrap_or("".to_string());
        let img_name_full_path = stream_dto_res_img.replacen(ALIAS_LOGO_FILES_DIR, &strm_logo_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(stream_dto_res_img.len() > 0);
        assert!(stream_dto_res_img.starts_with(ALIAS_LOGO_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(stream_dto_res_img);
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
    async fn test_put_stream_d_with_old1_new0() {
        let strm_logo_files_dir = config_strm::get_test_config().strm_logo_files_dir;

        let name0_file = "test_put_stream_d_with_old1_new0.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&path_name0_file, 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file);

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_text("title", "title1".to_string())
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let mut stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_alias.clone());
        let stream_id = stream.id;
        let data_c = (data_c.0, data_c.1, vec![stream]);

        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
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
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let stream_dto_res_img = stream_dto_res.logo.unwrap_or("".to_string());
        assert!(stream_dto_res_img.len() > 0);
        assert!(stream_dto_res_img.starts_with(ALIAS_LOGO_FILES_DIR));
        assert_eq!(&path_name0_alias, &stream_dto_res_img);
    }
    #[actix_web::test]
    async fn test_put_stream_e_with_old1_new_size0() {
        let strm_logo_files_dir = config_strm::get_test_config().strm_logo_files_dir;

        let name0_file = "test_put_stream_e_with_old1_new_size0.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file);

        let name1_file = "test_put_stream_e_with_old1_new_size0_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let mut stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_alias);
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
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
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(stream_dto_res.logo.is_none());
    }
    #[actix_web::test]
    async fn test_put_stream_f_with_old0_new_size0() {
        let name1_file = "test_put_stream_f_with_old0_new_size0.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
            let (header, body) = MultiPartFormDataBuilder::new()
                .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
                .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let stream = create_stream(0, profile1_id, "title1", "tag11,tag12", Utc::now());
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
                .insert_header(header_auth(&token))
                .insert_header(header).set_payload(body).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(stream_dto_res.logo.is_none());
    }

    // ** put_toggle_state **

    #[actix_web::test]
    async fn test_put_toggle_state_no_data() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/1"))
            .insert_header(header_auth(&token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Content type error"));
    }
    #[actix_web::test]
    async fn test_put_toggle_state_empty_json_object() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/1"))
            .insert_header(header_auth(&token))
            .set_json(serde_json::json!({}))
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
    async fn test_put_toggle_state_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        let stream_id_bad = format!("{}a", stream_id);
        let new_state = data_c.2.get(0).unwrap().state.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream_id_bad))
            .insert_header(header_auth(&token))
            .set_json(ToggleStreamStateDto{ state: new_state })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {} ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE, stream_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_put_toggle_state_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id2 = data_c.2.get(0).unwrap().id + 1;
        let new_state = data_c.2.get(0).unwrap().state.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream_id2))
            .insert_header(header_auth(&token))
            .set_json(ToggleStreamStateDto{ state: new_state })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_toggle_state_invalid_state() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        let new_state = data_c.2.get(0).unwrap().state.clone();
        let old_state = new_state;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream_id))
            .insert_header(header_auth(&token))
            .set_json(ToggleStreamStateDto{ state: new_state })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(&app_err.message, MSG_INVALID_STREAM_STATE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "oldState": &old_state, "newState": &new_state });
        assert_eq!(*app_err.params.get("invalidState").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_toggle_state_not_acceptable() {
        let buff = [
            // [Started, Paused] -> Preparing
            (StreamState::Started, StreamState::Preparing),
            (StreamState::Paused, StreamState::Preparing),
            // [Waiting, Stopped] -> Started
            (StreamState::Waiting, StreamState::Started),
            (StreamState::Stopped, StreamState::Started),
            // [Waiting, Stopped, Preparing] -> Paused
            (StreamState::Waiting, StreamState::Paused),
            (StreamState::Stopped, StreamState::Paused),
            (StreamState::Preparing, StreamState::Paused),
            // Waiting -> Stopped
            (StreamState::Waiting, StreamState::Stopped),
        ];
        for (old_state, new_state) in buff {
            let (cfg_c, mut data_c, token) = get_cfg_data();
            let stream = data_c.2.get_mut(0).unwrap();
            stream.state = old_state;
            let stream_id = stream.id;
            #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
            #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream_id))
                .insert_header(header_auth(&token))
                .set_json(ToggleStreamStateDto{ state: new_state })
                .to_request();
            let resp: dev::ServiceResponse = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
            #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
            let body = body::to_bytes(resp.into_body()).await.unwrap();
            let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
            assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
            assert_eq!(&app_err.message, MSG_INVALID_STREAM_STATE);
            #[rustfmt::skip]
            let json = serde_json::json!({ "oldState": &old_state, "newState": &new_state });
            assert_eq!(*app_err.params.get("invalidState").unwrap(), json);
        }
    }
    #[actix_web::test]
    async fn test_put_toggle_state_conflict() {
        let old_state = StreamState::Waiting;
        let new_state = StreamState::Preparing;
        let (cfg_c, mut data_c, token) = get_cfg_data();
        let user_id = data_c.0.get(0).unwrap().user_id;
        let stream1 = data_c.2.get_mut(0).unwrap();
        stream1.state = old_state;
        let stream1_id = stream1.id;
        let stream2_title = "title_2";
        let mut stream2 = create_stream(1, user_id, stream2_title, "tag01,tag02", Utc::now());
        stream2.state = StreamState::Preparing;
        stream2.live = true;
        let stream2_id = stream2.id;
        data_c.2.push(stream2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream1_id))
            .insert_header(header_auth(&token))
            .set_json(ToggleStreamStateDto{ state: new_state })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::CONFLICT));
        assert_eq!(&app_err.message, MSG_EXIST_IS_ACTIVE_STREAM);
        #[rustfmt::skip]
        let json = serde_json::json!({ "id": stream2_id, "title": &stream2_title });
        assert_eq!(*app_err.params.get("activeStream").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_toggle_state_ok() {
        let buff = [
            (StreamState::Preparing, StreamState::Started),
            (StreamState::Started, StreamState::Paused),
            (StreamState::Paused, StreamState::Started),
            (StreamState::Started, StreamState::Stopped),
            (StreamState::Paused, StreamState::Stopped),
        ];
        for (old_state, new_state) in buff {
            let (cfg_c, mut data_c, token) = get_cfg_data();
            let stream = data_c.2.get_mut(0).unwrap();
            stream.state = old_state;
            let stream_id = stream.id;
            let stream_user_id = stream.user_id;
            let new_live = vec![StreamState::Preparing, StreamState::Started, StreamState::Paused].contains(&new_state);
            #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(put_toggle_state).configure(configure_stream(cfg_c, data_c))).await;
            #[rustfmt::skip]
            let req = test::TestRequest::put().uri(&format!("/api/streams/toggle/{}", stream_id))
                .insert_header(header_auth(&token))
                .set_json(ToggleStreamStateDto{ state: new_state })
                .to_request();
            let resp: dev::ServiceResponse = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK); // 200
            #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
            let body = body::to_bytes(resp.into_body()).await.unwrap();
            let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
            assert_eq!(stream_dto_res.id, stream_id);
            assert_eq!(stream_dto_res.user_id, stream_user_id);
            assert_eq!(stream_dto_res.state, new_state);
            assert_eq!(stream_dto_res.live, new_live);
        }
    }
}

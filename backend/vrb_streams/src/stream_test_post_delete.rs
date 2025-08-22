#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{fs, path};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{Duration, SecondsFormat, Utc};
    use serde_json;
    use vrb_authent::{
        config_jwt,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, USER, USER1, USER1_ID},
    };
    use vrb_common::{
        api_error::{code_to_str, ApiError},
        consts,
    };
    use vrb_tools::{cdis::coding, err, png_files};

    use crate::{
        config_strm,
        stream_controller::{delete_stream, post_stream, tests as StrCtTest, MSG_INVALID_FIELD_TAG},
        stream_models::{self, StreamInfoDto, StreamModelsTest},
        stream_orm::tests::StreamOrmTest as Strm_Test,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";
    const MSG_MULTIPART_STREAM_INCOMPLETE: &str = "Multipart stream is incomplete";
    const MSG_CONTENT_TYPE_NOT_FOUND: &str = "Could not find Content-Type header";

    // ** post_stream **

    #[actix_web::test]
    async fn test_post_stream_no_form() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
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
    async fn test_post_stream_empty_form() {
        let form_builder = MultiPartFormDataBuilder::new();
        let (header, body) = form_builder.build();

        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
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
    async fn test_post_stream_title_empty() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", "")
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TITLE_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_post_stream_title_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TITLE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_title_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TITLE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_descript_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_descript_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_starttime_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let starttime_s = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("starttime", starttime_s)
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_MIN_VALID_STARTTIME]);
    }
    #[actix_web::test]
    async fn test_post_stream_source_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_SOURCE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_source_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_SOURCE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_tags_min_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_min();
        let tags_len = tags.len();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        let msg = if tags_len == 0 { stream_models::MSG_TAG_REQUIRED } else { stream_models::MSG_TAG_MIN_AMOUNT };
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[msg]);
    }
    #[actix_web::test]
    async fn test_post_stream_tags_max_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_max();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MAX_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_post_stream_tag_name_min() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_min());
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_tag_name_max() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_max());
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        StrCtTest::check_app_err(app_err_vec, &code_to_str(StatusCode::EXPECTATION_FAILED), &[stream_models::MSG_TAG_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_tag() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", "aaa")
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; {}", MSG_INVALID_FIELD_TAG, "expected value at line 1 column 1"));
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_file_size() {
        let name1_file = "test_post_stream_invalid_file_size.png";
        let path_name1_file = format!("./{}", &name1_file);
        let (size, _name) = png_files::save_file_png(&path_name1_file, 2).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let strm_logo_max_size = 160;
        let mut config_strm = config_strm::get_test_config();
        config_strm.strm_logo_max_size = strm_logo_max_size;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::PAYLOAD_TOO_LARGE));
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_SIZE);
        let json = serde_json::json!({ "actualFileSize": size, "maxFileSize": strm_logo_max_size });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_file_type() {
        let name1_file = "post_ellipse5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 1).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .with_file(path_name1_file.clone(), "logofile", "image/bmp", name1_file)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
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
    async fn test_post_stream_valid_data_without_logo_file() {
        let title_s = StreamModelsTest::title_enough();
        let descript_s = format!("{}a", StreamModelsTest::descript_min());
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();
        let starttime = Utc::now() + Duration::minutes(2);
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let source_s = format!("{}a", StreamModelsTest::source_min());

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("descript", &descript_s)
            .with_text("starttime", &starttime_s)
            .with_text("source", &source_s)
            .with_text("tags", &tags_s)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let stream1_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream1_id + 1);
        assert_eq!(stream_dto_res.user_id, user1_id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, descript_s);
        assert!(stream_dto_res.logo.is_none());
        #[rustfmt::skip]
        assert_eq!(stream_dto_res.starttime.to_rfc3339_opts(SecondsFormat::Millis, true), starttime_s);
        assert_eq!(stream_dto_res.source, source_s);
        assert_eq!(stream_dto_res.tags, tags);
        assert_eq!(stream_dto_res.is_my_stream, true);
    }
    #[actix_web::test]
    async fn test_post_stream_with_new_file() {
        let name1_file = "test_post_stream_with_new_file.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 1).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let stream1_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream1_id + 1);
        assert_eq!(stream_dto_res.user_id, user1_id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.tags, tags);

        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());
        let logo_name_full_path = stream_dto_res_logo.replacen(consts::ALIAS_LOGO_FILES_DIR, &strm_logo_files_dir, 1);
        let is_exists_logo_new = path::Path::new(&logo_name_full_path).exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(consts::ALIAS_LOGO_FILES_DIR));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string();
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string();
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string();
        assert_eq!(file_stem_part1, user1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_with_empty_file() {
        let name1_file = "post_circle_empty.png";
        let path_name1_file = format!("./{}", name1_file);
        png_files::save_empty_file(&path_name1_file).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let stream1_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res.id, stream1_id + 1);
        assert_eq!(stream_dto_res.user_id, user1_id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, "");
        assert_eq!(stream_dto_res.logo, None);
        assert_eq!(stream_dto_res.tags.len(), tags.len());
        assert_eq!(stream_dto_res.tags, tags);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_with_logo_convert_file_new() {
        let name1_file = "post_triangle_23x19.png";
        let path_name1_file = format!("./{}", &name1_file);
        png_files::save_file_png(&path_name1_file, 3).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let user1_id = data_u.0.get(0).unwrap().id;

        let mut config_strm = config_strm::get_test_config();
        let file_ext = "jpeg".to_string();
        config_strm.strm_logo_ext = Some(file_ext.clone());
        config_strm.strm_logo_max_width = 18;
        config_strm.strm_logo_max_height = 18;
        let strm_logo_files_dir = config_strm.strm_logo_files_dir.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams")
            .insert_header(StrCtTest::header_auth(&token1))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());
        let logo_name_full_path = stream_dto_res_logo.replacen(consts::ALIAS_LOGO_FILES_DIR, &strm_logo_files_dir, 1);
        let path = path::Path::new(&logo_name_full_path);
        let receiver_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
        let is_exists_logo_new = path.exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert_eq!(file_ext, receiver_ext);
        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(consts::ALIAS_LOGO_FILES_DIR));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string();
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string();
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string();
        assert_eq!(file_stem_part1, user1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }

    // ** delete_stream **

    #[actix_web::test]
    async fn test_delete_stream_invalid_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream_id_bad = format!("{}a", streams.get(0).unwrap().id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream_id_bad))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[actix_web::test]
    async fn test_delete_stream_non_existent_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream1_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream1_id + 1))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_stream_existent_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream1 = streams.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream1.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_stream1 = serde_json::json!(stream1).to_string();
        let stream1_dto_org: StreamInfoDto = serde_json::from_slice(json_stream1.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream1_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_stream_with_img() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "test_delete_stream_with_img.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", consts::ALIAS_LOGO_FILES_DIR, name0_file);

        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let mut streams = Strm_Test::streams(&[USER1]);
        let stream1 = streams.get_mut(0).unwrap();
        stream1.logo = Some(path_name0_alias);
        let stream2 = stream1.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_stream = serde_json::json!(stream2).to_string();
        let stream_dto_org: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_stream_with_img_not_alias() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "test_delete_stream_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_logo = format!("/not_alias{}/{}", consts::ALIAS_LOGO_FILES_DIR, name0_file);

        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let mut streams = Strm_Test::streams(&[USER1]);
        let stream1 = streams.get_mut(0).unwrap();
        stream1.logo = Some(path_name0_logo);
        let stream2 = stream1.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm::get_test_config()))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_stream = serde_json::json!(stream2).to_string();
        let stream_dto_org: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_org);
    }
}

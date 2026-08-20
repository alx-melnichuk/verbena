#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, App, body, dev,
        http::StatusCode,
        http::header::{CONTENT_TYPE, HeaderValue},
        test,
    };

    use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Timelike, Utc};
    use serde_json;
    use vrb_authent::{
        config_jwt,
        user_db::tests::{ADMIN, USER, USER1, USER1_ID, USER2, UserDbTest},
    };
    use vrb_common::{api_error::ApiError, err};
    use vrb_db::enm_stream_state::StreamState;

    use crate::{
        stream_controller::{
            MSG_FINISH_EXCEEDS_LIMIT, MSG_FINISH_LESS_START, PERIOD_MAX_NUMBER_DAYS, get_stream_and_tags, get_stream_and_tags_by_id,
            tests as StreamCtrlTest,
        },
        stream_db::tests::StreamDbTest,
        stream_models::{MSG_FINISHTIME_REQUIRED, MSG_STARTTIME_REQUIRED, PageStreamAndTagsDto, StreamAndTagsDto},
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    fn to_utc(value: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(value)
    }

    // ** get_stream_and_tags_by_id **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_id_invalid_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        let streams = StreamDbTest::streams(&[USER1]);
        let stream_id = streams.get(0).unwrap().id.clone();
        let stream_id_bad = format!("{}a", stream_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id_bad))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.status, StatusCode::RANGE_NOT_SATISFIABLE.as_u16());
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {} ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE, stream_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_id_valid_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        let streams = StreamDbTest::streams(&[USER1]);
        let stream = streams.get(0).unwrap().clone();
        let stream_dto: StreamAndTagsDto = stream.into();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_dto.id))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream_dto).to_string();
        let stream_dto_ser: StreamAndTagsDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_id_non_existent_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        let streams = StreamDbTest::streams(&[USER1]);
        let stream_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_id_another_user() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let streams = StreamDbTest::streams(&[0, 1]);
        let stream2 = streams.get(1).unwrap().clone();
        let stream2_dto: StreamAndTagsDto = stream2.into();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2_dto.id))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream2_dto).to_string();
        let stream_dto_ser: StreamAndTagsDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_id_another_user_by_admin() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[ADMIN, USER]);
        let streams = StreamDbTest::streams(&[USER1, USER2]);
        let stream2 = streams.get(1).unwrap().clone();
        let stream2_dto: StreamAndTagsDto = stream2.into();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags_by_id)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2_dto.id))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream2_dto).to_string();
        let stream_dto_ser: StreamAndTagsDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }

    // ** get_stream_and_tags **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1 and user2.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER2, USER2, USER2]);
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user1_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_page_limit_no_user_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1 and user2.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER2, USER2, USER2]);
        let count: u32 = streams.len() as u32;
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        let pages = (count as f32 / limit as f32).ceil() as u32;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_streams1b = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json_streams1b.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, pages);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id_page2() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1 and user2.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER2, USER2, USER1, USER1]);
        // Select streams with indices: 4,5.
        let streams1b = &streams.clone()[4..6];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user1_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_streams1b = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json_streams1b.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_another_user_id_role_user() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user2.
        let streams = StreamDbTest::streams(&[USER2, USER2]);
        let streams1b_dto: Vec<StreamAndTagsDto> = streams.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user2_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_another_user_id_role_admin() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[ADMIN, USER]);
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user2.
        let streams = StreamDbTest::streams(&[USER2, USER2]);
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone();
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user2_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (live) **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_live() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let live = true;
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1]);
        streams.get_mut(2).unwrap().state = StreamState::Preparing;
        streams.get_mut(3).unwrap().state = StreamState::Preparing;
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone()[2..4];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?live={}&page={}&limit={}", live, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list[0].live, live);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (filter="future") **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_future_no_starttime() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri("/api/streams?filter=future&page=1&limit=3")
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        StreamCtrlTest::check_app_err(app_err_vec, StatusCode::EXPECTATION_FAILED.as_u16(), &[MSG_STARTTIME_REQUIRED]);
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get("required").unwrap(), true);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_future() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        streams.get_mut(0).unwrap().starttime = yesterday;
        streams.get_mut(1).unwrap().starttime = now - sec;
        streams.get_mut(2).unwrap().starttime = now;
        streams.get_mut(3).unwrap().starttime = now + sec;
        streams.get_mut(4).unwrap().starttime = tomorrow;
        // Then return streams with a "starttime" date greater than or equal to "now".
        // Select streams with indices: 2,3,4.
        let streams1b = &streams.clone()[2..5];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=future&starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_future_sort_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        streams.get_mut(0).unwrap().starttime = yesterday;
        streams.get_mut(1).unwrap().starttime = now - sec;
        streams.get_mut(2).unwrap().starttime = now;
        streams.get_mut(3).unwrap().starttime = now + sec;
        streams.get_mut(4).unwrap().starttime = tomorrow;
        // Then return streams with a "starttime" date greater than or equal to "now".
        // Select streams with indices: 2,3,4.
        let streams1b = &streams.clone()[2..5];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().rev().map(|v| v.clone().into()).collect();
        let starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=future&starttime={}&sortDesc=true&page={}&limit={}", starttime, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (filter="past") **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_past_no_starttime() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri("/api/streams?filter=past&page=1&limit=3")
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        StreamCtrlTest::check_app_err(app_err_vec, StatusCode::EXPECTATION_FAILED.as_u16(), &[MSG_STARTTIME_REQUIRED]);
        #[rustfmt::skip]
        assert_eq!(*app_err.params.get("required").unwrap(), true);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_past() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        streams.get_mut(0).unwrap().starttime = yesterday;
        streams.get_mut(1).unwrap().starttime = now - sec;
        streams.get_mut(2).unwrap().starttime = now;
        streams.get_mut(3).unwrap().starttime = now + sec;
        streams.get_mut(4).unwrap().starttime = tomorrow;
        // Then return streams with a "startstarttime" date less than "now".
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=past&starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_past_sort_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        streams.get_mut(0).unwrap().starttime = yesterday;
        streams.get_mut(1).unwrap().starttime = now - sec;
        streams.get_mut(2).unwrap().starttime = now;
        streams.get_mut(3).unwrap().starttime = now + sec;
        streams.get_mut(4).unwrap().starttime = tomorrow;
        // Then return streams with a "startstarttime" date less than "now".
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().rev().map(|v| v.clone().into()).collect();
        let starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=past&starttime={}&sortDesc=true&page={}&limit={}", starttime, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (filter="period") **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_period_no_starttime() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri("/api/streams?filter=period&page=1&limit=3")
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err1 = app_err_vec.get(0).unwrap().clone();
        let app_err2 = app_err_vec.get(1).unwrap().clone();
        let msgs = [MSG_STARTTIME_REQUIRED, MSG_FINISHTIME_REQUIRED];
        #[rustfmt::skip]
        StreamCtrlTest::check_app_err(app_err_vec, StatusCode::EXPECTATION_FAILED.as_u16(), &msgs);
        #[rustfmt::skip]
        assert_eq!(*app_err1.params.get("required").unwrap(), true);
        #[rustfmt::skip]
        assert_eq!(*app_err2.params.get("required").unwrap(), true);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_period_no_finishtime() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=period&starttime={}&page=1&limit=3", starttime))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let app_err1 = app_err_vec.get(0).unwrap().clone();
        #[rustfmt::skip]
        StreamCtrlTest::check_app_err(app_err_vec, StatusCode::EXPECTATION_FAILED.as_u16(), &[MSG_FINISHTIME_REQUIRED]);
        #[rustfmt::skip]
        assert_eq!(*app_err1.params.get("required").unwrap(), true);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_period_startime_gr_finishtime() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let sec = Duration::seconds(1);
        let starttime = (now + sec).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finishtime = (now - sec).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=period&starttime={}&finishtime={}&page=1&limit=3", starttime, finishtime))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.status, StatusCode::NOT_ACCEPTABLE.as_u16());
        assert_eq!(app_err.message, MSG_FINISH_LESS_START);
        let json = serde_json::json!({ "streamPeriodStart": starttime, "streamPeriodFinish": finishtime });
        assert_eq!(*app_err.params.get("invalidPeriod").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_period_finish_too_big_start() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        let max_finish_s = to_utc(max_finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=period&starttime={}&finishtime={}&page=1&limit=3", start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.status, StatusCode::PAYLOAD_TOO_LARGE.as_u16());
        assert_eq!(app_err.message, MSG_FINISH_EXCEEDS_LIMIT);
        let json = serde_json::json!({ "actualPeriodFinish": finish_s
            , "maxPeriodFinish": max_finish_s, "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        assert_eq!(*app_err.params.get("periodTooLong").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_filter_period() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1, USER1]);
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        streams.get_mut(0).unwrap().starttime = yesterday;
        streams.get_mut(1).unwrap().starttime = now - sec;
        streams.get_mut(2).unwrap().starttime = now;
        streams.get_mut(3).unwrap().starttime = now + sec;
        streams.get_mut(4).unwrap().starttime = tomorrow;
        // Select streams with indices: 1,2,3.
        let streams1b = &streams.clone()[1..4];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().map(|v| v.clone().into()).collect();
        let starttime = (now - sec).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finishtime = (now + sec).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?filter=period&starttime={}&finishtime={}&page={}&limit={}", starttime, finishtime, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (user_id order starttime) **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id_order_starttime_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1]);
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        streams.get_mut(0).unwrap().starttime = now + two_days;
        streams.get_mut(1).unwrap().starttime = now + one_day;
        streams.get_mut(2).unwrap().starttime = now - one_day;
        streams.get_mut(3).unwrap().starttime = now - two_days;
        // Select streams with indices: 3,2,1,0.
        let streams1b_dto: Vec<StreamAndTagsDto> = streams.iter().rev().map(|v| v.clone().into()).collect();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user1_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id_order_starttime_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER1, USER1]);
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        streams.get_mut(0).unwrap().starttime = now - two_days;
        streams.get_mut(1).unwrap().starttime = now - one_day;
        streams.get_mut(2).unwrap().starttime = now + one_day;
        streams.get_mut(3).unwrap().starttime = now + two_days;
        // Select streams with indices: 3,2,1,0.
        let streams1b_dto: Vec<StreamAndTagsDto> = streams.iter().rev().map(|v| v.clone().into()).collect();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&sortDesc=true&page={}&limit={}", user1_id, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_and_tags (tag order starttime) **

    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id_tag_order_starttime_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        // Create streams for user1.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER2, USER2]);
        let now = Utc::now();
        streams.get_mut(0).unwrap().starttime = now + Duration::days(2);
        streams.get_mut(1).unwrap().starttime = now + Duration::days(1);
        streams.get_mut(2).unwrap().starttime = now - Duration::days(1);
        streams.get_mut(3).unwrap().starttime = now - Duration::days(2);
        let tag = "tag2";
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone()[2..4];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().rev().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?tag={}&page={}&limit={}", tag, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_stream_and_tags_by_user_id_tag_order_starttime_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserDbTest::users(&[USER]);
        // Create streams for user1, user2.
        let mut streams = StreamDbTest::streams(&[USER1, USER1, USER2, USER2]);
        let now = Utc::now();
        streams.get_mut(0).unwrap().starttime = now + Duration::days(2);
        streams.get_mut(1).unwrap().starttime = now + Duration::days(1);
        streams.get_mut(2).unwrap().starttime = now - Duration::days(2);
        streams.get_mut(3).unwrap().starttime = now - Duration::days(1);
        let tag = "tag2";
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone()[2..4];
        let streams1b_dto: Vec<StreamAndTagsDto> = streams1b.iter().rev().map(|v| v.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_and_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserDbTest::cfg_user_db(data_u))
                .configure(StreamDbTest::cfg_stream_db(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?tag={}&sortDesc=true&page={}&limit={}", tag, page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamAndTagsDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b_dto.len() as u32;
        let json = serde_json::json!(streams1b_dto).to_string();
        let streams1b_ser: Vec<StreamAndTagsDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
}

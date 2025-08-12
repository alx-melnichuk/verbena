#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Timelike, Utc};
    use serde_json;
    use vrb_common::api_error::{code_to_str, ApiError};
    use vrb_dbase::{
        config_jwt,
        user_auth_orm::tests::{UserAuthOrmTest as User_Test, ADMIN, USER, USER1, USER1_ID, USER2},
    };
    use vrb_tools::err;

    use crate::streams::{
        config_strm,
        stream_controller::{
            get_stream_by_id, get_stream_config, get_streams, get_streams_events, get_streams_period, tests as StrCtTest,
            MSG_FINISH_EXCEEDS_LIMIT, MSG_FINISH_LESS_START, MSG_GET_LIST_OTHER_USER_STREAMS, MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS,
            MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, PERIOD_MAX_NUMBER_DAYS,
        },
        stream_models::{self, StreamConfigDto, StreamEventDto, StreamEventPageDto, StreamInfoDto, StreamInfoPageDto},
        stream_orm::tests::StreamOrmTest as Strm_Test,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    fn to_utc(value: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(value)
    }

    // ** get_stream_by_id **

    #[actix_web::test]
    async fn test_get_stream_by_id_invalid_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream_id = streams.get(0).unwrap().id.clone();
        let stream_id_bad = format!("{}a", stream_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id_bad))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
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
    async fn test_get_stream_by_id_valid_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream_dto = streams.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_dto.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream_dto).to_string();
        let stream_dto_ser: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_non_existent_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let streams = Strm_Test::streams(&[USER1]);
        let stream_id = streams.get(0).unwrap().id.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_another_user() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let streams = Strm_Test::streams(&[0, 1]);
        let mut stream2 = streams.get(1).unwrap().clone();
        stream2.is_my_stream = false;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream2).to_string();
        let stream_dto_ser: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_another_user_by_admin() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN, USER]);
        let streams = Strm_Test::streams(&[USER1, USER2]);
        let mut stream2 = streams.get(1).unwrap().clone();
        stream2.is_my_stream = false;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream = serde_json::json!(stream2).to_string();
        let stream_dto_ser: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }

    // ** get_streams **

    #[actix_web::test]
    async fn test_get_streams_search_by_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1 and user2.
        let streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2, USER2]);
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user1_id, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_page_limit_without_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1 and user2.
        let streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2, USER2]);
        // Select streams with indices: 0,1.
        let streams1b = &streams.clone()[0..2];
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?page={}&limit={}", page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_streams1b = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json_streams1b.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_page2() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1 and user2.
        let streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2, USER1, USER1]);
        // Select streams with indices: 4,5.
        let streams1b = &streams.clone()[4..6];
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user1_id, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_streams1b = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json_streams1b.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_another_user_id_with_role_user() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user2.
        let streams = Strm_Test::streams(&[USER2, USER2]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page=1&limit=2", user2_id))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::FORBIDDEN));
        let text = format!("curr_user_id: {}, user_id: {}", user1_id, user2_id);
        #[rustfmt::skip]
        let message = format!("{}; {}; {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_another_user_id_with_role_admin() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN, USER]);
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user2.
        let mut streams = Strm_Test::streams(&[USER2, USER2]);
        streams.get_mut(0).unwrap().is_my_stream = false;
        streams.get_mut(1).unwrap().is_my_stream = false;
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", user2_id, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_live() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let live = true;
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1]);
        streams.get_mut(0).unwrap().live = !live;
        streams.get_mut(1).unwrap().live = !live;
        streams.get_mut(2).unwrap().live = live;
        streams.get_mut(3).unwrap().live = live;
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone()[2..4];
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?live={}&page={}&limit={}", live, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list[0].live, live);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_is_future() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1, USER1]);
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
        let future_starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?futureStarttime={}&page={}&limit={}", future_starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_is_not_future() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1, USER1]);
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
        let past_starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?pastStarttime={}&page={}&limit={}", past_starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_asc() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1]);
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        streams.get_mut(0).unwrap().starttime = now + two_days;
        streams.get_mut(1).unwrap().starttime = now + one_day;
        streams.get_mut(2).unwrap().starttime = now - one_day;
        streams.get_mut(3).unwrap().starttime = now - two_days;
        // Select streams with indices: 3,2,1,0.
        let streams1b: Vec<StreamInfoDto> = streams.clone().into_iter().rev().collect();
        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Asc.to_string();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
                order_column, order_dir, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_desc() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1]);
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        streams.get_mut(0).unwrap().starttime = now - two_days;
        streams.get_mut(1).unwrap().starttime = now - one_day;
        streams.get_mut(2).unwrap().starttime = now + one_day;
        streams.get_mut(3).unwrap().starttime = now + two_days;
        // Select streams with indices: 3,2,1,0.
        let streams1b: Vec<StreamInfoDto> = streams.clone().into_iter().rev().collect();
        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Desc.to_string();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
                order_column, order_dir, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_config **

    #[actix_web::test]
    async fn test_get_stream_config_data() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let config_strm = config_strm::get_test_config();
        #[rustfmt::skip]
        let stream_config_dto = StreamConfigDto::new(
            if config_strm.strm_logo_max_size > 0 { Some(config_strm.strm_logo_max_size) } else { None },
            config_strm.strm_logo_valid_types.clone(),
            config_strm.strm_logo_ext.clone(),
            if config_strm.strm_logo_max_width > 0 { Some(config_strm.strm_logo_max_width) } else { None },
            if config_strm.strm_logo_max_height > 0 { Some(config_strm.strm_logo_max_height) } else { None },
        );
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_config)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_config_strm(config_strm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/streams_config")
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let stream_config_dto_res: StreamConfigDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_config_dto_res, stream_config_dto);
    }

    // ** get_streams_events **

    #[actix_web::test]
    async fn test_get_streams_events_search_by_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let day = Duration::hours(23) + Duration::minutes(59) + Duration::seconds(59);
        streams.get_mut(0).unwrap().starttime = to_utc(today - Duration::seconds(1));
        streams.get_mut(1).unwrap().starttime = to_utc(today);
        streams.get_mut(2).unwrap().starttime = to_utc(today + day);
        streams.get_mut(3).unwrap().starttime = to_utc(today + Duration::hours(24));
        // Select streams with indices: 1,2.
        let streams1b = &streams.clone()[1..3];
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}",
                user1_id, starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_without_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let d23_59_59 = Duration::hours(23) + Duration::minutes(59) + Duration::seconds(59);
        streams.get_mut(0).unwrap().starttime = to_utc(today - Duration::seconds(1));
        streams.get_mut(1).unwrap().starttime = to_utc(today);
        streams.get_mut(2).unwrap().starttime = to_utc(today + d23_59_59);
        streams.get_mut(3).unwrap().starttime = to_utc(today + Duration::hours(24));
        streams.get_mut(4).unwrap().starttime = to_utc(today);
        // Select streams with indices: 1,2.
        let streams1b = &[streams[1].clone(), streams[4].clone()];

        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = streams1b.len() as u32;
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_page2() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2, USER1, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        streams.get_mut(0).unwrap().starttime = to_utc(today);
        streams.get_mut(1).unwrap().starttime = to_utc(today);
        streams.get_mut(2).unwrap().starttime = to_utc(today);
        streams.get_mut(3).unwrap().starttime = to_utc(today);
        streams.get_mut(4).unwrap().starttime = to_utc(today);
        streams.get_mut(5).unwrap().starttime = to_utc(today);
        // Select streams with indices: 4,5.
        let streams1b = &streams.clone()[4..6];
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_bad_starttime() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let today_decrem1 = to_utc(today - Duration::days(1));
        let today_increm1 = to_utc(today + Duration::days(2));
        streams.get_mut(0).unwrap().starttime = today_decrem1;
        streams.get_mut(1).unwrap().starttime = today_decrem1;
        streams.get_mut(2).unwrap().starttime = today_increm1;
        streams.get_mut(3).unwrap().starttime = today_increm1;
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, vec![]);
        assert_eq!(response.list.len(), 0);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 0);
        assert_eq!(response.page, 1);
        assert_eq!(response.pages, 0);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_another_user_id_with_role_user() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        streams.get_mut(0).unwrap().starttime = to_utc(today);
        streams.get_mut(1).unwrap().starttime = to_utc(today);
        streams.get_mut(2).unwrap().starttime = to_utc(today);
        streams.get_mut(3).unwrap().starttime = to_utc(today);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}", user2_id, starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::FORBIDDEN));
        let text = format!("curr_user_id: {}, user_id: {}", user1_id, user2_id);
        #[rustfmt::skip]
        let message = format!("{}; {}; {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_another_user_id_with_role_admin() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN, USER]);
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user1.
        let mut streams = Strm_Test::streams(&[USER1, USER1, USER2, USER2]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        streams.get_mut(0).unwrap().starttime = to_utc(today);
        streams.get_mut(1).unwrap().starttime = to_utc(today);
        streams.get_mut(2).unwrap().starttime = to_utc(today);
        streams.get_mut(3).unwrap().starttime = to_utc(today);
        // Select streams with indices: 2,3.
        let streams1b = &streams.clone()[2..4];
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}", user2_id, starttime, page, limit))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(streams1b).to_string();
        let streams1b_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, streams1b_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_streams_period **

    #[actix_web::test]
    async fn test_get_streams_period_by_finish_less_start() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start - Duration::seconds(1);
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", user1_id, start_s, finish_s))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, MSG_FINISH_LESS_START);
        let json = serde_json::json!({ "streamPeriodStart": start_s, "streamPeriodFinish": finish_s });
        assert_eq!(*app_err.params.get("invalidPeriod").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_finish_more_on_2_month() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        let max_finish_s = to_utc(max_finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(Strm_Test::streams(&[])))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", user1_id, start_s, finish_s))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::PAYLOAD_TOO_LARGE));
        assert_eq!(app_err.message, MSG_FINISH_EXCEEDS_LIMIT);
        let json = serde_json::json!({ "actualPeriodFinish": finish_s
            , "maxPeriodFinish": max_finish_s, "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        assert_eq!(*app_err.params.get("periodTooLong").unwrap(), json);
    }

    fn get_streams2(user_idx: usize) -> (Vec<StreamInfoDto>, String, String, Vec<DateTime<Utc>>) {
        let mut streams = Strm_Test::streams(&[user_idx, user_idx, user_idx, user_idx]);
        let dt = Local::now();
        let month1 = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let month2 = Local.with_ymd_and_hms(dt.year(), dt.month() + 1, 1, 0, 0, 0).unwrap();
        streams.get_mut(0).unwrap().starttime = to_utc(month1 - Duration::seconds(1));
        streams.get_mut(1).unwrap().starttime = to_utc(month1);
        streams.get_mut(2).unwrap().starttime = to_utc(month2 - Duration::seconds(1));
        streams.get_mut(3).unwrap().starttime = to_utc(month2);
        let period: Vec<DateTime<Utc>> = vec![to_utc(month1), to_utc(month2 - Duration::seconds(1))];
        let start = to_utc(month1).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish = to_utc(month2 - Duration::seconds(1)).to_rfc3339_opts(SecondsFormat::Millis, true);
        (streams, start, finish, period)
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let (streams, start, finish, period) = get_streams2(USER1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", user1_id, start, finish))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_period = serde_json::json!(period).to_string();
        let period_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_period.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), period_ser.len());
        assert_eq!(response, period_ser);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_without_user_id() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER]);
        let (streams, start, finish, period) = get_streams2(USER1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?start={}&finish={}", start, finish))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_period = serde_json::json!(period).to_string();
        let period_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_period.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), period_ser.len());
        assert_eq!(response, period_ser);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_another_user_id_with_role_user() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user2_id = data_u.0.get(1).unwrap().id;
        let (streams, start, finish, _period) = get_streams2(USER2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", user2_id, start, finish))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::FORBIDDEN));
        let text = format!("curr_user_id: {}, user_id: {}", user1_id, user2_id);
        #[rustfmt::skip]
        let message = format!("{}; {}; {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_another_user_id_with_role_admin_99() {
        let token1 = User_Test::get_token(USER1_ID);
        let data_u = User_Test::users(&[ADMIN, USER]);
        let user2_id = data_u.0.get(1).unwrap().id;
        let (streams, start, finish, period) = get_streams2(USER2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period)
                .configure(User_Test::cfg_config_jwt(config_jwt::get_test_config()))
                .configure(User_Test::cfg_user_auth_orm(data_u))
                .configure(Strm_Test::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", user2_id, start, finish))
            .insert_header(StrCtTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_period = serde_json::json!(period).to_string();
        let period_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_period.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), period_ser.len());
        assert_eq!(response, period_ser);
    }
}

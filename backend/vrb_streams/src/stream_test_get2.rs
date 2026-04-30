#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, App, body, dev,
        http::StatusCode,
        http::header::{CONTENT_TYPE, HeaderValue},
        test,
    };

    use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Utc};
    use serde_json;
    use vrb_authent::{
        config_jwt,
        user_orm::tests::{USER, USER1, USER1_ID, USER2, USER3, UserOrmTest},
    };
    use vrb_common::{
        api_error::{ApiError, code_to_str},
        err,
    };
    use vrb_dbase::enm_user_role::UserRole;

    use crate::{
        config_strm,
        stream_controller::{
            MSG_FINISH_EXCEEDS_LIMIT, MSG_FINISH_LESS_START, MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, PERIOD_MAX_NUMBER_DAYS,
            get_stream_config, get_stream_popural_tags, get_streams_calendar, tests as StreamCtrlTest,
        },
        stream_models::{PageStreamTagDto, StreamConfigDto, StreamTag, StreamTagDto},
        stream_orm::tests::{STREAM_TAG_ID, StreamOrmTest, TAG_NAME},
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn to_utc(value: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(value)
    }

    // ** get_stream_config **

    #[actix_web::test]
    async fn test_get_stream_config_data() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER]);
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
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_config_strm(config_strm))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/streams_config")
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let stream_config_dto_res: StreamConfigDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_config_dto_res, stream_config_dto);
    }

    // ** get_streams_calendar **

    #[actix_web::test]
    async fn test_get_streams_calendar_by_another_user_id() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        let user1_id = data_u.0.get(0).unwrap().id;
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let start_s = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(today + Duration::days(1)).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_calendar)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_calendar?userId={}&start={}&finish={}", user2_id, start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
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
    async fn test_get_streams_calendar_by_start_gr_finish() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let start_s = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(today - Duration::days(1)).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_calendar)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_calendar?start={}&finish={}", start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
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
    async fn test_get_streams_calendar_by_finish_too_big_start() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER1]);
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        let max_finish_s = to_utc(max_finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_calendar)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_calendar?start={}&finish={}", start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
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
    #[actix_web::test]
    async fn test_get_streams_calendar() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let mut streams = StreamOrmTest::streams(&[USER1, USER2, USER1, USER2, USER1, USER1]);
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let date1 = to_utc(start + Duration::days(1));
        let date2 = to_utc(start + Duration::days(2));
        let date3 = to_utc(start + Duration::days(3));
        let date4 = to_utc(start + Duration::days(4));
        let date5 = to_utc(start + Duration::days(5));
        streams.get_mut(0).unwrap().starttime = date5;
        streams.get_mut(1).unwrap().starttime = date4; // 2
        streams.get_mut(2).unwrap().starttime = date3;
        streams.get_mut(3).unwrap().starttime = date2; // 2
        streams.get_mut(4).unwrap().starttime = date1;
        streams.get_mut(5).unwrap().starttime = to_utc(start);
        let finish = start + Duration::days(5);
        let dates = [to_utc(start), date1, date3, date5];
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_calendar)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_calendar?start={}&finish={}", start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(dates).to_string();
        let dates_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response, dates_ser);
    }
    #[actix_web::test]
    async fn test_get_streams_calendar_for_admin() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let mut data_u = UserOrmTest::users(&[USER, USER]);
        let user1 = data_u.0.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user2_id = data_u.0.get(1).unwrap().id;
        // Create streams for user1.
        let mut streams = StreamOrmTest::streams(&[USER2, USER1, USER2, USER1, USER2, USER2]);
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let date1 = to_utc(start + Duration::days(1));
        let date2 = to_utc(start + Duration::days(2));
        let date3 = to_utc(start + Duration::days(3));
        let date4 = to_utc(start + Duration::days(4));
        let date5 = to_utc(start + Duration::days(5));
        streams.get_mut(0).unwrap().starttime = date5;
        streams.get_mut(1).unwrap().starttime = date4; // 1
        streams.get_mut(2).unwrap().starttime = date3;
        streams.get_mut(3).unwrap().starttime = date2; // 1
        streams.get_mut(4).unwrap().starttime = date1;
        streams.get_mut(5).unwrap().starttime = to_utc(start);
        let finish = start + Duration::days(5);
        let dates = [to_utc(start), date1, date3, date5];
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_calendar)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_calendar?userId={}&start={}&finish={}", user2_id, start_s, finish_s))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(dates).to_string();
        let dates_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response, dates_ser);
    }

    // ** get_stream_popural_tags **

    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page1_limit_sort_id_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID, TAG_NAME, 6),
            StreamTag::new(STREAM_TAG_ID + 1, &format!("{}{}", TAG_NAME, 1), 3),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page2_limit_sort_id_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 2, &format!("{}{}", TAG_NAME, 2), 2),
            StreamTag::new(STREAM_TAG_ID + 3, &format!("{}{}", TAG_NAME, 3), 1),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page1_limit_sort_id_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 3, &format!("{}{}", TAG_NAME, 3), 1),
            StreamTag::new(STREAM_TAG_ID + 2, &format!("{}{}", TAG_NAME, 2), 2),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortDesc=true&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page2_limit_sort_id_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 1, &format!("{}{}", TAG_NAME, 1), 3),
            StreamTag::new(STREAM_TAG_ID, TAG_NAME, 6),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortDesc=true&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page1_limit_sort_name_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID, TAG_NAME, 6),
            StreamTag::new(STREAM_TAG_ID + 1, &format!("{}{}", TAG_NAME, 1), 3),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortColumn=name&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page2_limit_sort_name_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 1, &format!("{}{}", TAG_NAME, 1), 3),
            StreamTag::new(STREAM_TAG_ID, TAG_NAME, 6),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortColumn=name&sortDesc=true&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page1_limit_sort_countlinks_asc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 3, &format!("{}{}", TAG_NAME, 3), 1),
            StreamTag::new(STREAM_TAG_ID + 2, &format!("{}{}", TAG_NAME, 2), 2),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortColumn=countlinks&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
    #[actix_web::test]
    async fn test_get_stream_popural_tags_by_page2_limit_sort_countlinks_desc() {
        let token1 = config_jwt::tests::get_token(USER1_ID);
        let data_u = UserOrmTest::users(&[USER, USER]);
        // Create streams for user1.
        let streams = StreamOrmTest::streams(&[USER1, USER1, USER1, USER2, USER2, USER3]);
        let strm_tags = vec![
            StreamTag::new(STREAM_TAG_ID + 2, &format!("{}{}", TAG_NAME, 2), 2),
            StreamTag::new(STREAM_TAG_ID + 3, &format!("{}{}", TAG_NAME, 3), 1),
        ];
        let strm_tags_dto: Vec<StreamTagDto> = strm_tags.iter().map(|t| t.clone().into()).collect();
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_popural_tags)
                .configure(config_jwt::tests::cfg_config_jwt(config_jwt::tests::get_config()))
                .configure(UserOrmTest::cfg_user_orm(data_u))
                .configure(StreamOrmTest::cfg_stream_orm(streams))
        ).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_popural_tags?sortColumn=countlinks&sortDesc=true&page={}&limit={}", page, limit))
            .insert_header(StreamCtrlTest::header_auth(&token1)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: PageStreamTagDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_strm_tags = serde_json::json!(strm_tags_dto).to_string();
        let strm_tags_ser: Vec<StreamTagDto> = serde_json::from_slice(json_strm_tags.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, strm_tags_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.page, page);
    }
}

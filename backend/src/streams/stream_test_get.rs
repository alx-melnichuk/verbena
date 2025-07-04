#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::{
            header::{HeaderValue, CONTENT_TYPE},
            StatusCode,
        },
        test, App,
    };
    use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Timelike, Utc};
    use serde_json;

    use crate::errors::AppError;
    use crate::profiles::profile_orm::tests::ProfileOrmApp;
    use crate::settings::err;
    use crate::streams::{
        stream_controller::{
            get_stream_by_id, get_stream_config, get_streams, get_streams_events, get_streams_period,
            tests::{
                configure_stream, create_stream, get_cfg_data, header_auth, MSG_CASTING_TO_TYPE, MSG_FAILED_DESER,
            },
            MSG_FINISH_EXCEEDS_LIMIT, MSG_FINISH_LESS_START, MSG_GET_LIST_OTHER_USER_STREAMS,
            MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, PERIOD_MAX_NUMBER_DAYS,
        },
        stream_models::{self, StreamConfigDto, StreamEventDto, StreamEventPageDto, StreamInfoDto, StreamInfoPageDto},
        stream_orm::tests::StreamOrmApp,
    };
    use crate::users::user_models::UserRole;

    // ** get_stream_by_id **

    #[actix_web::test]
    async fn test_get_stream_by_id_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        let stream_id_bad = format!("{}a", stream_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_RANGE_NOT_SATISFIABLE);
        #[rustfmt::skip]
        let msg = format!("{}; `{}` - {} ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE, stream_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_valid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_dto = data_c.2.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_dto.id))
            .insert_header(header_auth(&token)).to_request();
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
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_another_user() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile_vec = ProfileOrmApp::create(&vec![
            data_c.0.get(0).unwrap().clone(),
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let stream_vec = StreamOrmApp::create(&[
            data_c.2.get(0).unwrap().clone(),
            create_stream(2, profile2_id, "title_2", "tag0,tag2", Utc::now()),
        ])
        .stream_info_vec;
        let stream2 = stream_vec.get(1).unwrap().clone();

        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let mut stream2b = stream2.clone();
        stream2b.is_my_stream = false;
        let json_stream = serde_json::json!(stream2b).to_string();
        let stream_dto_ser: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_stream_by_id_another_user_by_admin() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let stream_vec = StreamOrmApp::create(&[
            data_c.2.get(0).unwrap().clone(),
            create_stream(2, profile2_id, "title_2", "tag0,tag2", Utc::now()),
        ])
        .stream_info_vec;
        let stream2 = stream_vec.get(1).unwrap().clone();

        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_by_id).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let mut stream2b = stream2.clone();
        stream2b.is_my_stream = false;
        let json_stream = serde_json::json!(stream2b).to_string();
        let stream_dto_ser: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_ser);
    }

    // ** get_streams **

    #[actix_web::test]
    async fn test_get_streams_search_by_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let stream_orm = StreamOrmApp::create(&[
            create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now()),
            create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now()),
            create_stream(2, profile1_id + 1, "demo21", "tag21,tag22", Utc::now()),
            create_stream(3, profile1_id + 1, "demo22", "tag24,tag25", Utc::now()),
            create_stream(4, profile1_id + 1, "demo32", "tag36,tag37", Utc::now()),
        ]);
        let stream_vec = stream_orm.stream_info_vec.clone();
        let stream1b_vec = &stream_vec[0..2];

        let limit = 2;
        let page = 1;
        let data_c = (data_c.0, data_c.1, stream_vec.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", profile1_id, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_page_limit_without_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let stream_orm = StreamOrmApp::create(&[
            create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now()),
            create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now()),
            create_stream(2, profile1_id + 1, "demo21", "tag21,tag22", Utc::now()),
            create_stream(3, profile1_id + 1, "demo22", "tag24,tag25", Utc::now()),
            create_stream(4, profile1_id + 1, "demo32", "tag36,tag37", Utc::now()),
        ]);
        let stream_vec = stream_orm.stream_info_vec.clone();
        let stream1b_vec = &stream_vec[0..2];
        let limit = 2;
        let page = 1;
        let data_c = (data_c.0, data_c.1, stream_vec.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?page={}&limit={}", page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_page2() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let stream_orm = StreamOrmApp::create(&[
            create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now()),
            create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now()),
            create_stream(2, profile1_id + 1, "demo21", "tag21,tag22", Utc::now()),
            create_stream(3, profile1_id + 1, "demo22", "tag24,tag25", Utc::now()),
            create_stream(4, profile1_id, "demo31", "tag31,tag32", Utc::now()),
            create_stream(5, profile1_id, "demo32", "tag34,tag35", Utc::now()),
        ]);
        let stream_vec = stream_orm.stream_info_vec.clone();
        let stream1b_vec = &stream_vec[4..6];
        let limit = 2;
        let page = 2;
        let data_c = (data_c.0, data_c.1, stream_vec.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", profile1_id, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_another_user_id_with_role_user() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let stream_orm = StreamOrmApp::create(&[
            create_stream(0, profile2_id, "demo11", "tag11,tag12", Utc::now()),
            create_stream(1, profile2_id, "demo12", "tag14,tag15", Utc::now()),
        ]);
        let stream_vec = stream_orm.stream_info_vec.clone();
        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page=1&limit=2", profile2_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        let text = format!("curr_user_id: {}, user_id: {}", profile1_id, profile2_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_another_user_id_with_role_admin() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let stream_orm = StreamOrmApp::create(&[
            create_stream(0, profile2_id, "demo11", "tag11,tag12", Utc::now()),
            create_stream(1, profile2_id, "demo12", "tag14,tag15", Utc::now()),
        ]);
        let mut stream1 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        stream1.is_my_stream = false;
        let mut stream2 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        stream2.is_my_stream = false;
        let stream_vec = vec![stream1, stream2];

        let data_c = (profile_vec, data_c.1, stream_orm.stream_info_vec);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?userId={}&page={}&limit={}", profile2_id, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_live() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let live = true;
        let mut stream1 = create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now());
        stream1.live = !live;
        let mut stream2 = create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now());
        stream2.live = !live;
        let mut stream3 = create_stream(2, profile1_id, "demo21", "tag21,tag22", Utc::now());
        stream3.live = live;
        let mut stream4 = create_stream(3, profile1_id, "demo22", "tag24,tag25", Utc::now());
        stream4.live = live;

        let stream_orm = StreamOrmApp::create(&[stream1, stream2, stream3, stream4]);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = &(stream_orm_vec.clone())[2..4];

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?live={}&page={}&limit={}", live, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list[0].live, live);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_is_future() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag10,tag11", yesterday));
        streams.push(create_stream(1, profile1_id, "demo12", "tag10,tag12", now - sec));
        streams.push(create_stream(2, profile1_id, "demo13", "tag10,tag13", now));
        streams.push(create_stream(3, profile1_id, "demo14", "tag10,tag14", now + sec));
        streams.push(create_stream(4, profile1_id, "demo15", "tag10,tag15", tomorrow));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        // Then return streams with a "starttime" date greater than or equal to "now".
        let stream_vec = &(stream_orm_vec.clone())[2..5];
        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let future_starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?futureStarttime={}&page={}&limit={}", future_starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_is_not_future() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let now = Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let sec = Duration::seconds(1);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag10,tag11", yesterday));
        streams.push(create_stream(1, profile1_id, "demo12", "tag10,tag12", now - sec));
        streams.push(create_stream(2, profile1_id, "demo13", "tag10,tag13", now));
        streams.push(create_stream(3, profile1_id, "demo14", "tag10,tag14", now + sec));
        streams.push(create_stream(4, profile1_id, "demo15", "tag10,tag15", tomorrow));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        // Then return streams with a "startstarttime" date less than "now".
        let stream_vec = &(stream_orm_vec.clone())[0..2];

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let past_starttime = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 3;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?pastStarttime={}&page={}&limit={}", past_starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_asc() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", now + two_days));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", now + one_day));
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", now - one_day));
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", now - two_days));
        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(3).unwrap().clone(),
            stream_orm_vec.get(2).unwrap().clone(),
            stream_orm_vec.get(1).unwrap().clone(),
            stream_orm_vec.get(0).unwrap().clone(),
        ];

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Asc.to_string();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
                order_column, order_dir, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_desc() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let now = Utc::now();
        let one_day = Duration::days(1);
        let two_days = Duration::days(2);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", now - two_days));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", now - one_day));
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", now + one_day));
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", now + two_days));
        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(3).unwrap().clone(),
            stream_orm_vec.get(2).unwrap().clone(),
            stream_orm_vec.get(1).unwrap().clone(),
            stream_orm_vec.get(0).unwrap().clone(),
        ];

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Desc.to_string();
        let limit = 4;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
                order_column, order_dir, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamInfoDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_stream_config **
    #[actix_web::test]
    async fn test_get_stream_config_data() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let cfg_strm = cfg_c.1.clone();
        #[rustfmt::skip]
        let stream_config_dto = StreamConfigDto::new(
            if cfg_strm.strm_logo_max_size > 0 { Some(cfg_strm.strm_logo_max_size) } else { None },
            cfg_strm.strm_logo_valid_types.clone(),
            cfg_strm.strm_logo_ext.clone(),
            if cfg_strm.strm_logo_max_width > 0 { Some(cfg_strm.strm_logo_max_width) } else { None },
            if cfg_strm.strm_logo_max_height > 0 { Some(cfg_strm.strm_logo_max_height) } else { None },
        );
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_stream_config).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/streams_config")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let stream_config_dto_res: StreamConfigDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_config_dto_res, stream_config_dto);
    }

    // ** get_streams_events **

    fn to_utc(value: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(value)
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let day = Duration::hours(23) + Duration::minutes(59) + Duration::seconds(59);

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        #[rustfmt::skip]
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", to_utc(today - Duration::seconds(1))));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", to_utc(today)));
        #[rustfmt::skip]
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", to_utc(today + day)));
        #[rustfmt::skip]
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", to_utc(today + Duration::hours(24))));
        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(1).unwrap().clone(),
            stream_orm_vec.get(2).unwrap().clone(),
        ];

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}",
                profile1_id, starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_without_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let day = Duration::hours(23) + Duration::minutes(59) + Duration::seconds(59);

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        #[rustfmt::skip]
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", to_utc(today - Duration::seconds(1))));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", to_utc(today)));
        #[rustfmt::skip]
        streams.push(create_stream(2, profile2_id, "demo21", "tag21,tag22", to_utc(today + day)));
        #[rustfmt::skip]
        streams.push(create_stream(3, profile2_id, "demo22", "tag24,tag25", to_utc(today + Duration::hours(24))));
        streams.push(create_stream(4, profile1_id, "demo31", "tag31,tag32", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(1).unwrap().clone(),
            stream_orm_vec.get(4).unwrap().clone(),
        ];

        let data_c = (profile_vec, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let count = stream_vec.len() as u32;
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, count);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_page2() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", to_utc(today)));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", to_utc(today)));
        streams.push(create_stream(2, profile2_id, "demo21", "tag21,tag22", to_utc(today)));
        streams.push(create_stream(3, profile2_id, "demo22", "tag24,tag25", to_utc(today)));
        streams.push(create_stream(4, profile1_id, "demo31", "tag31,tag32", to_utc(today)));
        streams.push(create_stream(5, profile1_id, "demo32", "tag34,tag35", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(4).unwrap().clone(),
            stream_orm_vec.get(5).unwrap().clone(),
        ];

        let data_c = (profile_vec, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 2;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_bad_starttime() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let today_decrem1 = to_utc(today - Duration::days(1));
        let today_increm1 = to_utc(today + Duration::days(2));

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", today_decrem1));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", today_decrem1));
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", today_increm1));
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", today_increm1));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();

        let data_c = (data_c.0, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?starttime={}&page={}&limit={}", starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
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
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", to_utc(today)));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", to_utc(today)));
        streams.push(create_stream(2, profile2_id, "demo21", "tag21,tag22", to_utc(today)));
        streams.push(create_stream(3, profile2_id, "demo22", "tag24,tag25", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();

        let data_c = (profile_vec, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}", profile2_id, starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        let text = format!("curr_user_id: {}, user_id: {}", profile1_id, profile2_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_events_search_by_another_user_id_with_role_admin() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();

        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", to_utc(today)));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", to_utc(today)));
        streams.push(create_stream(2, profile2_id, "demo21", "tag21,tag22", to_utc(today)));
        streams.push(create_stream(3, profile2_id, "demo22", "tag24,tag25", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&streams);
        let stream_orm_vec = stream_orm.stream_info_vec.clone();
        let stream_vec = vec![
            stream_orm_vec.get(2).unwrap().clone(),
            stream_orm_vec.get(3).unwrap().clone(),
        ];

        let data_c = (profile_vec, data_c.1, stream_orm_vec);
        let starttime = to_utc(today).to_rfc3339_opts(SecondsFormat::Millis, true);
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_events).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_events?userId={}&starttime={}&page={}&limit={}", profile2_id, starttime, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(stream_vec).to_string();
        let stream_vec_ser: Vec<StreamEventDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_streams_period **

    #[actix_web::test]
    async fn test_get_streams_period_by_finish_less_start() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start - Duration::seconds(1);
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile1_id, start_s, finish_s))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_FINISH_LESS_START);
        let json = serde_json::json!({ "streamPeriodStart": start_s, "streamPeriodFinish": finish_s });
        assert_eq!(*app_err.params.get("invalidPeriod").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_finish_more_on_2_month() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start_s = to_utc(start).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        let max_finish_s = to_utc(max_finish).to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile1_id, start_s, finish_s))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONTENT_TOO_LARGE);
        assert_eq!(app_err.message, MSG_FINISH_EXCEEDS_LIMIT);
        let json = serde_json::json!({ "actualPeriodFinish": finish_s
            , "maxPeriodFinish": max_finish_s, "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        assert_eq!(*app_err.params.get("periodTooLong").unwrap(), json);
    }

    fn get_streams2(user_id: i32) -> (Vec<StreamInfoDto>, String, String, Vec<DateTime<Utc>>) {
        let dt = Local::now();
        let month1 = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let month2 = Local.with_ymd_and_hms(dt.year(), dt.month() + 1, 1, 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        let d1 = month1 - Duration::seconds(1);
        stream_vec.push(create_stream(0, user_id, "demo11", "tag11,tag12", to_utc(d1)));
        let d2 = month1;
        stream_vec.push(create_stream(1, user_id, "demo12", "tag12,tag13", to_utc(d2)));
        let d3 = month2 - Duration::seconds(1);
        stream_vec.push(create_stream(2, user_id, "demo13", "tag13,tag14", to_utc(d3)));
        let d4 = month2;
        stream_vec.push(create_stream(3, user_id, "demo14", "tag14,tag15", to_utc(d4)));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream_info2 = stream_orm.stream_info_vec.get(2).unwrap().clone();
        let result_vec: Vec<DateTime<Utc>> = vec![stream_info1.starttime, stream_info2.starttime];
        let start = to_utc(d2).to_rfc3339_opts(SecondsFormat::Millis, true);
        let finish = to_utc(d3).to_rfc3339_opts(SecondsFormat::Millis, true);

        (stream_vec, start, finish, result_vec)
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let (stream_vec, start, finish, res_vec) = get_streams2(profile1_id);
        let data_c = (data_c.0, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile1_id, start, finish))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_without_user_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let (stream_vec, start, finish, res_vec) = get_streams2(profile1_id);
        let data_c = (data_c.0, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?start={}&finish={}", start, finish))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_another_user_id_with_role_user() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let (stream_vec, start, finish, _res_vec) = get_streams2(profile2_id);
        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile2_id, start, finish))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN); // 403

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        let text = format!("curr_user_id: {}, user_id: {}", profile1_id, profile2_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, &text);
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_get_streams_period_by_another_user_id_with_role_admin_99() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;
        let profile_vec = ProfileOrmApp::create(&vec![
            profile1,
            ProfileOrmApp::new_profile(2, "Liam_Smith", "Liam_Smith@gmail.com", UserRole::User),
        ])
        .profile_vec;
        let profile2_id = profile_vec.get(1).unwrap().user_id;

        let (stream_vec, start, finish, res_vec) = get_streams2(profile2_id);
        let data_c = (profile_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile2_id, start, finish))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
}

use std::{ops::Deref, time::Instant};

use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Duration, Utc};

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{config_strm, stream_models, stream_orm::StreamOrm};
use crate::users::user_models::UserRole;
use crate::utils::parser::{parse_i32, CD_PARSE_INT_ERROR};

pub const PERIOD_MAX_NUMBER_DAYS: u16 = 65;

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/streams/{id}
    cfg.service(get_stream_by_id)
        // GET api/streams
        .service(get_streams)
        // GET api/streams_events
        .service(get_streams_events)
        // GET api/streams_period
        .service(get_streams_period);
}

fn err_parse_int(err: String) -> AppError {
    log::error!("{}: id: {}", CD_PARSE_INT_ERROR, err);
    AppError::new(CD_PARSE_INT_ERROR, &format!("id: {}", err)).set_status(400)
}
fn err_database(err: String) -> AppError {
    log::error!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_no_access_to_streams() -> AppError {
    log::error!("{}: {}", err::CD_NO_ACCESS_TO_STREAMS, err::MSG_NO_ACCESS_TO_STREAMS);
    AppError::new(err::CD_NO_ACCESS_TO_STREAMS, err::MSG_NO_ACCESS_TO_STREAMS).set_status(400)
}
fn err_finish_less_start() -> AppError {
    log::error!("{}: {}", err::CD_FINISH_LESS_START, err::MSG_FINISH_LESS_START);
    AppError::new(err::CD_FINISH_LESS_START, err::MSG_FINISH_LESS_START).set_status(400)
}
fn err_bad_finish_period(max_days_period: u16) -> AppError {
    let msg = format!("{} ({}).", err::MSG_FINISH_GREATER_MAX, max_days_period);
    log::error!("{}: {}", err::CD_FINISH_GREATER_MAX, msg);
    AppError::new(err::CD_FINISH_GREATER_MAX, &msg).set_status(400)
}

// GET api/streams/{id}
#[rustfmt::skip]
#[get("/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_by_id(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let now: Instant = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    let res_data = web::block(move || {
        // Find 'stream' by id.
        let res_data =
            stream_orm.find_stream_by_id(id, curr_user_id).map_err(|e| err_database(e.to_string()));
        res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let opt_data = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let opt_stream_tag_dto = if let Some((stream, stream_tags)) = opt_data {
        let streams: Vec<stream_models::Stream> = vec![stream];
        // Merge a "stream" and a corresponding list of "tags".
        let list = stream_models::StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, curr_user_id);
        list.into_iter().nth(0)
    } else {
        None
    };

    if config_strm.strm_show_lead_time {
        log::info!("get_stream_by_id() lead time: {:.2?}", now.elapsed());
    }
    if let Some(stream_tag_dto) = opt_stream_tag_dto {
        Ok(HttpResponse::Ok().json(stream_tag_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/* Name:
* @route streams
* @example streams?groupBy=date&userId=385e0469-7143-4915-88d0-f23f5b27ed28/9/2022&orderColumn=title&orderDirection=desc&live=true
* @type get
* @query pagination (optional):
* - userId (only for groupBy "date")
* - key (keyword by tag or date, the date should be YYYY-MM-DD)
* - live (false, true)
* - starttime (none, past, future)
* - groupBy (none / tag / date, none by default)
* - page (number, 1 by default)
* - limit (number, 10 by default)
* - orderColumn (starttime / title, starttime by default)
* - orderDirection (asc / desc, asc by default)
* @access public
*/
// GET api/streams
#[rustfmt::skip]
#[get("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<stream_models::SearchStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get search parameters.
    let search_stream_info_dto: stream_models::SearchStreamInfoDto = query_params.into_inner();

    let page: u32 = search_stream_info_dto.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
    let limit: u32 = search_stream_info_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
    let mut search_stream = stream_models::SearchStream::convert(search_stream_info_dto);

    if search_stream.user_id.is_none() {
        search_stream.user_id = Some(curr_user_id);
    } else if let Some(user_id) = search_stream.user_id {
        if user_id != curr_user_id && curr_user.role != UserRole::Admin {
            return Err(err_no_access_to_streams());
        }
    }

    let res_data = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_data =
            stream_orm.find_streams(search_stream).map_err(|e| err_database(e.to_string()));
        res_data
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;
        
    let (count, streams, stream_tags) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    // Merge a "stream" and a corresponding list of "tags".
    let list = stream_models::StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, curr_user_id);

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = stream_models::StreamInfoPageDto { list, limit, count, page, pages };

    if config_strm.strm_show_lead_time {
        log::info!("get_streams() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().json(result)) // 200
}

// 'starttime' only format Utc ("%Y-%m-%dT%H:%M:%S.%3fZ").
// GET api/streams_events
#[rustfmt::skip]
#[get("/streams_events", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_events(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<stream_models::SearchStreamEventDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get search parameters.
    let search_stream_event_dto: stream_models::SearchStreamEventDto = query_params.into_inner();

    let page: u32 = search_stream_event_dto.page.unwrap_or(stream_models::SEARCH_STREAM_EVENT_PAGE);
    let limit: u32 = search_stream_event_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_EVENT_LIMIT);
    let search_stream_event = stream_models::SearchStreamEvent::convert(search_stream_event_dto, curr_user_id);
        
    if search_stream_event.user_id != curr_user_id && curr_user.role != UserRole::Admin {
        return Err(err_no_access_to_streams());
    }

    let res_data = web::block(move || {
        // Find for an entity (stream event) by SearchStreamEvent.
        let res_data =
            stream_orm.find_stream_events(search_stream_event).map_err(|e| err_database(e.to_string()));
        res_data
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let (count, streams) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let list = streams.into_iter().map(|v| stream_models::StreamEventDto::convert(v)).collect();

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = stream_models::StreamEventPageDto { list, limit, count, page, pages };

    if config_strm.strm_show_lead_time {
        log::info!("get_streams_events() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().json(result)) // 200
}

// GET api/streams_period
#[rustfmt::skip]
#[get("/streams_period", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_period(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<stream_models::SearchStreamPeriodDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get search parameters.
    let search_stream_period_dto: stream_models::SearchStreamPeriodDto = query_params.into_inner();

    let search_stream_period = stream_models::SearchStreamPeriod::convert(search_stream_period_dto, curr_user_id);
        
    if search_stream_period.user_id != curr_user_id && curr_user.role != UserRole::Admin {
        return Err(err_no_access_to_streams());
    }
    if search_stream_period.finish < search_stream_period.start {
        return Err(err_finish_less_start());
    }
    let finish = search_stream_period.start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
    if finish <= search_stream_period.finish {
        return Err(err_bad_finish_period(PERIOD_MAX_NUMBER_DAYS));
    }

    let res_data = web::block(move || {
        // Find for an entity (stream period) by SearchStreamEvent.
        let res_data =
            stream_orm.find_streams_period(search_stream_period).map_err(|e| err_database(e.to_string()));
        res_data
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let list: Vec<DateTime<Utc>> = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    if config_strm.strm_show_lead_time {
        log::info!("get_streams_period() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().json(list)) // 200

}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};

    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        config_strm,
        stream_models::{Stream, StreamEventDto, StreamEventPageDto, StreamInfoDto, StreamInfoPageDto},
        stream_orm::tests::STREAM_ID,
    };
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };
    use crate::utils::parser::{CD_PARSE_INT_ERROR, MSG_PARSE_INT_ERROR};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }
    fn create_stream(idx: i32, user_id: i32, title: &str, tags: &str, starttime: DateTime<Utc>) -> StreamInfoDto {
        let tags1: Vec<String> = tags.split(',').map(|val| val.to_string()).collect();
        let stream = Stream::new(STREAM_ID + idx, user_id, title, starttime);
        StreamInfoDto::convert(stream, user_id, &tags1)
    }

    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }

    async fn call_service1(
        cfg_c: (config_jwt::ConfigJwt, config_strm::ConfigStrm),
        data_c: (Vec<User>, Vec<Session>, Vec<StreamInfoDto>),
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(cfg_c.0);
        let data_config_strm = web::Data::new(cfg_c.1);
        let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
        let data_stream_orm = web::Data::new(StreamOrmApp::create(&data_c.2));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_strm))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_stream_orm))
                .service(factory),
        )
        .await;

        test::call_service(&app, request.to_request()).await
    }

    // ** get_stream_by_id **

    #[test]
    async fn test_get_stream_by_id_invalid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let stream_id_bad = format!("{}a", stream1b_dto.id);

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream_id_bad));
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_stream_by_id, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        #[rustfmt::skip]
        let msg = format!("id: {} `{}` - {}", MSG_PARSE_INT_ERROR, stream_id_bad, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[test]
    async fn test_get_stream_by_id_valid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream1b_dto.id));
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_stream_by_id, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_dto = serde_json::json!(stream1b_dto).to_string();
        let stream1b_dto_ser: StreamInfoDto =
            serde_json::from_slice(json_stream1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res, stream1b_dto_ser);
    }
    #[test]
    async fn test_get_stream_by_id_non_existent_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_id = stream_orm.stream_info_vec.get(0).unwrap().clone().id;

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream_id + 1));
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_stream_by_id, request).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[test]
    async fn test_get_stream_by_id_another_user() {
        let user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user2.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_id = stream_orm.stream_info_vec.get(0).unwrap().clone().id;

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream_id));
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_stream_by_id, request).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    // ** get_streams **

    #[test]
    async fn test_get_streams_search_by_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", now));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?userId={}&page={}&limit={}", user1.id, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
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
    #[test]
    async fn test_get_streams_search_by_page_limit_without_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", now));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?page={}&limit={}", page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
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
    #[test]
    async fn test_get_streams_search_by_user_id_page2() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", now));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));
        stream_vec.push(create_stream(4, user1.id, "demo31", "tag31,tag32", now));
        stream_vec.push(create_stream(5, user1.id, "demo32", "tag34,tag35", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info4 = stream_orm.stream_info_vec.get(4).unwrap().clone();
        let stream_info5 = stream_orm.stream_info_vec.get(5).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info4, stream_info5];
        let page = 2;
        let limit = 2;
        let uri = format!("/streams?userId={}&page={}&limit={}", user1.id, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
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
    #[test]
    async fn test_get_streams_search_by_another_user_id_with_role_user() {
        let user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user2.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user2.id, "demo12", "tag14,tag15", now));
        let uri = format!("/streams?userId={}&page=1&limit=2", user2.id);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1, user2], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NO_ACCESS_TO_STREAMS);
        assert_eq!(app_err.message, err::MSG_NO_ACCESS_TO_STREAMS);
    }
    #[test]
    async fn test_get_streams_search_by_another_user_id_with_role_admin() {
        let mut user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        user1.role = UserRole::Admin;
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user2.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user2.id, "demo12", "tag14,tag15", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let mut stream_info1 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        stream_info1.is_my_stream = false;
        let mut stream_info2 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        stream_info2.is_my_stream = false;
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info1, stream_info2];

        let page = 1;
        let limit = 2;
        let uri = format!("/streams?userId={}&page={}&limit={}", user2.id, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1, user2], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        let body = test::read_body(resp).await;
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

    #[test]
    async fn test_get_streams_search_by_live() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let live = true;
        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        let mut stream = create_stream(0, user1.id, "demo11", "tag11,tag12", now);
        stream.live = live;
        stream_vec.push(stream);
        let mut stream = create_stream(1, user1.id, "demo12", "tag14,tag15", now);
        stream.live = live;
        stream_vec.push(stream);
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?live={}&page={}&limit={}", live, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list[0].live, live);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_is_future() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", tomorrow));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", yesterday));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", yesterday));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?isFuture={}&page={}&limit={}", true, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_is_not_future() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", yesterday));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", yesterday));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", tomorrow));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?isFuture={}&page={}&limit={}", false, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_asc() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let next_2_day = now + Duration::days(2);
        let next_1_day = now + Duration::days(1);
        let prev_2_day = now - Duration::days(2);
        let prev_1_day = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", next_2_day));
        stream_vec.push(create_stream(1, user1.id, "demo13", "tag13,tag14", next_1_day));
        stream_vec.push(create_stream(2, user1.id, "demo15", "tag15,tag16", prev_1_day));
        stream_vec.push(create_stream(3, user1.id, "demo17", "tag17,tag18", prev_2_day));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_vec: Vec<StreamInfoDto> = vec![
            stream_orm.stream_info_vec.get(3).unwrap().clone(),
            stream_orm.stream_info_vec.get(2).unwrap().clone(),
            stream_orm.stream_info_vec.get(1).unwrap().clone(),
            stream_orm.stream_info_vec.get(0).unwrap().clone(),
        ];

        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Asc.to_string();
        let limit = 4;
        let page = 1;
        let uri = format!(
            "/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
            order_column, order_dir, page, limit
        );
        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_desc() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let next_2_day = now + Duration::days(2);
        let next_1_day = now + Duration::days(1);
        let prev_2_day = now - Duration::days(2);
        let prev_1_day = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo21", "tag21,tag22", prev_2_day));
        stream_vec.push(create_stream(1, user1.id, "demo23", "tag23,tag24", prev_1_day));
        stream_vec.push(create_stream(2, user1.id, "demo25", "tag25,tag26", next_1_day));
        stream_vec.push(create_stream(3, user1.id, "demo27", "tag27,tag28", next_2_day));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_vec: Vec<StreamInfoDto> = vec![
            stream_orm.stream_info_vec.get(3).unwrap().clone(),
            stream_orm.stream_info_vec.get(2).unwrap().clone(),
            stream_orm.stream_info_vec.get(1).unwrap().clone(),
            stream_orm.stream_info_vec.get(0).unwrap().clone(),
        ];

        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Desc.to_string();
        let limit = 4;
        let page = 1;
        let uri = format!(
            "/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
            order_column, order_dir, page, limit
        );
        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamInfoPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_streams_events **

    fn to_utc(value: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(value)
    }
    #[test]
    async fn test_get_streams_events_search_by_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        let d1 = today - Duration::seconds(1);
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", to_utc(d1)));
        let d2 = today;
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", to_utc(d2)));
        let d3 = today + Duration::hours(23) + Duration::minutes(59) + Duration::seconds(59);
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", to_utc(d3)));
        let d4 = today + Duration::hours(24);
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", to_utc(d4)));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream_info2 = stream_orm.stream_info_vec.get(2).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info1, stream_info2];

        let starttime = to_utc(today).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let uri = format!("/streams_events?userId={}&starttime={}&page={}&limit={}", user1.id, starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamEventDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_events_search_by_without_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", to_utc(today)));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", to_utc(today)));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", to_utc(today)));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let starttime = to_utc(today).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 1;
        let uri = format!("/streams_events?starttime={}&page={}&limit={}", starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamEventDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_events_search_by_page2() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", to_utc(today)));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", to_utc(today)));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", to_utc(today)));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info2 = stream_orm.stream_info_vec.get(2).unwrap().clone();
        let stream_info3 = stream_orm.stream_info_vec.get(3).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info2, stream_info3];
        let starttime = to_utc(today).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 2;
        let uri = format!("/streams_events?starttime={}&page={}&limit={}", starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamEventDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[test]
    async fn test_get_streams_events_search_by_bad_starttime() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", to_utc(today)));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", to_utc(today)));

        let starttime = to_utc(today - Duration::days(1)).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 1;
        let uri = format!("/streams_events?starttime={}&page={}&limit={}", starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, vec![]);
        assert_eq!(response.list.len(), 0);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 0);
        assert_eq!(response.page, 1);
        assert_eq!(response.pages, 0);
    }
    #[test]
    async fn test_get_streams_events_search_by_another_user_id_with_role_user() {
        let user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user2.id, "demo11", "tag11,tag12", to_utc(today)));
        stream_vec.push(create_stream(1, user2.id, "demo12", "tag14,tag15", to_utc(today)));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", to_utc(today)));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", to_utc(today)));

        let starttime = to_utc(today).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let uri = format!("/streams_events?userId={}&starttime={}&page={}&limit={}", user2.id, starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NO_ACCESS_TO_STREAMS);
        assert_eq!(app_err.message, err::MSG_NO_ACCESS_TO_STREAMS);
    }
    #[test]
    async fn test_get_streams_events_search_by_another_user_id_with_role_admin() {
        let mut user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        user1.role = UserRole::Admin;
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let today = Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0).unwrap();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user2.id, "demo11", "tag11,tag12", to_utc(today)));
        stream_vec.push(create_stream(1, user2.id, "demo12", "tag14,tag15", to_utc(today)));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", to_utc(today)));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", to_utc(today)));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let starttime = to_utc(today).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let limit = 2;
        let page = 1;
        #[rustfmt::skip]
        let uri = format!("/streams_events?userId={}&starttime={}&page={}&limit={}", user2.id, starttime, page, limit);

        // GET api/streams_events
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_events, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: StreamEventPageDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamEventDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    // ** get_streams_period **

    #[test]
    async fn test_get_streams_period_by_finish_less_start() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start - Duration::seconds(1);
        let start2 = to_utc(start).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let finish2 = to_utc(finish).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        #[rustfmt::skip]
        let uri = format!("/streams_period?userId={}&start={}&finish={}", user1.id, start2, finish2);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], vec![]);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_FINISH_LESS_START);
        assert_eq!(app_err.message, err::MSG_FINISH_LESS_START);
    }
    #[test]
    async fn test_get_streams_period_by_finish_more_on_2_month() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start2 = to_utc(start).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let finish2 = to_utc(finish).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        #[rustfmt::skip]
        let uri = format!("/streams_period?userId={}&start={}&finish={}", user1.id, start2, finish2);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], vec![]);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_FINISH_GREATER_MAX);
        let msg = format!("{} ({}).", err::MSG_FINISH_GREATER_MAX, PERIOD_MAX_NUMBER_DAYS);
        assert_eq!(app_err.message, msg);
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
        let start = to_utc(d2).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let finish = to_utc(d3).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();

        (stream_vec, start, finish, result_vec)
    }
    #[test]
    async fn test_get_streams_period_by_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        let (stream_vec, start, finish, res_vec) = get_streams2(user1.id);
        #[rustfmt::skip]
        let uri = format!("/streams_period?userId={}&start={}&finish={}", user1.id, start, finish);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        let body = test::read_body(resp).await;

        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
    #[test]
    async fn test_get_streams_period_by_without_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        let (stream_vec, start, finish, res_vec) = get_streams2(user1.id);
        #[rustfmt::skip]
        let uri = format!("/streams_period?start={}&finish={}", start, finish);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        let body = test::read_body(resp).await;

        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
    #[test]
    async fn test_get_streams_period_by_another_user_id_with_role_user() {
        let user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let dt = Local::now();
        let start = Local.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0).unwrap();
        let finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        let start2 = to_utc(start).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        let finish2 = to_utc(finish).format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
        #[rustfmt::skip]
        let uri = format!("/streams_period?userId={}&start={}&finish={}", user2.id, start2, finish2);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], vec![]);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NO_ACCESS_TO_STREAMS);
        assert_eq!(app_err.message, err::MSG_NO_ACCESS_TO_STREAMS);
    }
    #[test]
    async fn test_get_streams_period_by_another_user_id_with_role_admin() {
        let mut user1 = UserOrmApp::new_user(1, "Jacob_Moore", "Jacob_Moore@gmail.com", "passwdT1R1");
        user1.role = UserRole::Admin;
        let user2 = UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdT1R1");

        let user_orm = UserOrmApp::create(&vec![user1, user2]);
        let user1 = user_orm.user_vec.get(0).unwrap().clone();
        let user2 = user_orm.user_vec.get(1).unwrap().clone();

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        let (stream_vec, start, finish, res_vec) = get_streams2(user2.id);
        #[rustfmt::skip]
        let uri = format!("/streams_period?userId={}&start={}&finish={}", user2.id, start, finish);

        // GET api/streams_period
        let request = test::TestRequest::get().uri(&uri.to_string());
        let request = request.insert_header(header_auth(&token));
        let cfg_c = (config_jwt, config_strm::get_test_config());
        let data_c = (vec![user1], vec![session1], stream_vec);
        let resp = call_service1(cfg_c, data_c, get_streams_period, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: Vec<DateTime<Utc>> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_res_vec = serde_json::json!(res_vec).to_string();
        let res_vec_ser: Vec<DateTime<Utc>> = serde_json::from_slice(json_res_vec.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), res_vec_ser.len());
        assert_eq!(response, res_vec_ser);
    }
}

use std::{borrow::Cow, fs, ops::Deref, path};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, http::StatusCode, post, put, web, HttpResponse};
use chrono::{DateTime, Duration, SecondsFormat::Millis, Utc};
use log::error;
use mime::IMAGE;
use serde_json::{self, json};
use utoipa;
use vrb_authent::authentication::{Authenticated, RequireAuth};
use vrb_common::{
    alias_path::alias_path_stream,
    api_error::{code_to_str, ApiError},
    parser,
    validators::{self, msg_validation, ValidationChecks, Validator},
};
use vrb_dbase::db_enums::{StreamState, UserRole};
use vrb_tools::{cdis::coding, err, loading::dynamic_image};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm::{self, ConfigStrm},
    stream_models::{
        self, CreateStreamInfoDto, ModifyStream, ModifyStreamInfoDto, SearchStreamEventDto, SearchStreamInfoDto, SearchStreamPeriodDto,
        StreamConfigDto, StreamEventPageDto, StreamInfoDto, StreamInfoPageDto, ToggleStreamStateDto,
    },
    stream_orm::StreamOrm,
};

// ** Section: Stream Get **

pub const PERIOD_MAX_NUMBER_DAYS: u16 = 65;
// 406 Not Acceptable - The finish date is less than the start date.
pub const MSG_FINISH_LESS_START: &str = "finish_date_less_start_date";
// 413 Content Too Large - The finish date of the search period exceeds the limit.
pub const MSG_FINISH_EXCEEDS_LIMIT: &str = "finish_date_exceeds_limit";
// 403 Access denied - insufficient user rights.
pub const MSG_GET_LIST_OTHER_USER_STREAMS: &str = "get_list_other_users_streams";
// 403 Access denied - insufficient user rights.
pub const MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS: &str = "get_list_other_users_event_streams";
// 403 Access denied - insufficient user rights.
pub const MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD: &str = "get_period_other_users_streams";

// ** Section: Stream Post **
// ** Section: Stream Put **
// 406 Not acceptable - Error deserializing field tag. // Use: post_stream, put_stream
pub const MSG_INVALID_FIELD_TAG: &str = "invalid_field_tag";

// ** Section: Stream Put state **

// 406 Not acceptable - Not acceptable to go from old_state to new_state.
pub const MSG_INVALID_STREAM_STATE: &str = "invalid_stream_state";
// 409 Conflict - Exist is an active stream.
pub const MSG_EXIST_IS_ACTIVE_STREAM: &str = "exist_is_active_stream";

// ** Section: Stream Delete **
// ** **

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        //     GET /api/streams/{id}
        config
            .service(get_stream_by_id)
            // GET /api/streams
            .service(get_streams)
            // GET /api/streams_config
            .service(get_stream_config)
            // GET /api/streams_events
            .service(get_streams_events)
            // GET /api/streams_period
            .service(get_streams_period)
            // POST /api/streams
            .service(post_stream)
            // PUT /api/streams/toggle/{id}
            .service(put_toggle_state)
            // PUT /api/streams/{id}
            .service(put_stream)
            // DELETE /api/streams/{id}
            .service(delete_stream);
    }
}

pub fn get_file_name(user_id: i32, date_time: DateTime<Utc>) -> String {
    format!("{}_{}", user_id, coding::encode(date_time, 1))
}

// ** Section: Stream Get **

/// get_stream_by_id
///
/// Search for a stream by his ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams/1
/// ```
///
/// Return the found specified stream (`StreamInfoDto`) with status 200 or 204 (no content) if the stream is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "A stream with the specified ID was found.", body = StreamInfoDto),
        (status = 204, description = "The stream with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X GET http://localhost:8080/api/streams/2a`", 
            body = ApiError, example = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                , "`id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("id", description = "Unique stream ID.")),
    security(("bearer_auth" = [])),
)]#[rustfmt::skip]
#[get("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_by_id(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    // let opt_user_id: Option<i32> = if user.role == UserRole::Admin { None } else { Some(user.user_id) };

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}-{}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), &message);
        ApiError::new(416, &message) // 416
    })?;

    let res_data = web::block(move || {
        // Get 'stream' by id.
        let res_data = stream_orm
            .find_stream_by_params(Some(id), None, None, true, &[])
            .map_err(|e| {
                #[rustfmt::skip]
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
        res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_data = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let opt_stream_tag_dto = if let Some((stream, stream_tags)) = opt_data {
        let streams: Vec<stream_models::Stream> = vec![stream];
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, user.id);
        list.into_iter().nth(0)
    } else {
        None
    };

    if let Some(stream_tag_dto) = opt_stream_tag_dto {
        Ok(HttpResponse::Ok().json(stream_tag_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// get_streams
///
/// Get a list of your streams (page by page).
///
/// Request structure:
/// ```text
/// {
///   userId?: number,                      // optional
///   live?: boolean,                       // optional
///   futureStarttime?: DateTime<Utc>,      // optional
///   pastStarttime?: DateTime<Utc>,        // optional
///   orderColumn?: ["starttime", "title"], // optional
///   orderDirection?: ["asc", "desc"],     // optional
///   page?: number,                        // optional
///   limit?: number,                       // optional
/// }
/// Where:
/// "userId" - user identifier (current default user);
/// "live" - sign of a "live" stream ("state" = ["preparing", "started", "paused"]);
/// "futureStarttime" - get future streams with a "starttime" greater than or equal to the specified one (in Utc-format);
/// "pastStarttime" - get past streams with a "starttime" greater than or equal to the specified one (in Utc-format);
/// "orderColumn" - sorting column ["starttime" - (default), "title"];
/// "orderDirection" - sort order ["asc" - ascending (default), "desc" - descending];
/// "page" - page number, stratified from 1 (1 by default);
/// "limit" - number of records on the page (5 by default);
/// ```
/// It is recommended to enter the date and time in ISO8601 format.
/// ```text
/// var d1 = new Date();
/// { futureStarttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "futureStarttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams?orderColumn=starttime&orderDirection=asc&page=1&limit=5
/// ```
/// Could be called with all fields with the next curl.
/// Request future streams that start on or after a specified date (specify current date and time in Utc).
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams?userId=1&futureStarttime=2020-02-02T08:00:00.000Z&page=1&limit=5
/// ```
/// 
/// Could be called with all fields with the next curl.
/// Request past streams that started before the specified date (specify the current date and time in Utc).
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams?userId=1&pastStarttime=2020-02-02T08:00:00.000Z&page=1&limit=5
/// ```
/// 
/// Response structure:
/// ```text
/// {
///   list: [StreamInfoDto],
///   limit: number,
///   count: number,
///   page: number,
///   pages: number,
/// }
/// Where:
/// "list"  - array of streams;
/// "limit" - number of records on the page;
/// "count" - total number of records;
/// "page"  - current page number (stratified from 1);
/// "pages" - total pages with a given number of records on the page;
/// ```
/// 
/// Return found data on streams (`StreamInfoPageDto`) with status 200.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Result of the stream request.", body = StreamInfoPageDto),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::create(403, err::MSG_ACCESS_DENIED, 
                &format!("{}; curr_user_id: 1, user_id: 2", MSG_GET_LIST_OTHER_USER_STREAMS)) )),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamInfoDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();

    // Get search parameters.
    let search_stream_info_dto: SearchStreamInfoDto = query_params.into_inner();

    let page: u32 = search_stream_info_dto.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
    let limit: u32 = search_stream_info_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
    let search_stream = stream_models::SearchStream::convert(search_stream_info_dto, user.id);

    if search_stream.user_id != user.id && user.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", user.id, search_stream.user_id);
        let message = format!("{}; {}", MSG_GET_LIST_OTHER_USER_STREAMS, &text);
        error!("{}-{}", code_to_str(StatusCode::FORBIDDEN), &message);
        return Err(ApiError::create(403, err::MSG_ACCESS_DENIED, &message)); // 403
    }

    let res_data = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_data =
            stream_orm.find_streams_by_pages(search_stream, true).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
        res_data
        })
        .await
        .map_err(|e| {
            #[rustfmt::skip]
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;
        
    let (count, streams, stream_tags) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    // Merge a "stream" and a corresponding list of "tags".
    let list = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, user.id);

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = StreamInfoPageDto { list, limit, count, page, pages };

    Ok(HttpResponse::Ok().json(result)) // 200
}

/// get_streams_config
///
/// Get information about the image configuration settings in the stream (`StreamConfigDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_config
/// ```
///
/// Returns the configuration settings for the stream image (`StreamConfigDto`) with status 200.
///
/// The structure is returned:
/// ```text
/// {
///   logo_max_size?: Number,      // optional - Maximum size for logo files;
///   logo_valid_types: String[],  //          - List of valid input mime types for logo files;
/// //                                         ["image/bmp", "image/gif", "image/jpeg", "image/png"]
///   logo_ext?: String,           // optional - Logo files will be converted to this MIME type;
/// //                                  Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
///   logo_max_width?: Number,     // optional - Maximum width of logo image after saving;
///   logo_max_height?: Number,    // optional - Maximum height of logo image after saving;
/// }
/// ```
///
#[utoipa::path(
    responses(
        (status = 200, description = "Get information about the image configuration settings in the stream",
            body = StreamConfigDto,
            examples(
            ("max_config" = (summary = "maximum configuration", description = "Maximum configuration for logo image.",
                value = json!(StreamConfigDto::new(
                    Some(2*1024*1024), ConfigStrm::image_types(), Some(ConfigStrm::image_types()[0].clone()), Some(512), Some(512)))
            )),
            ("min_config" = (summary = "minimum configuration", description = "Minimum configuration for logo image.",
                value = json!(StreamConfigDto::new(None, ConfigStrm::image_types(), None, None, None))
            )), ),
        ),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/streams_config", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
#[rustfmt::skip]
pub async fn get_stream_config(config_strm: web::Data<ConfigStrm>) -> actix_web::Result<HttpResponse, ApiError> {
    let cfg_strm = config_strm;
    let max_size = if cfg_strm.strm_logo_max_size > 0 { Some(cfg_strm.strm_logo_max_size) } else { None };
    let valid_types = cfg_strm.strm_logo_valid_types.clone();
    let ext = cfg_strm.strm_logo_ext.clone();
    let max_width = if cfg_strm.strm_logo_max_width > 0 { Some(cfg_strm.strm_logo_max_width) } else { None };
    let max_height = if cfg_strm.strm_logo_max_height > 0 { Some(cfg_strm.strm_logo_max_height) } else { None };
    // Get configuration data.
    let stream_config_dto = StreamConfigDto::new(max_size, valid_types, ext, max_width, max_height);

    Ok(HttpResponse::Ok().json(stream_config_dto)) // 200
}

/// get_streams_events
///
/// Get a list with a brief description of your streams (page by page).
///
/// Request structure:
/// ```text
/// {
///   userId?: number,           // optional
///   starttime?: DateTime<Utc>, // optional
///   page?: number,             // optional
///   limit?: number,            // optional
/// }
/// Where:
/// "userId" - user identifier (current default user);
/// "starttime" - "starttime" - date and time (in Utc-format) to search for streams for this date;
/// "page" - page number, stratified from 1 (1 by default);
/// "limit" - number of records on the page (10 by default);
/// 
/// It is recommended to enter the date and time in ISO8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_events? \
///     starttime=2030-02-02T08:00:00.000Z&page=1
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_events?userId=1 \
///     &starttime=2030-02-02T08:00:00.000Z&page=1&limit=5
/// ```
/// Response structure:
/// ```text
/// {
///   list: [StreamEventDto],
///   limit: number,
///   count: number,
///   page: number,
///   pages: number,
/// }
/// Where:
/// "list"  - array of short streams;
/// "limit" - number of records on the page;
/// "count" - total number of records;
/// "page"  - current page number (stratified from 1);
/// "pages" - total pages with a given number of records on the page;
/// 
/// Return found data with short streams (`StreamEventPageDto`) with status 200.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Result of the short stream request.", body = StreamEventPageDto),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::create(403, err::MSG_ACCESS_DENIED, &format!("{}; {}",
                MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, "curr_user_id: 1, user_id: 2")))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams_events", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_events(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamEventDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();

    // Get search parameters.
    let search_stream_event_dto: SearchStreamEventDto = query_params.into_inner();

    let page: u32 = search_stream_event_dto.page.unwrap_or(stream_models::SEARCH_STREAM_EVENT_PAGE);
    let limit: u32 = search_stream_event_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_EVENT_LIMIT);
    let search_event = stream_models::SearchStreamEvent::convert(search_stream_event_dto, user.id);

    if search_event.user_id != user.id && user.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", user.id, search_event.user_id);
        let message = format!("{}; {}", MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, &text);
        error!("{}-{}", code_to_str(StatusCode::FORBIDDEN), &message);
        return Err(ApiError::create(403, err::MSG_ACCESS_DENIED, &message)); // 403
    }
    
    let res_data = web::block(move || {
        // Find for an entity (stream event) by SearchStreamEvent.
        let res_data =
            stream_orm.find_stream_events_by_pages(search_event).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
        res_data
        })
        .await
        .map_err(|e| {
            #[rustfmt::skip]
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;

    let (count, streams) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let list = streams.into_iter().map(|v| stream_models::StreamEventDto::from(v)).collect();

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = StreamEventPageDto { list, limit, count, page, pages };

    Ok(HttpResponse::Ok().json(result)) // 200
}

/// get_streams_period
///
/// Request structure:
/// ```text
/// {
///   userId?: number,       // optional
///   start: DateTime<Utc>,  // optional
///   finish: DateTime<Utc>, // optional
/// }
/// Where:
/// "userId" - user identifier (current default user);
/// "start" start date of the period (in Utc-format);
/// "finish" end date of the period (in Utc-format);
/// 
/// It is recommended to enter the date and time in ISO8601 format.
/// ```text
/// var d1 = new Date();
/// { start: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "start": "2020-01-20T22:10:57+02:00" }
/// 
/// The maximum period value (the difference between the "finish" date and the "start" date) is 65 days.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_period? \
///     start=2030-03-01T08:00:00.000Z&finish=2030-03-31T08:00:00.000Z
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_period?userId=1 \
///     &start=2030-03-01T08:00:00.000Z&finish=2030-03-31T08:00:00.000Z
/// ```
/// Return found dates that contain streams ([DateTime<Utc>]) with status 200.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Result is an array with dates containing streams.", body = Vec<DateTime<Utc>>, example = 
            json!([ "2030-04-01T08:00:00.000Z", "2030-04-04T08:00:00.000Z", "2030-04-10T08:00:00.000Z", "2030-04-0T08:00:00.000Z" ])),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::create(403, err::MSG_ACCESS_DENIED, &format!("{}; {}", 
                MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, "curr_user_id: 1, user_id: 2")))),
        (status = 406, description = "The finish date is less than the start date.", body = ApiError,
            example = json!(ApiError::new(406, MSG_FINISH_LESS_START).add_param(Cow::Borrowed("invalidPeriod"), &serde_json::json!(
                { "streamPeriodStart": "2030-03-02T08:00:00.000Z", "streamPeriodFinish": "2030-03-01T08:00:00.000Z" })) )),
        (status = 413, description = "The finish date of the search period exceeds the limit.", body = ApiError,
            example = json!(ApiError::new(413, MSG_FINISH_EXCEEDS_LIMIT).add_param(Cow::Borrowed("periodTooLong"), 
            &serde_json::json!({ "actualPeriodFinish": "2030-04-01T08:00:00.000Z", "maxPeriodFinish": "2030-03-10T08:00:00.000Z" 
                , "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS })))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams_period", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_period(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamPeriodDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();

    // Get search parameters.
    let search_period_dto: SearchStreamPeriodDto = query_params.into_inner();

    let search_period = stream_models::SearchStreamPeriod::convert(search_period_dto, user.id);
    let start = search_period.start.clone();
    let finish = search_period.finish.clone();

    if search_period.user_id != user.id && user.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", user.id, search_period.user_id);
        let message = format!("{}; {}", MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, &text);
        error!("{}-{}", code_to_str(StatusCode::FORBIDDEN), &message);
        return Err(ApiError::create(403, err::MSG_ACCESS_DENIED, &message)); // 403
    }
    if finish < start {
        let json = serde_json::json!({ "streamPeriodStart": start.to_rfc3339_opts(Millis, true)
            , "streamPeriodFinish": finish.to_rfc3339_opts(Millis, true) });
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), MSG_FINISH_LESS_START, json.to_string());
        return Err(ApiError::new(406, MSG_FINISH_LESS_START) // 406
            .add_param(Cow::Borrowed("invalidPeriod"), &json));
    }
    let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
    if max_finish <= finish {
        let json = serde_json::json!({ "actualPeriodFinish": finish.to_rfc3339_opts(Millis, true)
            , "maxPeriodFinish": max_finish.to_rfc3339_opts(Millis, true), "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        #[rustfmt::skip]
        error!("{}-{}: {}", code_to_str(StatusCode::PAYLOAD_TOO_LARGE), MSG_FINISH_EXCEEDS_LIMIT, json.to_string());
        return Err(ApiError::new(413, MSG_FINISH_EXCEEDS_LIMIT) // 413
            .add_param(Cow::Borrowed("periodTooLong"), &json));
    }

    let res_data = web::block(move || {
        // Find for an entity (stream period) by SearchStreamEvent.
        let res_data =
            stream_orm.find_streams_period(search_period).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)    
            });
        res_data
        })
        .await
        .map_err(|e| {
            #[rustfmt::skip]
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;

    let list: Vec<String> = match res_data {
        Ok(v) => v.iter().map(|d| d.to_rfc3339_opts(Millis, true)).collect(),
        Err(e) => return Err(e)
    };

    Ok(HttpResponse::Ok().json(list)) // 200

}

// ** Section: Stream Post **

// Convert the file to another mime type.
#[rustfmt::skip]
pub fn convert_logo_file(path_logo_file: &str, config_strm: config_strm::ConfigStrm, name: &str) -> Result<Option<String>, String> {
    let path: path::PathBuf = path::PathBuf::from(&path_logo_file);
    let file_source_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
    let strm_logo_ext = config_strm.strm_logo_ext.clone().unwrap_or(file_source_ext);
    // If you need to save in the specified format (img_ext.is_some()) or convert
    // to the specified size (img_max_width > 0 || img_max_height > 0), then do the following.
    if config_strm.strm_logo_ext.is_some()
        || config_strm.strm_logo_max_width > 0
        || config_strm.strm_logo_max_height > 0
    {
        // Convert the file to another mime type.
        let path_file = dynamic_image::convert_file(
            &path_logo_file,
            &strm_logo_ext,
            config_strm.strm_logo_max_width,
            config_strm.strm_logo_max_height,
        )?;
        if !path_file.eq(&path_logo_file) && path_logo_file.len() > 0 {
            let res_remove = fs::remove_file(path_logo_file);
            if let Err(err) = res_remove {
                error!("{} remove_file({}): error: {:?}", name, path_logo_file, err);
            }
        }
        Ok(Some(path_file))
    } else {
        Ok(None)
    }
}

fn new_stream_dto(title: &str, descript: &str, starttime: &str, tag_list: &str) -> CreateStreamInfoDto {
    #[rustfmt::skip]
    let descript = if descript.len() > 0 { Some(descript.to_string()) } else { None };
    // let starttime1 = if starttime.len() > 0 { Some(DateTime::parse_from_rfc3339(starttime).unwrap().with_timezone(&Utc)) } else {None};

    let start = if starttime.len() > 0 { Some(starttime) } else { None };
    let starttime = start.map(|val| DateTime::parse_from_rfc3339(val).unwrap().with_timezone(&Utc));

    let tags: Vec<String> = tag_list.split(',').map(|val| val.to_string()).collect();

    CreateStreamInfoDto {
        title: title.to_string(),
        descript,
        starttime,
        source: None,
        tags,
    }
}

#[derive(Debug, MultipartForm)]
pub struct CreateStreamForm {
    pub title: Text<String>,
    pub descript: Option<Text<String>>,
    pub starttime: Option<Text<DateTime<Utc>>>,
    pub source: Option<Text<String>>,
    pub tags: Text<String>,
    pub logofile: Option<TempFile>,
}

impl CreateStreamForm {
    pub fn convert(create_stream_form: CreateStreamForm) -> Result<(CreateStreamInfoDto, Option<TempFile>), String> {
        let val = create_stream_form.tags.into_inner();
        let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
        if let Err(err) = res_tags {
            return Err(err.to_string());
        }
        let tags: Vec<String> = res_tags.unwrap();

        Ok((
            CreateStreamInfoDto {
                title: create_stream_form.title.to_string(),
                descript: create_stream_form.descript.map(|v| v.to_string()),
                starttime: create_stream_form.starttime.map(|v| v.into_inner()),
                source: create_stream_form.source.map(|v| v.to_string()),
                tags,
            },
            create_stream_form.logofile,
        ))
    }
}

/// post_stream
/// 
/// Create a new stream.
/// 
/// Multipart/form-data is used to transfer data.
/// 
/// Request structure:
/// ```text
/// {
///   title: String,             // required
///   descript?: String,         // optional
///   starttime?: DateTime<Utc>, // optional
///   source?: String,           // optional
///   tags: String,              // required
///   logofile?: TempFile,       // optional
/// }
/// ```
/// Where:
/// "title" - stream title;
/// "descript" - description of the stream;
/// "starttime" - date and time (in Utc-format "2020-01-20T20:10:57.000Z") of the start of the stream;
/// "source" - source value ("obs" by default) of the stream;
/// "tags" - serialized array of string values of stream tags("['tag1','tag2']");
/// "logofile" - attached stream image file (jpeg,gif,png,bmp);
/// 
/// The "starttime" field indicates the date of the future stream.
/// And cannot contain a past period (date and time).
/// 
/// It is recommended to enter the date and time in ISO 8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// The "tags" field represents a serialized array of string values.
/// ```text
/// { 'tags': JSON.stringify(["tag1", "tag2"]) } // "['tag1', 'tag2']"
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title1" -F "tags=['tag1','tag2']"
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
/// -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']"
/// ```
/// Additionally, you can specify the name of the image file.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
/// -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']" -F "logofile=@image.jpg"
/// ```
///  
/// Return a new stream (`StreamInfoDto`) with status 201.
/// 
#[utoipa::path(
    responses(
        (status = 201, description = "Create a new stream.", body = StreamInfoDto,
            example = json!(new_stream_dto("Stream title", "Description of the stream.", "2020-01-20T20:10:57.000Z", "tag1,tag2")) ),
        (status = 406, description = "Error deserializing field \"tags\". `curl -X POST http://localhost:8080/api/streams
            -F 'title=title' -F 'tags=[\"tag\"'`",
            body = ApiError, example = json!(ApiError::create(406, MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6"))),
        (status = 413, description = "Invalid image file size. `curl -i -X POST http://localhost:8080/api/streams
            -F 'title=title2'  -F 'tags=[\"tag1\"]' -F 'logofile=@image.jpg'`", body = ApiError,
            example = json!(ApiError::new(413, err::MSG_INVALID_FILE_SIZE).add_param(Cow::Borrowed("invalidFileSize"),
                &json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X POST http://localhost:8080/api/streams
            -F 'title=title3'  -F 'tags=[\"tag3\"]' -F 'logofile=@image.svg'`", body = ApiError,
            example = json!(ApiError::new(415, err::MSG_INVALID_FILE_TYPE).add_param(Cow::Borrowed("invalidFileType"),
                &json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 417, description = "Validation error. `curl -X POST http://localhost:8080/api/streams
            -F 'title=t' -F 'descript=d' -F 'starttime=2020-01-20T20:10:57.000Z' -F 'tags=[]'`", body = [ApiError],
            example = json!(ApiError::validations((new_stream_dto("u", "d", "2020-01-20T20:10:57.000Z", "")).validate().err().unwrap()))
        ),
        (status = 500, description = "Error loading file.", body = ApiError, example = json!(
            ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg - File not found."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = ApiError,
            example = json!(ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\""))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[post("/api/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    MultipartForm(create_stream_form): MultipartForm<CreateStreamForm>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let curr_user_id = user.id;

    // Get data from MultipartForm.
    let (create_stream_info_dto, logo_file) = CreateStreamForm::convert(create_stream_form)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), MSG_INVALID_FIELD_TAG, &e);
            ApiError::create(406, MSG_INVALID_FIELD_TAG, &e) // 406
        })?;

    // Checking the validity of the data model.
    let validation_res = create_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let config_strm = config_strm.get_ref().clone();
    let mut path_new_logo_file = "".to_string();

    while let Some(temp_file) = logo_file {
        // If the size of the new file is zero, then we are not adding any file.
        if temp_file.size == 0 {
            break;
        }
        // Check file size for maximum value.
        let logo_max_size = usize::try_from(config_strm.strm_logo_max_size).unwrap();
        if logo_max_size > 0 && temp_file.size > logo_max_size {
            let json = json!({ "actualFileSize": temp_file.size, "maxFileSize": logo_max_size });
            error!("{}-{}; {}", code_to_str(StatusCode::PAYLOAD_TOO_LARGE), err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(ApiError::new(413, err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }
        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types = config_strm.strm_logo_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            error!("{}-{}; {}", code_to_str(StatusCode::UNSUPPORTED_MEDIA_TYPE), err::MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(ApiError::new(415, err::MSG_INVALID_FILE_TYPE) // 415
                .add_param(Cow::Borrowed("invalidFileType"), &json));
        }
        // Get the file stem and extension for the new file.
        #[rustfmt::skip]
        let name = format!("{}.{}", get_file_name(curr_user_id, Utc::now()), file_mime_type.replace(&format!("{}/", IMAGE), ""));
        // Add 'file path' + 'file name'.'file extension'.
        let path: path::PathBuf = [&config_strm.strm_logo_files_dir, &name].iter().collect();
        let full_path_file = path.to_str().unwrap().to_string();
        // Persist the temporary file at the target path.
        // If a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            let msg = format!("{} - {}", &full_path_file, err.to_string());
            error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_UPLOAD_FILE, &msg);
            return Err(ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, &msg)) // 500
        }
        path_new_logo_file = full_path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "post_stream()")
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), err::MSG_ERROR_CONVERT_FILE, &e);
                ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, &e) // 510
            })?;
        if let Some(new_path_file) = res_convert_logo_file {
            path_new_logo_file = new_path_file;
        }
        
        break;
    }
    let tags = create_stream_info_dto.tags.clone();
    let mut create_stream = stream_models::CreateStream::convert(create_stream_info_dto.clone(), curr_user_id);
    
    let alias_path_strm = alias_path_stream::AliasStrm::new(&config_strm.strm_logo_files_dir);
    let alias_strm = alias_path_strm.as_ref();

    if path_new_logo_file.len() > 0 {
        // Replace file path prefix with alias.
        let alias_logo_file= alias_strm.path_to_alias(&path_new_logo_file);
        create_stream.logo = Some(alias_logo_file);
    }

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data = stream_orm.create_stream(create_stream, &tags).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    if res_data.is_err() {
        if path_new_logo_file.len() > 0 {
            if let Err(err) = fs::remove_file(&path_new_logo_file) {
                error!("{} remove_file({}): error: {:?}", "post_stream()", &path_new_logo_file, err);
            }
        }
    }
    let (stream, stream_tags) = res_data?;
    // Merge a "stream" and a corresponding list of "tags".
    let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
    let stream_info_dto = list[0].clone();

    Ok(HttpResponse::Created().json(stream_info_dto)) // 201
}

// ** Section: Stream Put **

#[derive(Debug, MultipartForm)]
pub struct ModifyStreamForm {
    pub title: Option<Text<String>>,
    pub descript: Option<Text<String>>,
    pub starttime: Option<Text<DateTime<Utc>>>,
    pub source: Option<Text<String>>,
    pub tags: Option<Text<String>>,
    pub logofile: Option<TempFile>,
}

impl ModifyStreamForm {
    pub fn convert(modify_stream_form: ModifyStreamForm) -> Result<(stream_models::ModifyStreamInfoDto, Option<TempFile>), String> {
        let tags: Option<Vec<String>> = match modify_stream_form.tags {
            Some(v) => {
                let val = v.into_inner();
                let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
                if let Err(err) = res_tags {
                    return Err(err.to_string());
                }
                Some(res_tags.unwrap())
            }
            None => None,
        };
        Ok((
            stream_models::ModifyStreamInfoDto {
                title: modify_stream_form.title.map(|v| v.into_inner()),
                descript: modify_stream_form.descript.map(|v| v.into_inner()),
                starttime: modify_stream_form.starttime.map(|v| v.into_inner()),
                source: modify_stream_form.source.map(|v| v.into_inner()),
                tags,
            },
            modify_stream_form.logofile,
        ))
    }
}

/// put_stream
///
/// Update the stream with new data.
///
/// Multipart/form-data is used to transfer data.
///
/// Request structure:
/// ```text
/// {
///   title?: String,            // optional - stream title;
///   descript?: String,         // optional - description of the stream;
///   starttime?: DateTime<Utc>, // optional - date and time of the start of the stream;
///   source?: String,           // optional - source value ("obs" by default) of the stream;
///   tags?: String,             // optional - serialized array of string values of stream tags("['tag1','tag2']");
///   logofile?: TempFile,       // optional - attached stream image file (jpeg,gif,png,bmp);
/// }
/// ```
/// The "starttime" field is specified in the Utc format: "2020-01-20T20:10:57.000Z".
/// 
/// The "starttime" field indicates the date of the future stream.
/// And cannot contain a past period (date and time).
///
/// It is recommended to enter the date and time in ISO 8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// The "tags" field represents a serialized array of string values.
/// ```text
/// { 'tags': JSON.stringify(["tag1", "tag2"]) } // "['tag1', 'tag2']"
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams/1 -F "title=title1" -F "tags=['tag1','tag2']"
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams/1 -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']"
/// ```
/// Additionally, you can specify the name of the image file.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams/1 -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']" -F "logofile=@image.jpg"
/// ```
///  
/// Return the stream with updated data (`StreamInfoDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Update the stream with new data.", body = StreamInfoDto),
        (status = 204, description = "The stream with the specified ID was not found."),
        (status = 406, description = "Error deserializing field \"tags\". `curl -X PUT http://localhost:8080/api/streams/1
            -F 'title=title' -F 'tags=[\"tag\"'`",
            body = ApiError, example = json!(ApiError::create(406, MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6"))),
        (status = 413, description = "Invalid image file size. `curl -i -X PUT http://localhost:8080/api/streams/1
            -F 'title=title2'  -F 'tags=[\"tag1\"]' -F 'logofile=@image.jpg'`", body = ApiError,
            example = json!(ApiError::new(413, err::MSG_INVALID_FILE_SIZE).add_param(Cow::Borrowed("invalidFileSize"),
                &json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X PUT http://localhost:8080/api/streams/1
            -F 'title=title3'  -F 'tags=[\"tag3\"]' -F 'logofile=@image.svg'`", body = ApiError,
            example = json!(ApiError::new(415, err::MSG_INVALID_FILE_TYPE).add_param(Cow::Borrowed("invalidFileType"),
                &json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X PUT http://localhost:8080/api/streams/2a
                -F 'title=title3'  -F 'tags=[\"tag3\"]'`", body = ApiError,
            example = json!(ApiError::new(416, 
                &format!("{}; {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)")))),
        (status = 417, description = "Validation error. `curl -X PUT http://localhost:8080/api/streams/1
            -F 'title=t' -F 'descript=d' -F 'starttime=2020-01-20T20:10:57.000Z' -F 'tags=[]'`", body = [ApiError],
            example = json!(ApiError::validations(
                (ModifyStreamInfoDto {
                    title: Some("u".to_string()),
                    descript: Some("d".to_string()),
                    starttime: Some(DateTime::parse_from_rfc3339("2020-01-20T20:10:57.000Z").unwrap().with_timezone(&Utc)),
                    source: None,
                    tags: Some(vec!()),
                }).validate().err().unwrap()) )),
        (status = 500, description = "Error loading file.", body = ApiError, example = json!(
            ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg - File not found."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = ApiError,
            example = json!(ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\""))),
    ),
    security(("bearer_auth" = [])),
)]
// PUT /api/streams/{id}
#[rustfmt::skip]
#[put("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    MultipartForm(modify_stream_form): MultipartForm<ModifyStreamForm>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}-{}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), &message);
        ApiError::new(416, &message) // 416
    })?;

    // Get data from MultipartForm.
    let (modify_stream_info_dto, logo_file) = ModifyStreamForm::convert(modify_stream_form)
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), MSG_INVALID_FIELD_TAG, &e);
        ApiError::create(406, MSG_INVALID_FIELD_TAG, &e) // 406
    })?;

    // If there is not a single field in the MultipartForm, it gives an error 400 "Multipart stream is incomplete".

    // Checking the validity of the data model.
    let validation_res = modify_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        let mut is_no_fields_to_update = false;
        let errors = validation_errors.iter().map(|err| {
            if !is_no_fields_to_update && err.params.contains_key(validators::NM_NO_FIELDS_TO_UPDATE) {
                is_no_fields_to_update = true;
                let valid_names = [ModifyStreamInfoDto::valid_names(), vec!["logofile"]].concat().join(",");
                ValidationChecks::no_fields_to_update(&[false], &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err().unwrap()
            } else {
                err.clone()
            }
        }).collect();
        if !is_no_fields_to_update || logo_file.is_none() {
            error!("{}: {}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&errors));
            return Ok(ApiError::to_response(&ApiError::validations(errors))); // 417
        }
    }

    let mut logo: Option<Option<String>> = None;
    let config_strm = config_strm.get_ref().clone();
    let mut path_new_logo_file = "".to_string();

    while let Some(temp_file) = logo_file {
        // Delete the old version of the logo file.
        if temp_file.size == 0 {
            logo = Some(None); // Set the "logo" field to `NULL`.
            break;
        }
        let logo_max_size = usize::try_from(config_strm.strm_logo_max_size).unwrap();
        // Check file size for maximum value.
        if logo_max_size > 0 && temp_file.size > logo_max_size {
            let json = json!({ "actualFileSize": temp_file.size, "maxFileSize": logo_max_size });
            error!("{}-{}; {}", code_to_str(StatusCode::PAYLOAD_TOO_LARGE), err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(ApiError::new(413, err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }

        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types: Vec<String> = config_strm.strm_logo_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            error!("{}-{}; {}", code_to_str(StatusCode::UNSUPPORTED_MEDIA_TYPE), err::MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(ApiError::new(415, err::MSG_INVALID_FILE_TYPE) // 415
                .add_param(Cow::Borrowed("invalidFileType"), &json));
        }

        // Get the file stem and extension for the new file.
        #[rustfmt::skip]
        let name = format!("{}.{}", get_file_name(user.id, Utc::now()), file_mime_type.replace(&format!("{}/", IMAGE), ""));
        // Add 'file path' + 'file name'.'file extension'.
        let path: path::PathBuf = [&config_strm.strm_logo_files_dir, &name].iter().collect();
        let full_path_file = path.to_str().unwrap().to_string();
        // Persist the temporary file at the target path.
        // Note: if a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            let message = format!("{}; {} - {}", err::MSG_ERROR_UPLOAD_FILE, &full_path_file, err.to_string());
            error!("{}-{}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), &message);
            return Err(ApiError::new(500, &message)); // 500
        }
        path_new_logo_file = full_path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "put_stream()")
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), err::MSG_ERROR_CONVERT_FILE, &e);
                ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, &e) // 510
            })?;
        if let Some(new_path_file) = res_convert_logo_file {
            path_new_logo_file = new_path_file;
        }
        break;
    }

    let alias_path_strm = alias_path_stream::AliasStrm::new(&config_strm.strm_logo_files_dir);
    let alias_strm = alias_path_strm.as_ref();

    if path_new_logo_file.len() > 0 {
        // Replace file path prefix with alias.
        let alias_logo_file= alias_strm.path_to_alias(&path_new_logo_file);
        logo = Some(Some(alias_logo_file));
    }
    let tags = modify_stream_info_dto.tags.clone();
    let mut modify_stream: ModifyStream = modify_stream_info_dto.into();
    modify_stream.logo = logo;
    let opt_user_id: Option<i32> = if user.role == UserRole::Admin { None } else { Some(user.id) };

    let (path_old_logo_file, res_data_stream) = web::block(move || {
        let mut old_logo_file = "".to_string();
        if modify_stream.logo.is_some() {
            // Get the logo file name for an entity (stream) by ID.
            let res_get_stream_logo = stream_orm.get_stream_logo_by_id(id)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });

            if let Ok(Some(old_logo)) = res_get_stream_logo {
                old_logo_file = old_logo;
            }
        }
        // Modify an entity (stream).
        let res_data_stream = stream_orm.modify_stream(id, opt_user_id, modify_stream, tags)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });

        (old_logo_file, res_data_stream)
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_data_stream = res_data_stream
    .map_err(|err| {
        if path_new_logo_file.len() > 0 {
            if let Err(err) = fs::remove_file(&path_new_logo_file) {
                error!("put_stream() remove_file({}): error: {:?}", &path_new_logo_file, err);
            }
        }
        err
    })?;

    if let Some((stream, stream_tags)) = opt_data_stream {
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, user.id);
        let stream_info_dto: StreamInfoDto = list[0].clone();

        // If the file path starts with alice, then the file corresponds to the entity type.
        // And only then can the file be deleted.
        if alias_strm.starts_with_alias(&path_old_logo_file) {
            // Return file path prefix instead of alias.
            let full_path_logo = alias_strm.alias_to_path(&path_old_logo_file);
            if let Err(err) = fs::remove_file(&full_path_logo) {
                error!("put_stream() remove_file({}): error: {:?}", &full_path_logo, err);
            }
        }

        Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
    } else {
        if path_new_logo_file.len() > 0 {
            if let Err(err) = fs::remove_file(&path_new_logo_file) {
                error!("put_stream() remove_file({}): error: {:?}", &path_new_logo_file, err);
            }
        }
        Ok(HttpResponse::NoContent().finish()) // 204        
    }
}

// ** Section: Stream Put state **

/// put_toggle_state
///
/// Update the stream state.
///
/// Request structure:
/// ```text
/// {
///   state: StreamState,
/// }
/// ```
/// The "StreamState" type accepts the following values: "waiting", "preparing", "started", "paused", "stopped".
/// 
/// The new "state" value must be different from the old value.
/// 
/// By default, the stream has the "waiting" state.
/// In the "waiting" state, the stream is waiting for the broadcast start date and time.
/// 
/// From "waiting", the stream can be switched to the "preparing" state.
/// In the "preparing" state, you can prepare for the start of the broadcast: select web cameras,
/// microphone, check other settings.
/// 
/// From "preparing", the stream can be switched to the "started" state.
/// In the "started" state, the stream is broadcast.
/// 
/// From "preparing", the stream can be switched to the "stopped" state.
/// In the "stopped" state, the stream broadcast is stopped.
/// 
/// From "started", the stream can be switched to the "paused" state.
/// In the "paused" state, the stream broadcast is temporarily stopped.
/// 
/// From "paused", the stream can be switched to the "started" state.
/// 
/// From "paused", the stream can be switched to the "stopped" state.
/// 
/// From "started" a stream can be moved to the "stopped" state.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams/toggle/1  -d '{"state": "started"}'
/// ```
#[utoipa::path(
    responses(
        (status = 200, description = "Update the stream with new data.", body = StreamInfoDto),
        (status = 204, description = "The stream with the specified ID was not found."),
        (status = 406, description = "Unacceptable stream state.", body = ApiError,
            examples(
            ("old_equals_new" = (summary = "old state equals new", description = "Unacceptable transition: old flow state equals new.",
                value = json!(ApiError::new(406, MSG_INVALID_STREAM_STATE).add_param(Cow::Borrowed("invalidState"),
                        &json!({ "oldState": StreamState::Preparing, "newState": StreamState::Preparing })) ) )
            ),
            ("unacceptable" = (
                summary = "unacceptable state", description = "An unacceptable transition from an old state of flow to a new one.",
                value = json!(ApiError::new(406, MSG_INVALID_STREAM_STATE).add_param(Cow::Borrowed("invalidState"),
                        &json!({ "oldState": StreamState::Started, "newState": StreamState::Preparing })) ) )
            ) ),
        ),
        (status = 409, description = "There is already an active stream.", body = ApiError,
            example = json!(ApiError::new(409, MSG_EXIST_IS_ACTIVE_STREAM)
                    .add_param(Cow::Borrowed("activeStream"), &json!({ "id": 123, "title": Cow::Borrowed("Trip to Greece.") })) )
        ),
        (status = 416, description = "Error parsing input parameter. `curl -i -X PUT http://localhost:8080/api/streams/toggle/2a 
            -d '{\"state\": \"started\"}'`", body = ApiError,
            example = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
// PUT /api/streams/toggle/{id}
#[rustfmt::skip]
#[put("/api/streams/toggle/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_toggle_state(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<ToggleStreamStateDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let user = authenticated.deref();
    let opt_user_id: Option<i32> = if user.role == UserRole::Admin { None } else { Some(user.id) };

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), &message);
        ApiError::new(416, &message) // 416
    })?;

    let new_state: StreamState = json_body.into_inner().state;
    let stream_orm2 = stream_orm.clone();

    let res_stream_tags = web::block(move || {
        // Find a stream by ID.
        let res_stream_tags = stream_orm2
            .find_stream_by_params(Some(id), opt_user_id, None, false, &[])
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
        res_stream_tags
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_tags = match res_stream_tags { Ok(v) => v, Err(e) => return Err(e) };

    if opt_stream_tags.is_none() {
        // If a stream with the specified ID is not found for the current user, then return status 204.
        return Ok(HttpResponse::NoContent().finish()) // 204
    }
    let (stream, _tags) = opt_stream_tags.unwrap();

    if stream.state == new_state {
        let json = json!({ "oldState": &stream.state, "newState": &new_state });
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), MSG_INVALID_STREAM_STATE, json.to_string());
        return Err(ApiError::new(406, MSG_INVALID_STREAM_STATE) // 406
            .add_param(Cow::Borrowed("invalidState"), &json));
    }

    let is_not_acceptable = match new_state {
        StreamState::Preparing => vec![StreamState::Started, StreamState::Paused].contains(&stream.state),
        StreamState::Started => vec![StreamState::Waiting, StreamState::Stopped].contains(&stream.state),
        StreamState::Paused => vec![StreamState::Waiting, StreamState::Stopped, StreamState::Preparing].contains(&stream.state),
        StreamState::Stopped => vec![StreamState::Waiting].contains(&stream.state),
        _ => false,
    };
    if is_not_acceptable {
        let json = json!({ "oldState": &stream.state.to_string(), "newState": &new_state });
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), MSG_INVALID_STREAM_STATE, json.to_string());
        return Err(ApiError::new(406, MSG_INVALID_STREAM_STATE) // 406
            .add_param(Cow::Borrowed("invalidState"), &json));
    }
    // If the stream goes into active state, then
    if vec![StreamState::Preparing, StreamState::Started, StreamState::Paused].contains(&new_state) {
        let stream_orm2 = stream_orm.clone();
        // find any stream in active state.
        let res_stream2_tags = web::block(move || {
            let res_stream2_tags = stream_orm2
                .find_stream_by_params(None, opt_user_id, Some(true), false, &[id])
                .map_err(|e| {
                    error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                    ApiError::create(507, err::MSG_DATABASE, &e)
                });
            res_stream2_tags
        })
        .await
        .map_err(|e| {
            #[rustfmt::skip]
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;
        
        let opt_stream2_tags = match res_stream2_tags { Ok(v) => v, Err(e) => return Err(e) };

        if let Some((stream2, _tags)) = opt_stream2_tags {
            let json = json!({ "id": stream2.id, "title": &stream2.title });
            error!("{}-{}; {}", code_to_str(StatusCode::CONFLICT), MSG_EXIST_IS_ACTIVE_STREAM, json.to_string());
            return Err(ApiError::new(409, MSG_EXIST_IS_ACTIVE_STREAM) // 409
                .add_param(Cow::Borrowed("activeStream"), &json));
        }
    }

    let modify_stream: ModifyStream = ModifyStream {
        title: None,
        descript: None,
        logo: None,
        starttime: None,
        state: Some(new_state),
        started: None,
        stopped: None,
        source: None,
    };

    let res_stream_tags = web::block(move || {
        // Modify an entity (stream).
        let res_stream_tags = stream_orm.modify_stream(id, opt_user_id, modify_stream, None)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });
        res_stream_tags
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_tags = match res_stream_tags { Ok(v) => v, Err(e) => return Err(e) };

    if opt_stream_tags.is_none() {
        // If a stream with the specified ID is not found for the current user, then return status 204.
        return Ok(HttpResponse::NoContent().finish()) // 204
    }
    let (stream, tags) = opt_stream_tags.unwrap();

    // Merge a "stream" and a corresponding list of "tags".
    let list = StreamInfoDto::merge_streams_and_tags(&[stream], &tags, user.id);
    let stream_info_dto: StreamInfoDto = list[0].clone();
    Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
}

// ** Section: Stream Delete **

/// delete_stream
///
/// Delete the specified stream.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/streams/1
/// ```
///
/// Return the deleted stream (`StreamInfoDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified stream was deleted successfully.", body = StreamInfoDto),
        (status = 204, description = "The specified stream was not found."),
        (status = 416, description = "Error parsing input parameter. `curl -i -X DELETE http://localhost:8080/api/streams/2a`",
            body = ApiError, example = json!(ApiError::create(416, 
                err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("id", description = "Unique stream ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = format!("`{}` - {}", "id", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;

    let opt_user_id: Option<i32> = if user.role == UserRole::Admin { None } else { Some(user.id) };
    let res_stream = web::block(move || {
        // Add a new entity (stream).
        let res_data = stream_orm.delete_stream(id, opt_user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream = res_stream?;

    if let Some((stream, stream_tags)) = opt_stream {
        let config_strm = config_strm.get_ref().clone();
        let alias_path_strm = alias_path_stream::AliasStrm::new(&config_strm.strm_logo_files_dir);
        let alias_strm = alias_path_strm.as_ref();
        // Get the path to the "logo" file.
        let path_file_img: String = stream.logo.clone().unwrap_or("".to_string());

        // If the file path starts with alice, then the file corresponds to the entity type.
        // And only then can the file be deleted.
        if alias_strm.starts_with_alias(&path_file_img) {
            // Return file path prefix instead of alias.
            let full_path_logo = alias_strm.alias_to_path(&path_file_img);
            if let Err(err) = fs::remove_file(&full_path_logo) {
                error!("delete_stream() remove_file({}): error: {:?}", &full_path_logo, err);
            }
        }
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, user.id);
        let stream_info_dto = list[0].clone();
        Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::http;
    use vrb_common::api_error::ApiError;
    use vrb_tools::token_data::BEARER;

    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    pub fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
}

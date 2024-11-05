use std::{borrow::Cow, ops::Deref};

use actix_web::{get, web, HttpResponse};
use chrono::{Duration, SecondsFormat::Millis};
use utoipa;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm::ConfigStrm,
    stream_models::{
        self, SearchStreamEventDto, SearchStreamInfoDto, SearchStreamPeriodDto, StreamConfigDto, StreamEventPageDto,
        StreamInfoDto, StreamInfoPageDto,
    },
    stream_orm::StreamOrm,
};
use crate::users::user_models::UserRole;
use crate::utils::parser;

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

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        // GET /api/streams/{id}
        config
            .service(get_stream_by_id)
            // GET /api/streams
            .service(get_streams)
            // GET /api/streams_config
            .service(get_stream_config)
            // GET /api/streams_events
            .service(get_streams_events)
            // GET /api/streams_period
            .service(get_streams_period);
    }
}

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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X GET http://localhost:8080/api/streams/2a`", 
            body = AppError, example = json!(AppError::range_not_satisfiable416(
                &format!("{}: {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique stream ID.")),
    security(("bearer_auth" = [])),
)]#[rustfmt::skip]
#[get("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_by_id(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();
    let opt_user_id: Option<i32> = if profile.role == UserRole::Admin { None } else { Some(profile.user_id) };

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;

    let res_data = web::block(move || {
        // Get 'stream' by id.
        let res_data = stream_orm
            .find_streams_by_params(Some(id), opt_user_id, None, true, Vec::<i32>::new())
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        res_data
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })?;

    let opt_data = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let opt_stream_tag_dto = if let Some((stream, stream_tags)) = opt_data {
        let streams: Vec<stream_models::Stream> = vec![stream];
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, profile.user_id);
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
///   userId?: number,          // optional
///   live?: boolean,           // optional
///   isFuture?: boolean,       // optional
///   orderColumn?: ["starttime", "title"], // optional
///   orderDirection?: ["asc", "desc"],  // optional
///   page?: number,            // optional
///   limit?: number,           // optional
/// }
/// Where:
/// "userId" - user identifier (current default user);
/// "live" - sign of a "live" stream ("state" = ["preparing", "started", "paused"]);
/// "isFuture" - a sign that the stream will start in the future;
/// "orderColumn" - sorting column ["starttime" - (default), "title"];
/// "orderDirection" - sort order ["asc" - ascending (default), "desc" - descending];
/// "page" - page number, stratified from 1 (1 by default);
/// "limit" - number of records on the page (5 by default);
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams?page=1&limit=5
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams?userId=1&live=false \
///     &isFuture=true&orderColumn=starttime&orderDirection=asc&page=1&limit=5
/// ```
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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(&format!("{}: {}: {}", err::MSG_ACCESS_DENIED,
                MSG_GET_LIST_OTHER_USER_STREAMS, "curr_user_id: 1, user_id: 2")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();

    // Get search parameters.
    let search_stream_info_dto: SearchStreamInfoDto = query_params.into_inner();

    let page: u32 = search_stream_info_dto.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
    let limit: u32 = search_stream_info_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
    let search_stream = stream_models::SearchStream::convert(search_stream_info_dto, profile.user_id);

    if search_stream.user_id != profile.user_id && profile.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", profile.user_id, search_stream.user_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS, &text);
        log::error!("{}: {}", err::CD_FORBIDDEN, &message);
        return Err(AppError::forbidden403(&message)); // 403
    }

    let res_data = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_data =
            stream_orm.find_streams(search_stream, true).map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_data
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string())
        })?;
        
    let (count, streams, stream_tags) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    // Merge a "stream" and a corresponding list of "tags".
    let list = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, profile.user_id);

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = StreamInfoPageDto { list, limit, count, page, pages };

    Ok(HttpResponse::Ok().json(result)) // 200
}

/// streams_config
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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/streams_config", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
#[rustfmt::skip]
pub async fn get_stream_config(config_strm: web::Data<ConfigStrm>) -> actix_web::Result<HttpResponse, AppError> {
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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(&format!("{}: {}: {}", err::MSG_ACCESS_DENIED,
                MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, "curr_user_id: 1, user_id: 2")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams_events", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_events(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamEventDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();

    // Get search parameters.
    let search_stream_event_dto: SearchStreamEventDto = query_params.into_inner();

    let page: u32 = search_stream_event_dto.page.unwrap_or(stream_models::SEARCH_STREAM_EVENT_PAGE);
    let limit: u32 = search_stream_event_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_EVENT_LIMIT);
    let search_event = stream_models::SearchStreamEvent::convert(search_stream_event_dto, profile.user_id);

    if search_event.user_id != profile.user_id && profile.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", profile.user_id, search_event.user_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_EVENTS, &text);
        log::error!("{}: {}", err::CD_FORBIDDEN, &message);
        return Err(AppError::forbidden403(&message)); // 403
    }
    
    let res_data = web::block(move || {
        // Find for an entity (stream event) by SearchStreamEvent.
        let res_data =
            stream_orm.find_stream_events(search_event).map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_data
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string())
        })?;

    let (count, streams) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let list = streams.into_iter().map(|v| stream_models::StreamEventDto::from(v)).collect();

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = StreamEventPageDto { list, limit, count, page, pages };

    Ok(HttpResponse::Ok().json(result)) // 200
}

/// get_streams_period
/// ?
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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(&format!("{}: {}: {}", err::MSG_ACCESS_DENIED, 
                MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, "curr_user_id: 1, user_id: 2")))),
        (status = 406, description = "The finish date is less than the start date.", body = AppError,
            example = json!(AppError::not_acceptable406(MSG_FINISH_LESS_START).add_param(Cow::Borrowed("invalidPeriod"), &serde_json::json!(
                { "streamPeriodStart": "2030-03-02T08:00:00.000Z", "streamPeriodFinish": "2030-03-01T08:00:00.000Z" })))),
        (status = 413, description = "The finish date of the search period exceeds the limit.", body = AppError,
            example = json!(AppError::content_large413(MSG_FINISH_EXCEEDS_LIMIT).add_param(Cow::Borrowed("periodTooLong"), &serde_json::json!(
                { "actualPeriodFinish": "2030-04-01T08:00:00.000Z", "maxPeriodFinish": "2030-03-10T08:00:00.000Z" 
                , "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS })))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams_period", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_period(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamPeriodDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();

    // Get search parameters.
    let search_period_dto: SearchStreamPeriodDto = query_params.into_inner();

    let search_period = stream_models::SearchStreamPeriod::convert(search_period_dto, profile.user_id);
    let start = search_period.start.clone();
    let finish = search_period.finish.clone();

    if search_period.user_id != profile.user_id && profile.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", profile.user_id, search_period.user_id);
        #[rustfmt::skip]
        let message = format!("{}: {}: {}", err::MSG_ACCESS_DENIED, MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, &text);
        log::error!("{}: {}", err::CD_FORBIDDEN, &message);
        return Err(AppError::forbidden403(&message)); // 403
    }
    if finish < start {
        let json = serde_json::json!({ "streamPeriodStart": start.to_rfc3339_opts(Millis, true)
            , "streamPeriodFinish": finish.to_rfc3339_opts(Millis, true) });
        log::error!("{}: {}: {}", err::CD_NOT_ACCEPTABLE, MSG_FINISH_LESS_START, json.to_string());
        return Err(AppError::not_acceptable406(MSG_FINISH_LESS_START) // 406
            .add_param(Cow::Borrowed("invalidPeriod"), &json));
    }
    let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
    if max_finish <= finish {
        let json = serde_json::json!({ "actualPeriodFinish": finish.to_rfc3339_opts(Millis, true)
            , "maxPeriodFinish": max_finish.to_rfc3339_opts(Millis, true), "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        log::error!("{}: {}: {}", err::CD_CONTENT_TOO_LARGE, MSG_FINISH_EXCEEDS_LIMIT, json.to_string());
        return Err(AppError::content_large413(MSG_FINISH_EXCEEDS_LIMIT) // 413
            .add_param(Cow::Borrowed("periodTooLong"), &json));
    }

    let res_data = web::block(move || {
        // Find for an entity (stream period) by SearchStreamEvent.
        let res_data =
            stream_orm.find_streams_period(search_period).map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)    
            });
        res_data
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string())
        })?;

    let list: Vec<String> = match res_data {
        Ok(v) => v.iter().map(|d| d.to_rfc3339_opts(Millis, true)).collect(),
        Err(e) => return Err(e)
    };

    Ok(HttpResponse::Ok().json(list)) // 200

}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};

    use crate::extractors::authentication::BEARER;

    use crate::profiles::{profile_models::Profile, profile_orm::tests::ProfileOrmApp};
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        config_strm,
        stream_models::{Stream, StreamEventDto, StreamEventPageDto, StreamInfoDto},
        stream_orm::tests::{StreamOrmApp, STREAM_ID},
    };
    use crate::users::user_models::UserRole;

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    fn create_profile() -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);
        profile
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
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
    #[rustfmt::skip]
    fn get_cfg_data() -> ((config_jwt::ConfigJwt, config_strm::ConfigStrm), (Vec<Profile>, Vec<Session>, Vec<StreamInfoDto>), String) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile());
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(profile1.user_id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(profile1.user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // The "stream" value will be required for the "put_stream" method.
        let stream = create_stream(0, profile1.user_id, "title0", "tag01,tag02", Utc::now());

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let config_strm = config_strm::get_test_config();
        let cfg_c = (config_jwt, config_strm);
        let data_c = (vec![profile1], vec![session1], vec![stream_dto]);
        (cfg_c, data_c, token)
    }
    fn configure_stream(
        cfg_c: (config_jwt::ConfigJwt, config_strm::ConfigStrm),
        data_c: (Vec<Profile>, Vec<Session>, Vec<StreamInfoDto>),
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(cfg_c.0);
            let data_config_strm = web::Data::new(cfg_c.1);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_stream_orm = web::Data::new(StreamOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_strm))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_stream_orm));
        }
    }

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
        assert_eq!(resp.status(), http::StatusCode::RANGE_NOT_SATISFIABLE); // 416

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
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
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let tomorrow = Utc::now() + Duration::days(1);
        let yesterday = Utc::now() - Duration::days(1);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", yesterday));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", yesterday));
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", Utc::now()));
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", tomorrow));
        let stream_orm = StreamOrmApp::create(&streams);
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
            .uri(&format!("/api/streams?isFuture={}&page={}&limit={}", true, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let tomorrow = Utc::now() + Duration::days(1);
        let yesterday = Utc::now() - Duration::days(1);
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now()));
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", tomorrow));
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", yesterday));
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", yesterday));
        let stream_orm = StreamOrmApp::create(&streams);
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
            .uri(&format!("/api/streams?isFuture={}&page={}&limit={}", false, page, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        #[rustfmt::skip]
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now() + Duration::days(2)));
        #[rustfmt::skip]
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now() + Duration::days(1)));
        #[rustfmt::skip]
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", Utc::now() - Duration::days(1)));
        #[rustfmt::skip]
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", Utc::now() - Duration::days(2)));
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let mut streams: Vec<StreamInfoDto> = Vec::new();
        #[rustfmt::skip]
        streams.push(create_stream(0, profile1_id, "demo11", "tag11,tag12", Utc::now() - Duration::days(2)));
        #[rustfmt::skip]
        streams.push(create_stream(1, profile1_id, "demo12", "tag14,tag15", Utc::now() - Duration::days(1)));
        #[rustfmt::skip]
        streams.push(create_stream(2, profile1_id, "demo21", "tag21,tag22", Utc::now() + Duration::days(1)));
        #[rustfmt::skip]
        streams.push(create_stream(3, profile1_id, "demo22", "tag24,tag25", Utc::now() + Duration::days(2)));
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

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
        let starttime = to_utc(today).to_rfc3339_opts(Millis, true);
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        let start_s = to_utc(start).to_rfc3339_opts(Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile1_id, start_s, finish_s))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

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
        let start_s = to_utc(start).to_rfc3339_opts(Millis, true);
        let finish_s = to_utc(finish).to_rfc3339_opts(Millis, true);
        let max_finish_s = to_utc(max_finish).to_rfc3339_opts(Millis, true);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_streams_period).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/streams_period?userId={}&start={}&finish={}", profile1_id, start_s, finish_s))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::PAYLOAD_TOO_LARGE); // 413

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
        let start = to_utc(d2).to_rfc3339_opts(Millis, true);
        let finish = to_utc(d3).to_rfc3339_opts(Millis, true);

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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

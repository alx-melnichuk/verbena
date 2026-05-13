use std::{borrow::Cow, fs, ops::Deref, path};

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{HttpResponse, delete, get, post, put, web};
use chrono::{DateTime, Duration, SecondsFormat::Millis, Utc};
use log::error;
use mime::IMAGE;
use serde_json::{self, json};
use utoipa;
use vrb_authent::authentication::{Authenticated, RequireAuth};
use vrb_common::{
    alias_path::alias_path_stream,
    api_error::ApiError,
    err, parser,
    validators::{self, ValidationChecks, Validator, msg_validation},
};
use vrb_dbase::{enm_stream_state::StreamState, enm_user_role::UserRole};
use vrb_tools::{cdis::coding, loading::dynamic_image};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::stream_orm::impls::StreamOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::stream_orm::tests::StreamOrmApp;

use crate::{
    config_strm::{self, ConfigStrm},
    stream_models::{
        CreateStreamAndTags, CreateStreamAndTagsDto, FilterStream, ModifyStreamAndTags, ModifyStreamAndTagsDto, PageStreamAndTagsDto,
        PageStreamTagDto, SEARCH_STREAM_AND_TAGS_LIMIT, SEARCH_STREAM_AND_TAGS_LIMIT_MAX, SEARCH_STREAM_AND_TAGS_LIMIT_MIN,
        SEARCH_STREAM_AND_TAGS_PAGE, SEARCH_STREAM_TAGS_LIMIT, SEARCH_STREAM_TAGS_LIMIT_MAX, SEARCH_STREAM_TAGS_LIMIT_MIN,
        SEARCH_STREAM_TAGS_PAGE, SearchStreamAndTags, SearchStreamAndTagsDto, SearchStreamDate, SearchStreamDateDto, SearchStreamTag,
        SearchStreamTagDto, StreamAndTagsDto, StreamConfigDto, StreamTagDto, ToggleStreamStateDto,
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
// 406 Not acceptable - Error deserializing field tag. // Use: post_stream_and_tags, put_stream_and_tags
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
            .service(get_stream_and_tags_by_id)
            // GET /api/streams
            .service(get_stream_and_tags)
            // GET /api/streams_config
            .service(get_stream_config)
            // GET /api/streams_calendar
            .service(get_streams_calendar)
            // GET /api/streams_popural_tags
            .service(get_stream_popural_tags)
            // POST /api/streams
            .service(post_stream_and_tags)
            // PUT /api/streams/toggle/{id}
            .service(put_toggle_state)
            // PUT /api/streams/{id}
            .service(put_stream_and_tags)
            // DELETE /api/streams/{id}
            .service(delete_stream_and_tags);
    }
}

pub fn get_file_name(user_id: i32, date_time: DateTime<Utc>) -> String {
    format!("{}_{}", user_id, coding::encode(date_time, 1))
}

// ** Section: Stream Get **

/// get_stream_and_tags_by_id
///
/// Search for a stream by his ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams/1
/// ```
///
/// Return the found specified stream (`StreamAndTagsDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "A stream with the specified ID was found.", body = StreamAndTagsDto),
        (status = 204, description = "The stream with the specified ID was not found."),
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
)]
// Used to get information about a stream in chat without authorization.
#[get("/api/streams/{id}")]
pub async fn get_stream_and_tags_by_id(
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}.{}", 416, &message);
        ApiError::new(416, &message) // 416
    })?;

    let res_data = web::block(move || {
        // Get 'stream' by id.
        let res_data = stream_orm.get_stream_and_tags(id).map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_and_tags = res_data?;
    if let Some(stream_and_tags) = opt_stream_and_tags {
        let stream_and_tags_dto = StreamAndTagsDto::from(stream_and_tags);
        Ok(HttpResponse::Ok().json(stream_and_tags_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// get_stream_and_tags
///
/// Get a list of your streams (page by page).
///
/// Request structure:
/// ```text
/// {
///   userId?: number,                      // optional
///   live?: boolean,                       // optional
///   filter?: "future" | "past" | "period" // optional 
///   starttime?: DateTime<Utc>,            // optional
///   finishtime?: DateTime<Utc>,           // optional
///   sortDesc?: boolean,                   // optional
///   tag?: String,                         // optional
///   page?: number,                        // optional
///   limit?: number,                       // optional
/// }
/// Where:
/// "userId" - user identifier;
/// "live" - sign of a "live" stream ("state" = ["preparing", "started", "paused"]);
/// "filter" - defines data filtering:
///   "future" - Get future streams with a "starttime" greater than or equal 
///              to the one specified in the "starttime" field (in UTC format).
///   "past"   - Get past streams with a "starttime" less than the one 
///              specified in the "starttime" field (in UTC format).
///              To sort in descending order, use sortDesc=true.
///   "period" - Get streams with a "starttime" greater than or equal to 
///              the one specified in the "starttime" field.
///              And less than or equal to the one specified in the "finishtime"
///              field. The "starttime" and "finishtime" values ​​are specified in
///              UTC format.
/// "starttime"  - Date value (in UTC format). Used in conjunction with the "filter"
///                field. If not specified, the current date and time are used.
/// "finishtime" - Date value (in UTC format). Used in conjunction with the "filter" field.
/// 
/// "sortDesc"   - Descending sorting flag.
///                Takes the following values:
///   true              - Sort by the "starttime" field in descending order;
///   false (undefined) - Sort by the "starttime" field in ascending order;
/// 
/// "tag"   - Get streams that have the specified tag.
/// "page"  - page number, stratified from 1 (1 by default);
/// "limit" - number of records on the page (5 by default);
/// ```
/// It is recommended to enter the date and time in ISO8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// 
/// One could call with following curl.
/// Get streams with "live" true.
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&live=true' -H 'Content-Type: application/json'
/// ```
/// Get future streams (from current date and time).
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&filter=future' -H 'Content-Type: application/json'
/// ```
/// Get past streams (from current date and time).
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&filter=past' -H 'Content-Type: application/json'
/// ```
/// Get past streams (from the specified date and time).
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&filter=past&starttime=2020-02-02T20:10:00.000Z' \
///  -H 'Content-Type: application/json'
/// ```
/// Get streams for the specified period.
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&filter=period&starttime=2020-02-02T20:10:00.000Z \
/// &finishtime=2020-02-12T20:10:00.000Z' -H 'Content-Type: application/json'
/// ```
/// Get streams that have the tag "tag02".
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?tag=tag02' -H 'Content-Type: application/json'
/// ```
/// Get the first page of streams.
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&page=1&limit=5' -H 'Content-Type: application/json'
/// ```
/// Get the second page of streams.
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams?userId=1&page=2&limit=5' -H 'Content-Type: application/json'
/// ```
/// 
/// Response structure PageStreamAndTagsDto:
/// ```text
/// {
///   list: [StreamAndTagsDto],
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
/// StreamAndTagsDto structure:
/// ```text
/// {
///   id: number,
///   userId: number,
///   title: string,
///   descript: string,
///   logo?: string,
///   starttime: DateTime<Utc>,
///   live: boolean,
///   state: "" | "",
///   started?: DateTime<Utc>,
///   paused?: DateTime<Utc>,
///   stopped?: DateTime<Utc>,
///   source: string,
///   createdAt: DateTime<Utc>,
///   updatedAt: DateTime<Utc>,
///   tags: [string],
/// }
/// Where:
/// "id"        - record ID;
/// "userId"    - user identifier;
/// "title"     - stream title;
/// "descript"  - stream descript;
/// "logo"      - stream logo;
/// "starttime" - stream start date;
/// "live"      - a sign that the stream is "live";
///  // "state": "preparing" | "started" | "paused"
/// "state": "waiting" | "preparing" | "started" | "paused" | "topped";
/// "started"   - the date the stream started;
/// "paused"    - the date the stream was paused;
/// "stopped"   - the date the stream was stopped;
/// "source"    - stream source;
/// "createdAt" - date of stream creation;
/// "updatedAt" - date of stream modification; 
/// "tags"      - list of stream tags;
/// ```
/// 
/// Return found data on streams (`PageStreamAndTagsDto`) with status 200.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Result of the stream request.", body = PageStreamAndTagsDto),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 406, body = ApiError,
            description = "The finish date is less than the start date. \
            Validation error. `curl -X GET 'http://localhost:8080/api/streams?filter=period \
            &starttime=2030-03-02T08:00:00.000Z&finishtime=2030-03-01T08:00:00.000Z`",
            example = json!(ApiError::new(406, MSG_FINISH_LESS_START).add_param(Cow::Borrowed("invalidPeriod"), &serde_json::json!(
                { "streamPeriodStart": "2030-03-02T08:00:00.000Z", "streamPeriodFinish": "2030-03-01T08:00:00.000Z" })) )),
        (status = 413, body = ApiError,
            description = "The finish date of the search period exceeds the limit. \
            Validation error. `curl -X GET 'http://localhost:8080/api/streams?filter=period \
            &starttime=2030-01-01T08:00:00.000Z&finishtime=2030-04-01T08:00:00.000Z`",
            example = json!(ApiError::new(413, MSG_FINISH_EXCEEDS_LIMIT).add_param(Cow::Borrowed("periodTooLong"), 
            &serde_json::json!({ "actualPeriodFinish": "2030-04-01T08:00:00.000Z", "maxPeriodFinish": "2030-03-07T08:00:00.000Z" 
                , "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS })))),
        (status = 417, body = [ApiError], 
            description = "Validation error. `curl -X GET 'http://localhost:8080/api/streams?filter=period`",
            example = json!(ApiError::validations(
                (SearchStreamAndTagsDto::from(SearchStreamAndTags::new(None, Some(FilterStream::Period), None, None)))
                    .validate().err().unwrap()) )),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_and_tags(
    stream_orm: web::Data<StreamOrmApp>, 
    query_params: web::Query<SearchStreamAndTagsDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get search parameters.
    let search_stream_dto: SearchStreamAndTagsDto = query_params.into_inner();

    // Checking the validity of the data model.
    let validation_res = search_stream_dto.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}.{}", 417, msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let opt_filter = search_stream_dto.filter.clone();
    let opt_starttime  = search_stream_dto.starttime.clone();
    let opt_finishtime  = search_stream_dto.finishtime.clone();

    if Some(FilterStream::Period) == opt_filter && opt_starttime.is_some() && opt_finishtime.is_some() {
        let start = opt_starttime.unwrap();
        let finish = opt_finishtime.unwrap();
        if start > finish {
            let json = serde_json::json!({ "streamPeriodStart": start.to_rfc3339_opts(Millis, true)
                , "streamPeriodFinish": finish.to_rfc3339_opts(Millis, true) });
            error!("{}.{}; {}", 406, MSG_FINISH_LESS_START, json.to_string());
            return Err(ApiError::new(406, MSG_FINISH_LESS_START) // 406
                .add_param(Cow::Borrowed("invalidPeriod"), &json));
        }
        let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
        if max_finish <= finish {
            let json = serde_json::json!({ "actualPeriodFinish": finish.to_rfc3339_opts(Millis, true)
                , "maxPeriodFinish": max_finish.to_rfc3339_opts(Millis, true), "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
            error!("{}.{}; {}", 413, MSG_FINISH_EXCEEDS_LIMIT, json.to_string());
            return Err(ApiError::new(413, MSG_FINISH_EXCEEDS_LIMIT) // 413
                .add_param(Cow::Borrowed("periodTooLong"), &json));
        } 
    }
    
    let page: u32 = search_stream_dto.page.unwrap_or(SEARCH_STREAM_AND_TAGS_PAGE);
    let limit: u32 = search_stream_dto.limit.unwrap_or(SEARCH_STREAM_AND_TAGS_LIMIT);

    let mut search_stream: SearchStreamAndTags = search_stream_dto.into();
    search_stream.page = Some(page);
    let limit = if limit > SEARCH_STREAM_AND_TAGS_LIMIT_MIN { limit } else { SEARCH_STREAM_AND_TAGS_LIMIT_MIN };
    let limit = if limit <= SEARCH_STREAM_AND_TAGS_LIMIT_MAX { limit } else { SEARCH_STREAM_AND_TAGS_LIMIT_MAX };
    search_stream.limit = Some(limit);

    let res_data = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_data =
            stream_orm.filter_stream_and_tags_by_pages(search_stream).map_err(|e| {
                error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;
        
    let (count, stream_and_tags) = res_data?;

    let list: Vec<StreamAndTagsDto> = stream_and_tags.into_iter().map(|v| v.into()).collect();
    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };
    let result = PageStreamAndTagsDto { list, limit, count, page, pages };

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

/// get_streams_calendar
///
/// Get a list of dates for the specified period that have streams.
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
/// curl -i -X GET http://localhost:8080/api/streams_calendar? \
///     start=2030-03-01T08:00:00.000Z&finish=2030-03-31T08:00:00.000Z
/// ```
/// Could be called with all fields with the next curl. 
/// The "userId" parameter can be specified if the user has the "Admin" role.
/// ```text
/// curl -i -X GET http://localhost:8080/api/streams_calendar?userId=1 \
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
#[get("/api/streams_calendar", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams_calendar(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamDateDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let curr_user_id = user.id;

    // Get search parameters.
    let search_stream_date_dto: SearchStreamDateDto = query_params.into_inner();

    let start = search_stream_date_dto.start.clone();
    let finish = search_stream_date_dto.finish.clone();
    let user_id = search_stream_date_dto.user_id.unwrap_or(curr_user_id);

    if user_id != curr_user_id && user.role != UserRole::Admin {
        let text = format!("curr_user_id: {}, user_id: {}", curr_user_id, user_id);
        let message = format!("{}; {}", MSG_GET_LIST_OTHER_USER_STREAMS_PERIOD, &text);
        error!("{}.{}", 403, &message);
        return Err(ApiError::create(403, err::MSG_ACCESS_DENIED, &message)); // 403
    }
    if finish < start {
        let json = serde_json::json!({ "streamPeriodStart": start.to_rfc3339_opts(Millis, true)
            , "streamPeriodFinish": finish.to_rfc3339_opts(Millis, true) });
        error!("{}.{}; {}", 406, MSG_FINISH_LESS_START, json.to_string());
        return Err(ApiError::new(406, MSG_FINISH_LESS_START) // 406
            .add_param(Cow::Borrowed("invalidPeriod"), &json));
    }
    let max_finish = start + Duration::days(PERIOD_MAX_NUMBER_DAYS.into());
    if max_finish <= finish {
        let json = serde_json::json!({ "actualPeriodFinish": finish.to_rfc3339_opts(Millis, true)
            , "maxPeriodFinish": max_finish.to_rfc3339_opts(Millis, true), "periodMaxNumberDays": PERIOD_MAX_NUMBER_DAYS });
        error!("{}.{}; {}", 413, MSG_FINISH_EXCEEDS_LIMIT, json.to_string());
        return Err(ApiError::new(413, MSG_FINISH_EXCEEDS_LIMIT) // 413
            .add_param(Cow::Borrowed("periodTooLong"), &json));
    }

    let mut search_stream_date: SearchStreamDate = search_stream_date_dto.into();
    search_stream_date.user_id = user_id;

    let res_data = web::block(move || {
        // Find for an entity (stream period) by SearchStreamEvent.
        let res_data =
        stream_orm.filter_stream_dates(search_stream_date)
        .map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)    
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let list: Vec<String> = match res_data {
        Ok(v) => v.iter().map(|d| d.to_rfc3339_opts(Millis, true)).collect(),
        Err(e) => return Err(e)
    };

    Ok(HttpResponse::Ok().json(list)) // 200
}

/// get_stream_popural_tags
/// 
/// Get a list of popular tags.
/// 
/// Request structure:
/// ```text
/// {
///   sortColumn?: "id" | "name" | "countlinks" // optional
///   sortDesc?: boolean,                       // optional
///   page?: number,                            // optional
///   limit?: number,                           // optional
/// }
/// Where:
/// "sortColumn" - Name of the sorting field:
///   "id"         - Record ID (default).
///   "name"       - Tag value.
///   "countlinks" - Number of links to the tag.
/// "sortDesc"   - Descending sorting flag.
///                Takes the following values:
///   true              - Sort by the "starttime" field in descending order;
///   false (undefined) - Sort by the "starttime" field in ascending order (default);
/// "page"  - page number, stratified from 1 (1 by default);
/// "limit" - number of records on the page (5 by default);
/// ```
/// One could call with following curl.
/// Получить список тегов (сортировка по полю "" по возрастанию).
/// ```text
/// curl -i -X GET 'http://localhost:8080/api/streams_popural_tags' -H 'Content-Type: application/json'
/// ```
/// 
/// Response structure PageStreamTagDto:
/// ```text
/// {
///   list: [StreamTagDto],
///   limit: number,
///   page: number,
/// }
/// Where:
/// "list"  - array of tag;
/// "limit" - number of records on the page;
/// "page"  - current page number (stratified from 1);
/// ```
/// StreamTagDto structure:
/// ```text
/// {
///   id: number,
///   name: string,
///   countLinks: number,
/// }
/// Where:
/// "id"   - record ID;
/// "name" - tag value;
/// "countLinks"  - number of links to the tag;
/// ```
/// 
/// Return found data on tags (`PageStreamTagDto`) with status 200.
/// 
#[rustfmt::skip]
#[get("/api/streams_popural_tags", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_popural_tags(
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<SearchStreamTagDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get search parameters.
    let search_stream_tag_dto: SearchStreamTagDto = query_params.into_inner();

    let page: u32 = search_stream_tag_dto.page.unwrap_or(SEARCH_STREAM_TAGS_PAGE);
    let limit: u32 = search_stream_tag_dto.limit.unwrap_or(SEARCH_STREAM_TAGS_LIMIT);

    let mut search_stream_tag = SearchStreamTag::from(search_stream_tag_dto);
    search_stream_tag.page = Some(page);
    let limit = if limit > SEARCH_STREAM_TAGS_LIMIT_MIN { limit } else { SEARCH_STREAM_TAGS_LIMIT_MIN };
    let limit = if limit <= SEARCH_STREAM_TAGS_LIMIT_MAX { limit } else { SEARCH_STREAM_TAGS_LIMIT_MAX };
    search_stream_tag.limit = Some(limit);

    let res_data = web::block(move || {
        // Find for an entity (stream period) by SearchStreamEvent.
        let res_data =
        stream_orm.get_stream_tags(search_stream_tag).map_err(|e| {
                error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)    
            });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let (limit, page, stream_tags) = res_data?;

    let list: Vec<StreamTagDto> = stream_tags.into_iter().map(|v| v.into()).collect();

    let result = PageStreamTagDto { list, limit, page };

    Ok(HttpResponse::Ok().json(result)) // 200
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

fn new_stream_dto(title: &str, descript: &str, starttime: &str, tag_list: &str) -> CreateStreamAndTagsDto {
    #[rustfmt::skip]
    let descript = if descript.len() > 0 { Some(descript.to_string()) } else { None };
    // let starttime1 = if starttime.len() > 0 { Some(DateTime::parse_from_rfc3339(starttime).unwrap().with_timezone(&Utc)) } else {None};

    let start = if starttime.len() > 0 { Some(starttime) } else { None };
    let starttime = start.map(|val| DateTime::parse_from_rfc3339(val).unwrap().with_timezone(&Utc));

    let tags: Vec<String> = tag_list.split(',').map(|val| val.to_string()).collect();

    CreateStreamAndTagsDto {
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
    pub fn convert(create_stream_form: CreateStreamForm) -> Result<(CreateStreamAndTagsDto, Option<TempFile>), String> {
        let val = create_stream_form.tags.into_inner();
        let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
        if let Err(err) = res_tags {
            return Err(err.to_string());
        }
        let tags: Vec<String> = res_tags.unwrap();

        Ok((
            CreateStreamAndTagsDto {
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

/// post_stream_and_tags
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
/// Return a new stream (`StreamAndTagsDto`) with status 201.
/// 
#[utoipa::path(
    responses(
        (status = 201, description = "Create a new stream.", body = StreamAndTagsDto,
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
pub async fn post_stream_and_tags(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    MultipartForm(create_stream_form): MultipartForm<CreateStreamForm>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let curr_user_id = user.id;

    // Get data from MultipartForm.
    let (create_stream_and_tags_dto, logo_file) = CreateStreamForm::convert(create_stream_form)
        .map_err(|e| {
            error!("{}.{}; {}", 406, MSG_INVALID_FIELD_TAG, &e);
            ApiError::create(406, MSG_INVALID_FIELD_TAG, &e) // 406
        })?;

    // Checking the validity of the data model.
    let validation_res = create_stream_and_tags_dto.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}.{}", 417, msg_validation(&validation_errors));
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
            error!("{}.{}; {}", 413, err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(ApiError::new(413, err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }
        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types = config_strm.strm_logo_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            error!("{}.{}; {}", 415, err::MSG_INVALID_FILE_TYPE, json.to_string());
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
            error!("{}.{}; {}", 500, err::MSG_ERROR_UPLOAD_FILE, &msg);
            return Err(ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, &msg)) // 500
        }
        path_new_logo_file = full_path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "post_stream_and_tags()")
            .map_err(|e| {
                error!("{}.{}; {}", 510, err::MSG_ERROR_CONVERT_FILE, &e);
                ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, &e) // 510
            })?;
        if let Some(new_path_file) = res_convert_logo_file {
            path_new_logo_file = new_path_file;
        }
        
        break;
    }

    let mut create_stream_and_tags: CreateStreamAndTags = create_stream_and_tags_dto.into();
    create_stream_and_tags.user_id = curr_user_id;
    
    let alias_path_strm = alias_path_stream::AliasStrm::new(&config_strm.strm_logo_files_dir);
    let alias_strm = alias_path_strm.as_ref();

    if path_new_logo_file.len() > 0 {
        // Replace file path prefix with alias.
        let alias_logo_file= alias_strm.path_to_alias(&path_new_logo_file);
        create_stream_and_tags.logo = Some(alias_logo_file);
    }

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data = stream_orm.create_stream_and_tags(create_stream_and_tags).map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    if res_data.is_err() && path_new_logo_file.len() > 0 {
        if let Err(err) = fs::remove_file(&path_new_logo_file) {
            error!("{} remove_file({}): error: {:?}", "post_stream_and_tags()", &path_new_logo_file, err);
        }
    }
    let opt_stream_and_tags = res_data?;
    if let Some(stream_and_tags) = opt_stream_and_tags {
        let stream_and_tags_dto = StreamAndTagsDto::from(stream_and_tags);
        Ok(HttpResponse::Created().json(stream_and_tags_dto)) // 201
    } else {
        Ok(HttpResponse::Created().finish()) // 201
    }
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
    pub fn convert(modify_stream_form: ModifyStreamForm) -> Result<(ModifyStreamAndTagsDto, Option<TempFile>), String> {
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
            ModifyStreamAndTagsDto {
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

/// put_stream_and_tags
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
/// Return the stream with updated data (`StreamAndTagsDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Update the stream with new data.", body = StreamAndTagsDto),
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
                (ModifyStreamAndTagsDto {
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
    params(("id", description = "Unique stream ID.")),
    security(("bearer_auth" = [])),
)]
// PUT /api/streams/{id}
#[rustfmt::skip]
#[put("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_stream_and_tags(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    MultipartForm(modify_stream_form): MultipartForm<ModifyStreamForm>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let curr_user_id = user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}.{}", 416, &message);
        ApiError::new(416, &message) // 416
    })?;

    // Get data from MultipartForm.
    let (modify_stream_and_tags_dto, logo_file) = ModifyStreamForm::convert(modify_stream_form)
    .map_err(|e| {
        error!("{}.{}; {}", 406, MSG_INVALID_FIELD_TAG, &e);
        ApiError::create(406, MSG_INVALID_FIELD_TAG, &e) // 406
    })?;

    // If there is not a single field in the MultipartForm, it gives an error 400 "Multipart stream is incomplete".

    // Checking the validity of the data model.
    let validation_res = modify_stream_and_tags_dto.validate();
    if let Err(validation_errors) = validation_res {
        let mut is_no_fields_to_update = false;
        let errors = validation_errors.iter().map(|err| {
            if !is_no_fields_to_update && err.params.contains_key(validators::NM_NO_FIELDS_TO_UPDATE) {
                is_no_fields_to_update = true;
                let valid_names = [ModifyStreamAndTagsDto::valid_names(), vec!["logofile"]].concat().join(",");
                ValidationChecks::no_fields_to_update(&[false], &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err().unwrap()
            } else {
                err.clone()
            }
        }).collect();
        if !is_no_fields_to_update || logo_file.is_none() {
            error!("{}.{}", 417, msg_validation(&errors));
            return Ok(ApiError::to_response(&ApiError::validations(errors))); // 417
        }
    }

    let mut logo: Option<String> = None;
    let config_strm = config_strm.get_ref().clone();
    let mut path_new_logo_file = "".to_string();

    while let Some(temp_file) = logo_file {
        // Delete the old version of the logo file.
        if temp_file.size == 0 {
            logo = Some("".to_string()); // Set the "logo" field to ``.
            break;
        }
        let logo_max_size = usize::try_from(config_strm.strm_logo_max_size).unwrap();
        // Check file size for maximum value.
        if logo_max_size > 0 && temp_file.size > logo_max_size {
            let json = json!({ "actualFileSize": temp_file.size, "maxFileSize": logo_max_size });
            error!("{}.{}; {}", 413, err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(ApiError::new(413, err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }

        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types: Vec<String> = config_strm.strm_logo_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            error!("{}.{}; {}", 415, err::MSG_INVALID_FILE_TYPE, json.to_string());
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
            error!("{}.{}", 500, &message);
            return Err(ApiError::new(500, &message)); // 500
        }
        path_new_logo_file = full_path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "put_stream_and_tags()")
            .map_err(|e| {
                error!("{}.{}; {}", 510, err::MSG_ERROR_CONVERT_FILE, &e);
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
        let alias_logo_file = alias_strm.path_to_alias(&path_new_logo_file);
        logo = Some(alias_logo_file);
    }

    let mut modify_stream_and_tags: ModifyStreamAndTags = modify_stream_and_tags_dto.into();
    modify_stream_and_tags.logo = logo.clone();

    let res_stream_and_tags = web::block(move || {
        // Modify an entity (stream).
        let res_data = stream_orm.modify_stream_and_tags(id, curr_user_id, modify_stream_and_tags)
        .map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;
    // modify_stream_and_stream_tag() time: 7.46ms

    let opt_stream_and_tags = res_stream_and_tags
    .map_err(|err| {
        if path_new_logo_file.len() > 0 {
            if let Err(err) = fs::remove_file(&path_new_logo_file) {
                error!("put_stream_and_tags() remove_file({}): error: {:?}", &path_new_logo_file, err);
            }
        }
        err
    })?;

    if let Some(stream_and_tags) = opt_stream_and_tags {
        let path_old_logo_file: String = stream_and_tags.old_logo.clone().unwrap_or_default();

        // If the name of the new logo file is defined, then delete the old logo file.
        // If the file path starts with alice, then the file corresponds to the entity type.
        // And only then can the file be deleted.
        if logo.is_some() && alias_strm.starts_with_alias(&path_old_logo_file) {
            // Return file path prefix instead of alias.
            let full_path_file_img = alias_strm.alias_to_path(&path_old_logo_file);
            if let Err(err) = fs::remove_file(&full_path_file_img) {
                error!("put_stream_and_tags() remove_file({}): error: {:?}", &full_path_file_img, err);
            }
        }

        let stream_and_tags_dto: StreamAndTagsDto = stream_and_tags.into();
        Ok(HttpResponse::Ok().json(stream_and_tags_dto)) // 200
    } else {
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
        (status = 200, description = "Update the stream with new data.", body = StreamAndTagsDto),
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
    params(("id", description = "Unique stream ID.")),
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
    let curr_user_id = user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}.{}", 416, &message);
        ApiError::new(416, &message) // 416
    })?;

    let new_state: StreamState = json_body.into_inner().state;
    let stream_orm3 = stream_orm.clone();
    let res_data = web::block(move || {
        // Find a stream by ID.
        let res_data = stream_orm3.get_stream_and_tags(id)
            .map_err(|e| {
                error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
            res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_and_tags = res_data?;

    if opt_stream_and_tags.is_none() {
        // If a stream with the specified ID is not found for the current user, then return status 204.
        return Ok(HttpResponse::NoContent().finish()) // 204
    }
    let stream_and_tags = opt_stream_and_tags.unwrap();

    if stream_and_tags.state == new_state {
        let json = json!({ "oldState": &stream_and_tags.state, "newState": &new_state });
        error!("{}.{}; {}", 406, MSG_INVALID_STREAM_STATE, json.to_string());
        return Err(ApiError::new(406, MSG_INVALID_STREAM_STATE) // 406
            .add_param(Cow::Borrowed("invalidState"), &json));
    }

    let is_not_acceptable = match new_state {
        StreamState::Preparing => vec![StreamState::Started, StreamState::Paused].contains(&stream_and_tags.state),
        StreamState::Started => vec![StreamState::Waiting, StreamState::Stopped].contains(&stream_and_tags.state),
        StreamState::Paused => vec![StreamState::Waiting, StreamState::Stopped, StreamState::Preparing].contains(&stream_and_tags.state),
        StreamState::Stopped => vec![StreamState::Waiting].contains(&stream_and_tags.state),
        _ => false,
    };
    if is_not_acceptable {
        let json = json!({ "oldState": &stream_and_tags.state.to_string(), "newState": &new_state });
        error!("{}.{}; {}", 406, MSG_INVALID_STREAM_STATE, json.to_string());
        return Err(ApiError::new(406, MSG_INVALID_STREAM_STATE) // 406
            .add_param(Cow::Borrowed("invalidState"), &json));
    }
    // If the stream goes into active state, then
    if vec![StreamState::Preparing, StreamState::Started, StreamState::Paused].contains(&new_state) {
        let stream_orm3 = stream_orm.clone();
        // find any stream in active state.
        let res_data2 = web::block(move || {
            let res_data2 = stream_orm3
                .get_stream_and_tags_in_live(curr_user_id, id)
                .map_err(|e| {
                    error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
                    ApiError::create(507, err::MSG_DATABASE, &e)
                });
            res_data2
        })
        .await
        .map_err(|e| {
            error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;
        
        let opt_stream2_and_tags = res_data2?;

        if let Some(stream2_and_tags) = opt_stream2_and_tags {
            let json = json!({ "id": stream2_and_tags.id, "title": &stream2_and_tags.title });
            error!("{}.{}; {}", 409, MSG_EXIST_IS_ACTIVE_STREAM, json.to_string());
            return Err(ApiError::new(409, MSG_EXIST_IS_ACTIVE_STREAM) // 409
                .add_param(Cow::Borrowed("activeStream"), &json));
        }
    }

    let modify_stream_and_tags = ModifyStreamAndTags {
        title: None,
        descript: None,
        logo: None,
        starttime: None,
        state: Some(new_state),
        started: None,
        paused: None,
        stopped: None,
        source: None,
        tags: None,
    };

    let res_data3 = web::block(move || {
        // Modify an entity (stream).
        let res_data3 = stream_orm.modify_stream_and_tags(id, curr_user_id, modify_stream_and_tags)
        .map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });
        res_data3
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_and_tags = res_data3?;

    if opt_stream_and_tags.is_none() {
        // If a stream with the specified ID is not found for the current user, then return status 204.
        return Ok(HttpResponse::NoContent().finish()) // 204
    }
    let stream_and_tags = opt_stream_and_tags.unwrap();
    let stream_and_tags_dto = StreamAndTagsDto::from(stream_and_tags);
    Ok(HttpResponse::Ok().json(stream_and_tags_dto)) // 200
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
/// Return the deleted stream (`StreamAndTagsDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified stream was deleted successfully.", body = StreamAndTagsDto),
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
// DELETE /api/streams/{id}
#[rustfmt::skip]
#[delete("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_stream_and_tags(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let curr_user_id = user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = format!("`{}` - {}", "id", &e);
        error!("{}.{}; {}", 416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;

    let res_data = web::block(move || {
        // Delete an entity (stream, tags).
        let res_data = stream_orm.delete_stream_and_tags(id, curr_user_id).map_err(|e| {
            error!("{}.{}; {}", 507, err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}.{}; {}", 506, err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_stream_and_tags = res_data?;

    if let Some(stream_and_tags) = opt_stream_and_tags {
        // Get the path to the "logo" file.
        let path_file_img: String = stream_and_tags.logo.clone().unwrap_or_default();

        let config_strm = config_strm.get_ref().clone();
        let alias_path_strm = alias_path_stream::AliasStrm::new(&config_strm.strm_logo_files_dir);
        let alias_strm = alias_path_strm.as_ref();

        // If the file path starts with alice, then the file corresponds to the entity type.
        // And only then can the file be deleted.
        if alias_strm.starts_with_alias(&path_file_img) {
            // Return file path prefix instead of alias.
            let full_path_file_img = alias_strm.alias_to_path(&path_file_img);
            if let Err(err) = fs::remove_file(&full_path_file_img) {
                error!("delete_stream_and_tags() remove_file({}): error: {:?}", &full_path_file_img, err);
            }
        }
        let stream_and_tags_dto = StreamAndTagsDto::from(stream_and_tags);
        Ok(HttpResponse::Ok().json(stream_and_tags_dto)) // 200
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
    pub fn check_app_err(app_err_vec: Vec<ApiError>, status: u16, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.status, status);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
}

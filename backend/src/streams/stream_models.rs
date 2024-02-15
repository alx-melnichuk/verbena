use std::{borrow, collections::HashMap};

use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::schema;
use crate::utils::{serial_datetime, serial_datetime_option};
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const MSG_TITLE_REQUIRED: &str = "title:required";
pub const TITLE_MIN: u8 = 2;
pub const MSG_TITLE_MIN_LENGTH: &str = "title:min_len";
pub const TITLE_MAX: u16 = 255;
pub const MSG_TITLE_MAX_LENGTH: &str = "title:max_len";

pub const DESCRIPT_MIN: u8 = 2;
pub const MSG_DESCRIPT_MIN_LENGTH: &str = "descript:min_len";
pub const DESCRIPT_MAX: u16 = 2048; // 2*1024
pub const MSG_DESCRIPT_MAX_LENGTH: &str = "descript:max_len";

pub const LOGO_MIN: u8 = 2;
pub const MSG_LOGO_MIN_LENGTH: &str = "logo:min_len";
pub const LOGO_MAX: u16 = 255;
pub const MSG_LOGO_MAX_LENGTH: &str = "logo:max_len";

pub const MSG_MIN_VALID_STARTTIME: &str = "starttime:min_valid_date";

pub const MSG_SOURCE_REQUIRED: &str = "source:required";
pub const SOURCE_MIN: u8 = 2;
pub const MSG_SOURCE_MIN_LENGTH: &str = "source:min_len";
pub const SOURCE_MAX: u16 = 255;
pub const MSG_SOURCE_MAX_LENGTH: &str = "source:max_len";

pub const TAG_MIN_AMOUNT: u8 = 1;
pub const MSG_TAG_MIN_AMOUNT: &str = "tag:min_amount";
pub const TAG_MAX_AMOUNT: u8 = 4;
pub const MSG_TAG_MAX_AMOUNT: &str = "tag:max_amount";
pub const MSG_TAG_REQUIRED: &str = "tag:required";
pub const TAG_MIN: u8 = 2;
pub const MSG_TAG_MIN_LENGTH: &str = "tag:min_len";
pub const TAG_MAX: u16 = 255;
pub const MSG_TAG_MAX_LENGTH: &str = "tag:max_len";

// ** ModifyStreamInfoDto **

pub const MSG_NO_REQUIRED_FIELDS: &str = "Nothing to update! One of the required fields is missing.";

//  ** CreateStreamInfoDto **

pub fn validate_title(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, TITLE_MIN.into(), MSG_TITLE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TITLE_MAX.into(), MSG_TITLE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_descript(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, DESCRIPT_MIN.into(), MSG_DESCRIPT_MIN_LENGTH)?;
    ValidationChecks::max_length(value, DESCRIPT_MAX.into(), MSG_DESCRIPT_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_starttime(value: &DateTime<Utc>) -> Result<(), ValidationError> {
    let min_date_time = Utc::now() + Duration::minutes(1);
    ValidationChecks::min_valid_date(value, &min_date_time, MSG_MIN_VALID_STARTTIME)?;
    Ok(())
}
pub fn validate_source(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_SOURCE_REQUIRED)?;
    ValidationChecks::min_length(value, SOURCE_MIN.into(), MSG_SOURCE_MIN_LENGTH)?;
    ValidationChecks::max_length(&value, SOURCE_MAX.into(), MSG_SOURCE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_tag_name(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, TAG_MIN.into(), MSG_TAG_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TAG_MAX.into(), MSG_TAG_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_tag_amount(tags: &[String]) -> Result<(), ValidationError> {
    let min_amount = TAG_MIN_AMOUNT;
    ValidationChecks::min_amount_of_elem(tags.len(), min_amount.into(), MSG_TAG_MIN_AMOUNT)?;
    let max_amount = TAG_MAX_AMOUNT;
    ValidationChecks::max_amount_of_elem(tags.len(), max_amount.into(), MSG_TAG_MAX_AMOUNT)?;
    Ok(())
}
pub fn validate_tag(tags: &[String]) -> Result<(), ValidationError> {
    validate_tag_amount(tags)?;
    for tag_name in tags {
        validate_tag_name(tag_name)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::StreamState"]
pub enum StreamState {
    Waiting,
    Started,
    Stopped,
    Paused,
    Preparing,
}

impl StreamState {
    pub fn to_str(&self) -> &str {
        match self {
            StreamState::Waiting => "waiting",
            StreamState::Started => "started",
            StreamState::Stopped => "stopped",
            StreamState::Paused => "paused",
            StreamState::Preparing => "preparing",
        }
    }
}

// **  Section: table "streams" receiving data **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stream {
    pub id: i32,
    pub user_id: i32,
    pub title: String,                  // min_len=2 max_len=255
    pub descript: String,               // type: Text default ""
    pub logo: Option<String>,           // min_len=2 max_len=255 Nullable
    pub starttime: DateTime<Utc>,       //
    pub live: bool,                     // default false
    pub state: StreamState,             // default Waiting
    pub started: Option<DateTime<Utc>>, // Nullable
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub status: bool,                   // default true
    pub source: String,                 // min_len=2 max_len=255 default "obs"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(any(test, feature = "mockdata"))]
pub const STREAM_DESCRIPT_DEF: &str = "";
#[cfg(any(test, feature = "mockdata"))]
pub const STREAM_LIVE_DEF: bool = false;
#[cfg(any(test, feature = "mockdata"))]
pub const STREAM_STATE_DEF: StreamState = StreamState::Waiting;
#[cfg(any(test, feature = "mockdata"))]
pub const STREAM_STATUS_DEF: bool = true;
#[cfg(any(test, feature = "mockdata"))]
pub const STREAM_SOURCE_DEF: &str = "obs";

impl Stream {
    #[cfg(test)]
    pub fn new(id: i32, user_id: i32, title: &str, starttime: DateTime<Utc>) -> Stream {
        let now = Utc::now();
        Stream {
            id: id,
            user_id: user_id,
            title: title.to_owned(),
            descript: STREAM_DESCRIPT_DEF.to_string(),
            logo: None,
            starttime: starttime.clone(),
            live: STREAM_LIVE_DEF,
            state: STREAM_STATE_DEF,
            started: None,
            stopped: None,
            status: STREAM_STATUS_DEF,
            source: STREAM_SOURCE_DEF.to_string(),
            created_at: now,
            updated_at: now,
        }
    }
    #[cfg(feature = "mockdata")]
    pub fn create(create_stream: CreateStream, id: i32) -> Stream {
        let now = Utc::now();
        Stream {
            id: id,
            user_id: create_stream.user_id,
            title: create_stream.title.to_owned(),
            descript: create_stream.descript.clone().unwrap_or(STREAM_DESCRIPT_DEF.to_string()),
            logo: create_stream.logo.clone(),
            starttime: create_stream.starttime.clone(),
            live: create_stream.live.unwrap_or(STREAM_LIVE_DEF),
            state: create_stream.state.unwrap_or(STREAM_STATE_DEF),
            started: create_stream.started.clone(),
            stopped: create_stream.stopped.clone(),
            status: create_stream.status.unwrap_or(STREAM_STATUS_DEF),
            source: create_stream.source.clone().unwrap_or(STREAM_SOURCE_DEF.to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfoDto {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub descript: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    pub live: bool,
    pub state: StreamState,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>,
    pub status: bool,
    pub source: String,
    pub tags: Vec<String>,
    pub is_my_stream: bool,
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl StreamInfoDto {
    #[allow(dead_code)]
    pub fn convert(stream: Stream, user_id: i32, tags: &[String]) -> Self {
        StreamInfoDto {
            id: stream.id,
            user_id: stream.user_id,
            title: stream.title.to_owned(),
            descript: stream.descript.to_owned(),
            logo: stream.logo.clone(),
            starttime: stream.starttime.to_owned(),
            live: stream.live,
            state: stream.state.to_owned(),
            started: stream.started.clone(),
            stopped: stream.stopped.clone(),
            status: stream.status.clone(),
            source: stream.source.to_owned(),
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            is_my_stream: stream.user_id == user_id,
            created_at: stream.created_at.to_owned(),
            updated_at: stream.updated_at.to_owned(),
        }
    }
    /// Merge a "stream" and a corresponding list of "tags".
    pub fn merge_streams_and_tags(
        streams: &[Stream],
        stream_tags: &[StreamTagStreamId],
        user_id: i32,
    ) -> Vec<StreamInfoDto> {
        let mut result: Vec<StreamInfoDto> = Vec::new();

        let mut tags_map: HashMap<i32, Vec<String>> = HashMap::new();
        #[rustfmt::skip]
        let mut curr_stream_id: i32 = if stream_tags.len() > 0 { stream_tags[0].stream_id } else { -1 };
        let mut tags: Vec<String> = vec![];
        for stream_tag in stream_tags.iter() {
            if curr_stream_id != stream_tag.stream_id {
                tags_map.insert(curr_stream_id, tags.clone());
                tags.clear();
                curr_stream_id = stream_tag.stream_id;
            }
            tags.push(stream_tag.name.to_string());
        }
        tags_map.insert(curr_stream_id, tags.clone());

        for stream in streams.iter() {
            let stream = stream.clone();
            let mut tags: Vec<String> = Vec::new();
            let tags_opt = tags_map.get(&stream.id);
            if let Some(tags_vec) = tags_opt {
                tags.extend(tags_vec.clone());
            }
            let stream_info_dto = StreamInfoDto::convert(stream, user_id, &tags);
            result.push(stream_info_dto);
        }
        result
    }
}

// **  Section: table "streams" data creation **

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::streams)]
pub struct CreateStream {
    pub user_id: i32,
    pub title: String,                  // min_len=2 max_len=255
    pub descript: Option<String>,       // type: Text default ""
    pub logo: Option<String>,           // min_len=2 max_len=255 Nullable
    pub starttime: DateTime<Utc>,       //
    pub live: Option<bool>,             // default false
    pub state: Option<StreamState>,     // default Waiting
    pub started: Option<DateTime<Utc>>, // Nullable
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub status: Option<bool>,           // default true
    pub source: Option<String>,         // min_len=2 max_len=255 default "obs"
}

impl CreateStream {
    pub fn convert(create_stream_info: CreateStreamInfoDto, user_id: i32) -> Self {
        let min_date_time = Utc::now() + Duration::minutes(2);
        CreateStream {
            user_id: user_id,
            title: create_stream_info.title.to_owned(),
            descript: create_stream_info.descript.clone(),
            logo: None,
            starttime: create_stream_info.starttime.unwrap_or(min_date_time),
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: create_stream_info.source.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStreamInfoDto {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serial_datetime_option")]
    pub starttime: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub tags: Vec<String>,
}

impl Validator for CreateStreamInfoDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(ValidationChecks::required(&self.title, MSG_TITLE_REQUIRED).err());
        errors.push(validate_title(&self.title).err());

        if let Some(value) = &self.descript {
            errors.push(validate_descript(&value).err());
        }
        if let Some(value) = &self.starttime {
            errors.push(validate_starttime(value).err());
        }
        if let Some(value) = &self.source {
            errors.push(validate_source(&value).err());
        }
        errors.push(validate_tag(&self.tags).err());

        self.filter_errors(errors)
    }
}

// **  Section: table "streams" data editing **

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ModifyStream {
    pub title: Option<String>,                  // min_len=2 max_len=255
    pub descript: Option<String>,               // type: Text default ""
    pub logo: Option<Option<String>>,           // min_len=2 max_len=255 Nullable
    pub starttime: Option<DateTime<Utc>>,       //
    pub live: Option<bool>,                     // default false
    pub state: Option<StreamState>,             // default Waiting
    pub started: Option<Option<DateTime<Utc>>>, // Nullable
    pub stopped: Option<Option<DateTime<Utc>>>, // Nullable
    pub status: Option<bool>,                   // default true
    pub source: Option<String>,                 // min_len=2 max_len=255 default "obs"
}

impl ModifyStream {
    pub fn convert(modify_stream_info: ModifyStreamInfoDto) -> Self {
        ModifyStream {
            title: modify_stream_info.title.clone(),
            descript: modify_stream_info.descript.clone(),
            logo: None,
            starttime: modify_stream_info.starttime.clone(),
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: modify_stream_info.source.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifyStreamInfoDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serial_datetime_option")]
    pub starttime: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl ModifyStreamInfoDto {
    pub fn check_required_fields(&self) -> Result<(), Vec<ValidationError>> {
        let starttime_is_none = self.starttime.is_none();
        let source_is_none = self.source.is_none();
        let tags_is_none = self.tags.is_none();
        if self.title.is_none() && self.descript.is_none() && starttime_is_none && source_is_none && tags_is_none {
            let mut err = ValidationError::new(MSG_NO_REQUIRED_FIELDS);
            let data = true;
            err.add_param(borrow::Cow::Borrowed("noRequiredFields"), &data);
            return Err(vec![err]);
        }
        Ok(())
    }
}

impl Validator for ModifyStreamInfoDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.title {
            errors.push(validate_title(&value).err());
        }
        if let Some(value) = &self.descript {
            errors.push(validate_descript(&value).err());
        }
        if let Some(value) = &self.starttime {
            errors.push(validate_starttime(value).err());
        }
        if let Some(value) = &self.source {
            errors.push(validate_source(&value).err());
        }
        if let Some(value) = &self.tags {
            errors.push(validate_tag(value).err());
        }

        self.filter_errors(errors)
    }
}

// **  Section: table "stream_tags" receiving data **

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, QueryableByName)]
#[diesel(table_name = schema::stream_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StreamTag {
    pub id: i32,
    pub user_id: i32,
    pub name: String, // min_len=2 max_len=255
}

#[derive(Debug, Serialize, Deserialize, Clone, QueryableByName)]
#[diesel(table_name = schema::stream_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StreamTagStreamId {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "stream_id")]
    pub stream_id: i32,
    pub id: i32,
    pub user_id: i32,
    pub name: String,
}

// **  Section: table "link_stream_tags_to_streams" receiving data **

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = schema::link_stream_tags_to_streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LinkStreamTagsToStreams {
    pub id: i32,
    pub stream_id: i32,
    pub stream_tag_id: i32,
}

// **  Section: Search "StreamInfoDto". **

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OrderColumn {
    Starttime, // default
    Title,
}

impl std::fmt::Display for OrderColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OrderDirection {
    Asc, // default
    Desc,
}

impl std::fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

pub const SEARCH_STREAM_PAGE: u32 = 1;
pub const SEARCH_STREAM_LIMIT: u32 = 5;
pub const SEARCH_STREAM_ORDER_COLUMN: OrderColumn = OrderColumn::Starttime;
pub const SEARCH_STREAM_ORDER_DIRECTION: OrderDirection = OrderDirection::Asc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchStream {
    pub user_id: Option<i32>,
    pub live: Option<bool>,
    pub is_future: Option<bool>,
    pub order_column: Option<OrderColumn>,
    pub order_direction: Option<OrderDirection>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl SearchStream {
    pub fn convert(search_stream_info: SearchStreamInfoDto) -> Self {
        SearchStream {
            user_id: search_stream_info.user_id.clone(),
            live: search_stream_info.live.clone(),
            is_future: search_stream_info.is_future.clone(),
            order_column: search_stream_info.order_column.clone(),
            order_direction: search_stream_info.order_direction.clone(),
            page: search_stream_info.page.clone(),
            limit: search_stream_info.limit.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchStreamInfoDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    // pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    // true - (starttime >= now), false - (starttime < now)
    pub is_future: Option<bool>,
    // groupBy?: 'none' | 'tag' | 'date' = 'none';
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_column: Option<OrderColumn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_direction: Option<OrderDirection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// ** SearchResponseDto<T> **

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchStreamInfoResponseDto {
    pub list: Vec<StreamInfoDto>,
    pub limit: u32,
    pub count: u32,
    pub page: u32,
    pub pages: u32,
}

// ** **

// OLD

// ** Section: "stream_tags" table **

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::stream_tags)]
#[serde(rename_all = "camelCase")]
pub struct CreateStreamTagDto {
    pub user_id: i32,
    pub name: String, // min_len=2 max_len=255
}

impl Validator for CreateStreamTagDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_tag_name(&self.name).err());

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::stream_tags)]
pub struct ModifyStreamTagDto {
    pub name: String, // min_len=2 max_len=255
}

impl Validator for ModifyStreamTagDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_tag_name(&self.name).err());

        self.filter_errors(errors)
    }
}

// ** Section: "StreamDto" **

#[cfg(test)]
mod tests {

    use super::*;

    // ** StreamInfoDto **

    #[test]
    fn test_merge_streams_and_tags_with_one_item() {
        let user_id: i32 = 123;
        let mut tag_id = 0;

        let mut streams: Vec<Stream> = Vec::new();
        let mut stream_tags: Vec<StreamTagStreamId> = Vec::new();

        let stream = Stream::new(0, user_id, "title1", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag11".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        let streams_info: Vec<StreamInfoDto> = vec![StreamInfoDto::convert(stream, user_id, &tags)];

        let result = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, user_id);

        assert_eq!(result.len(), 1);
        assert_eq!(result, streams_info);
    }

    #[test]
    fn test_merge_streams_and_tags_with_two_items() {
        let user_id: i32 = 123;
        let mut tag_id = 0;

        let mut streams: Vec<Stream> = Vec::new();
        let mut stream_tags: Vec<StreamTagStreamId> = Vec::new();
        let mut streams_info: Vec<StreamInfoDto> = Vec::new();

        let stream = Stream::new(0, user_id, "title1", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag11,tag12".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        streams_info.push(StreamInfoDto::convert(stream, user_id, &tags));

        let stream = Stream::new(1, user_id, "title2", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag21,tag22".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        streams_info.push(StreamInfoDto::convert(stream, user_id, &tags));

        let result = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, user_id);

        assert_eq!(result.len(), 2);
        assert_eq!(result, streams_info);
    }

    #[test]
    fn test_merge_streams_and_tags_with_three_items() {
        let user_id: i32 = 123;
        let mut tag_id = 0;

        let mut streams: Vec<Stream> = Vec::new();
        let mut stream_tags: Vec<StreamTagStreamId> = Vec::new();
        let mut streams_info: Vec<StreamInfoDto> = Vec::new();

        let stream = Stream::new(0, user_id, "title1", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag11,tag12,tag13".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        streams_info.push(StreamInfoDto::convert(stream, user_id, &tags));

        let stream = Stream::new(1, user_id, "title2", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag21,tag22,tag23".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        streams_info.push(StreamInfoDto::convert(stream, user_id, &tags));

        let stream = Stream::new(2, user_id, "title3", Utc::now());
        streams.push(stream.clone());
        let tags: Vec<String> = "tag31,tag32,tag33".split(',').map(|v| v.to_string()).collect();
        for tag in tags.iter() {
            #[rustfmt::skip]
            stream_tags.push(StreamTagStreamId { stream_id: stream.id, id: tag_id, user_id, name: tag.to_string() });
            tag_id += 1;
        }
        streams_info.push(StreamInfoDto::convert(stream, user_id, &tags));

        let result = StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, user_id);

        assert_eq!(result.len(), 3);
        assert_eq!(result, streams_info);
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub struct StreamModelsTest {}

#[cfg(all(test, feature = "mockdata"))]
impl StreamModelsTest {
    pub fn title_min() -> String {
        (0..(TITLE_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn title_max() -> String {
        (0..(TITLE_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn title_enough() -> String {
        (0..(TITLE_MIN)).map(|_| 'a').collect()
    }
    pub fn descript_min() -> String {
        (0..(DESCRIPT_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn descript_max() -> String {
        (0..(DESCRIPT_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn logo_min() -> String {
        (0..(LOGO_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn logo_max() -> String {
        (0..(LOGO_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn source_min() -> String {
        (0..(SOURCE_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn source_max() -> String {
        (0..(SOURCE_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn tag_name_min() -> String {
        (0..(TAG_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn tag_name_enough() -> String {
        (0..(TAG_MIN)).map(|_| 'a').collect()
    }
    pub fn tag_name_max() -> String {
        (0..(TAG_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn tag_names_min() -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let tag_name: String = (0..TAG_MIN).map(|_| 'a').collect();
        let min_value = TAG_MIN_AMOUNT - 1;
        let mut idx = 0;
        while idx < min_value {
            result.push(format!("{}{}", tag_name, idx));
            idx += 1;
        }
        result
    }
    pub fn tag_names_max() -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let tag_name: String = (0..TAG_MIN).map(|_| 'a').collect();
        let max_value = TAG_MAX_AMOUNT + 1;
        let mut idx = 0;
        while idx < max_value {
            result.push(format!("{}{}", tag_name, idx));
            idx += 1;
        }
        result
    }
    pub fn tag_names_enough() -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let tag_name: String = (0..TAG_MIN).map(|_| 'a').collect();
        let mut idx = 0;
        while idx < TAG_MIN_AMOUNT {
            result.push(format!("{}{}", tag_name, idx));
            idx += 1;
        }
        result
    }
}

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::schema;
use crate::utils::{serial_datetime, serial_datetime_option};
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const MSG_TITLE_REQUIRED: &str = "title:required";
pub const TITLE_MIN: u8 = 2;
pub const MSG_TITLE_MIN_LENGTH: &str = "title:min_len";
pub const TITLE_MAX: u8 = 255;
pub const MSG_TITLE_MAX_LENGTH: &str = "title:max_len";

pub const LOGO_MIN: u8 = 2;
pub const MSG_LOGO_MIN_LENGTH: &str = "logo:min_len";
pub const LOGO_MAX: u8 = 255;
pub const MSG_LOGO_MAX_LENGTH: &str = "logo:max_len";

pub const MSG_SOURCE_REQUIRED: &str = "source:required";
pub const SOURCE_MIN: u8 = 2;
pub const MSG_SOURCE_MIN_LENGTH: &str = "source:min_len";
pub const SOURCE_MAX: u8 = 255;
pub const MSG_SOURCE_MAX_LENGTH: &str = "source:max_len";

pub const TAG_NAME_MIN_AMOUNT: u8 = 1;
pub const MSG_TAG_NAME_MIN_AMOUNT: &str = "name:min_amount";
pub const TAG_NAME_MAX_AMOUNT: u8 = 4;
pub const MSG_TAG_NAME_MAX_AMOUNT: &str = "name:max_amount";
pub const MSG_TAG_NAME_REQUIRED: &str = "name:required";
pub const TAG_NAME_MIN: u8 = 2;
pub const MSG_TAG_NAME_MIN_LENGTH: &str = "name:min_len";
pub const TAG_NAME_MAX: u8 = 255;
pub const MSG_TAG_NAME_MAX_LENGTH: &str = "name:max_len";

pub fn validate_title(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_TITLE_REQUIRED)?;
    ValidationChecks::min_length(value, TITLE_MIN.into(), MSG_TITLE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TITLE_MAX.into(), MSG_TITLE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_logo(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, LOGO_MIN.into(), MSG_LOGO_MIN_LENGTH)?;
    ValidationChecks::max_length(value, LOGO_MAX.into(), MSG_LOGO_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_source(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_SOURCE_REQUIRED)?;
    ValidationChecks::min_length(value, SOURCE_MIN.into(), MSG_SOURCE_MIN_LENGTH)?;
    ValidationChecks::max_length(&value, SOURCE_MAX.into(), MSG_SOURCE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_tag_name(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_TAG_NAME_REQUIRED)?;
    ValidationChecks::min_length(value, TAG_NAME_MIN.into(), MSG_TAG_NAME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TAG_NAME_MAX.into(), MSG_TAG_NAME_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_tag_names(tags: &Vec<String>) -> Result<(), ValidationError> {
    let min_amount = TAG_NAME_MIN_AMOUNT;
    ValidationChecks::min_amount_of_elem(tags.len(), min_amount.into(), MSG_TAG_NAME_MIN_AMOUNT)?;
    let max_amount = TAG_NAME_MAX_AMOUNT;
    ValidationChecks::max_amount_of_elem(tags.len(), max_amount.into(), MSG_TAG_NAME_MAX_AMOUNT)?;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StreamInfoDto {
    pub id: i32,
    #[serde(rename = "userId")]
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
    #[serde(rename = "createdAt", with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl StreamInfoDto {
    #[allow(dead_code)]
    pub fn convert(stream: Stream, user_id: i32, tags: &[&str]) -> Self {
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
}

// **  Section: table "streams" data creation **

pub const STREAM_DESCRIPT_DEF: &str = "";
pub const STREAM_LIVE_DEF: bool = false;
pub const STREAM_STATE_DEF: StreamState = StreamState::Waiting;
pub const STREAM_STATUS_DEF: bool = true;
pub const STREAM_SOURCE_DEF: &str = "obs";

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
        CreateStream {
            user_id: user_id,
            title: create_stream_info.title.to_owned(),
            descript: create_stream_info.descript.clone(),
            logo: create_stream_info.logo.clone(),
            starttime: create_stream_info.starttime.to_owned(),
            live: create_stream_info.live.clone(),
            state: create_stream_info.state.clone(),
            started: create_stream_info.started.clone(),
            stopped: create_stream_info.stopped.clone(),
            status: create_stream_info.status.clone(),
            source: create_stream_info.source.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStreamInfoDto {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StreamState>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub tags: Vec<String>,
}

impl Validator for CreateStreamInfoDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_title(&self.title).err());

        if let Some(value) = &self.logo {
            errors.push(validate_logo(&value).err());
        }

        errors.push(validate_tag_names(&self.tags).err());

        if let Some(value) = &self.source {
            errors.push(validate_source(&value).err());
        }

        self.filter_errors(errors)
    }
}

// **  Section: table "streams" data editing **

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ModifyStream {
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
}

impl ModifyStream {
    pub fn convert(modify_stream_info: ModifyStreamInfoDto, user_id: i32) -> Self {
        ModifyStream {
            user_id: user_id,
            title: modify_stream_info.title.to_owned(),
            descript: modify_stream_info.descript.clone(),
            logo: modify_stream_info.logo.clone(),
            starttime: modify_stream_info.starttime.to_owned(),
            live: modify_stream_info.live.clone(),
            state: modify_stream_info.state.clone(),
            started: modify_stream_info.started.clone(),
            stopped: modify_stream_info.stopped.clone(),
            status: modify_stream_info.status.clone(),
            source: modify_stream_info.source.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifyStreamInfoDto {
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
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none" )]
    pub stopped: Option<DateTime<Utc>>,
    pub status: bool,
    pub source: String,
    pub tags: Vec<String>,
}

impl Validator for ModifyStreamInfoDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_title(&self.title).err());

        if let Some(value) = &self.logo {
            errors.push(validate_logo(&value).err());
        }
        errors.push(validate_tag_names(&self.tags).err());
        errors.push(validate_source(&self.source).err());

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
pub struct CreateStreamTagDto {
    #[serde(rename = "userId")]
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

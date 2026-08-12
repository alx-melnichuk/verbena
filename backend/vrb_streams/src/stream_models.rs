use std::{cmp::Ordering, fmt};

use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vrb_common::{
    err, serial_datetime, serial_datetime_option,
    validators::{ValidationChecks, ValidationError, Validator},
};
use vrb_dbase::{enm_stream_state::StreamState, schema};

pub const MSG_TITLE_REQUIRED: &str = "title:required";
pub const TITLE_MIN: u8 = 2;
pub const MSG_TITLE_MIN_LENGTH: &str = "title:min_length";
pub const TITLE_MAX: u16 = 255;
pub const MSG_TITLE_MAX_LENGTH: &str = "title:max_length";

pub const DESCRIPT_MIN: u8 = 2;
pub const MSG_DESCRIPT_MIN_LENGTH: &str = "descript:min_length";
pub const DESCRIPT_MAX: u16 = 2048; // 2*1024
pub const MSG_DESCRIPT_MAX_LENGTH: &str = "descript:max_length";

pub const LOGO_MIN: u8 = 2;
pub const MSG_LOGO_MIN_LENGTH: &str = "logo:min_length";
pub const LOGO_MAX: u16 = 255;
pub const MSG_LOGO_MAX_LENGTH: &str = "logo:max_length";

pub const MSG_MIN_VALID_STARTTIME: &str = "starttime:min_valid_date";

pub const SOURCE_MIN: u8 = 2;
pub const MSG_SOURCE_MIN_LENGTH: &str = "source:min_length";
pub const SOURCE_MAX: u16 = 255;
pub const MSG_SOURCE_MAX_LENGTH: &str = "source:max_length";

pub const MSG_TAG_REQUIRED: &str = "tag:required";
pub const TAG_MIN_AMOUNT: u8 = 1;
pub const MSG_TAG_MIN_AMOUNT: &str = "tag:min_amount";
pub const TAG_MAX_AMOUNT: u8 = 4;
pub const MSG_TAG_MAX_AMOUNT: &str = "tag:max_amount";
pub const TAG_MIN: u8 = 2;
pub const MSG_TAG_MIN_LENGTH: &str = "tag:min_length";
pub const TAG_MAX: u16 = 255;
pub const MSG_TAG_MAX_LENGTH: &str = "tag:max_length";

pub const MSG_STARTTIME_REQUIRED: &str = "starttime:required";
pub const MSG_FINISHTIME_REQUIRED: &str = "finishtime:required";


// MIN=2, MAX=255
pub fn validate_title(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::required(value, MSG_TITLE_REQUIRED)?;
    ValidationChecks::min_length(value, TITLE_MIN.into(), MSG_TITLE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TITLE_MAX.into(), MSG_TITLE_MAX_LENGTH)?;
    Ok(())
}
// MIN=2, MAX=2048
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
// MIN=2, MAX=255
pub fn validate_source(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, SOURCE_MIN.into(), MSG_SOURCE_MIN_LENGTH)?;
    ValidationChecks::max_length(&value, SOURCE_MAX.into(), MSG_SOURCE_MAX_LENGTH)?;
    Ok(())
}
// MIN=2, MAX=255
pub fn validate_tag(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, TAG_MIN.into(), MSG_TAG_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TAG_MAX.into(), MSG_TAG_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_tag_amount(tags: &[String]) -> Result<(), ValidationError> {
    let min_amount = TAG_MIN_AMOUNT;
    ValidationChecks::min_amount(tags.len(), min_amount.into(), MSG_TAG_MIN_AMOUNT)?;
    let max_amount = TAG_MAX_AMOUNT;
    ValidationChecks::max_amount(tags.len(), max_amount.into(), MSG_TAG_MAX_AMOUNT)?;
    Ok(())
}
pub fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    validate_tag_amount(tags)?;
    for tag in tags {
        validate_tag(tag)?;
    }
    Ok(())
}
// * * * * Section: models for "StreamOrm". * * * *

// ** Model: "StreamAndTags". Used to return "stream" and "tags" data. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StreamAndTags {
    pub id: i32,
    pub user_id: i32,
    pub title: String,                  // min_len=2 max_len=255
    pub descript: String,               // min_len=2,max_len=2048 default ""
    pub logo: Option<String>,           // min_len=2 max_len=255 Nullable
    pub starttime: DateTime<Utc>,       //
    pub live: bool,                     // default false
    pub state: StreamState,             // default Waiting
    pub started: Option<DateTime<Utc>>, // Nullable
    pub paused: Option<DateTime<Utc>>,  // Nullable
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub source: String,                 // min_len=2 max_len=255 default "obs"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Text>)]
    #[diesel(column_name = "tags")]
    pub tags: Vec<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    #[diesel(column_name = "old_logo")]
    pub old_logo: Option<String>,
}

pub const STREAM_AND_TAGS_SOURCE_DEF: &str = "obs";

impl StreamAndTags {
    pub fn new(id: i32, user_id: i32, title: &str, starttime: DateTime<Utc>, tags: &[String]) -> StreamAndTags {
        let now = Utc::now();
        StreamAndTags {
            id: id,
            user_id: user_id,
            title: title.to_owned(),
            descript: String::default(),
            logo: None,
            starttime: starttime.clone(),
            live: StreamState::is_live(StreamState::default()),
            state: StreamState::default(),
            started: None,
            paused: None,
            stopped: None,
            source: STREAM_AND_TAGS_SOURCE_DEF.to_string(),
            created_at: now,
            updated_at: now,
            tags: tags.to_vec().clone(),
            old_logo: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[rustfmt::skip]
pub struct StreamAndTagsDto {
    pub id: i32,
    pub user_id: i32,
    pub title: String,                  // min_len=2 max_len=255
    pub descript: String,               // min_len=2,max_len=2048 default ""
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,           // min_len=2 max_len=255 Nullable
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    pub live: bool,                     // default false
    pub state: StreamState,             // default Waiting
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>, // Nullable
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub paused: Option<DateTime<Utc>>,  // Nullable
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub source: String,                 // min_len=2 max_len=255 default "obs"
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl From<StreamAndTags> for StreamAndTagsDto {
    fn from(stream_and_tags: StreamAndTags) -> Self {
        StreamAndTagsDto {
            id: stream_and_tags.id,
            user_id: stream_and_tags.user_id,
            title: stream_and_tags.title.to_owned(),
            descript: stream_and_tags.descript.clone(),
            logo: stream_and_tags.logo.clone(),
            starttime: stream_and_tags.starttime.clone(),
            live: StreamState::is_live(stream_and_tags.state),
            state: stream_and_tags.state.clone(),
            started: stream_and_tags.started.clone(),
            paused: stream_and_tags.paused.clone(),
            stopped: stream_and_tags.stopped.clone(),
            source: stream_and_tags.source.clone(),
            created_at: stream_and_tags.created_at.clone(),
            updated_at: stream_and_tags.updated_at.clone(),
            tags: stream_and_tags.tags.to_vec().clone(),
        }
    }
}

// ** Model Dto: "StreamConfigDto". Used: in "stream_controller::get_stream_config()". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamConfigDto {
    // Maximum size for logo files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_max_size: Option<u32>,
    // List of valid input mime types for logo files.
    // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
    pub logo_valid_types: Vec<String>,
    // Logo files will be converted to this MIME type.
    // Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_ext: Option<String>,
    // Maximum width of logo image after saving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_max_width: Option<u32>,
    // Maximum height of logo image after saving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_max_height: Option<u32>,
}

impl StreamConfigDto {
    pub fn new(
        max_size: Option<u32>,
        valid_types: Vec<String>,
        ext: Option<String>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> StreamConfigDto {
        StreamConfigDto {
            logo_max_size: max_size.clone(),
            logo_valid_types: valid_types.clone(),
            logo_ext: ext.clone(),
            logo_max_width: max_width.clone(),
            logo_max_height: max_height.clone(),
        }
    }
}

// ** Model: "CreateStreamAndTags". Used: StreamOrm::create_stream_and_tags() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateStreamAndTags {
    pub user_id: i32,
    pub title: String,                  // min_len=2 max_len=255
    pub descript: Option<String>,       // min_len=2,max_len=2048 default ""
    pub logo: Option<String>,           // min_len=2 max_len=255 Nullable
    pub starttime: DateTime<Utc>,       //
    pub state: Option<StreamState>,     // default Waiting
    pub started: Option<DateTime<Utc>>, // Nullable
    pub paused: Option<DateTime<Utc>>,  // Nullable
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub source: Option<String>,         // min_len=2 max_len=255 default "obs"
    #[diesel(skip_update)]
    pub tags: Vec<String>,
}

impl Into<StreamAndTags> for CreateStreamAndTags {
    fn into(self) -> StreamAndTags {
        let state = self.state.unwrap_or_default().clone();
        let now = Utc::now();
        StreamAndTags {
            id: i32::default(),
            user_id: self.user_id,
            title: self.title.to_owned(),
            descript: self.descript.unwrap_or_default().clone(),
            logo: self.logo.clone(),
            starttime: self.starttime.clone(),
            live: StreamState::is_live(state.clone()),
            state: state.clone(),
            started: self.started.clone(),
            paused: self.paused.clone(),
            stopped: self.stopped.clone(),
            source: self.source.unwrap_or_default().clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            tags: self.tags.to_vec().clone(),
            old_logo: None,
        }
    }
}

// ** Model Dto: "CreateStreamAndTagsInfoDto". Used: "stream_controller::post_stream_and_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateStreamAndTagsDto {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serial_datetime_option")]
    pub starttime: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub tags: Vec<String>,
}

impl Validator for CreateStreamAndTagsDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

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
        if self.tags.len() == 0 {
            errors.push(ValidationChecks::required(&self.tags.join(","), MSG_TAG_REQUIRED).err());
        } else {
            errors.push(validate_tags(&self.tags).err());
        }

        self.filter_errors(errors)
    }
}

impl Into<CreateStreamAndTags> for CreateStreamAndTagsDto {
    fn into(self) -> CreateStreamAndTags {
        let min_date_time = Utc::now() + Duration::minutes(2);
        CreateStreamAndTags {
            user_id: i32::default(),
            title: self.title.clone(),
            descript: self.descript.clone(),
            logo: None,
            starttime: self.starttime.unwrap_or(min_date_time),
            state: None,
            started: None,
            paused: None,
            stopped: None,
            source: self.source.clone(),
            tags: self.tags.to_vec().clone(),
        }
    }
}

// ** Model: "ModifyStreamAndTags". Used: StreamOrm::modify_stream_and_tags() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ModifyStreamAndTags {
    pub title: Option<String>,            // min_len=2 max_len=255
    pub descript: Option<String>,         // min_len=2,max_len=2048 default ""
    pub logo: Option<String>,             // min_len=2 max_len=255 Nullable
    pub starttime: Option<DateTime<Utc>>, //
    pub state: Option<StreamState>,       // default Waiting
    pub started: Option<DateTime<Utc>>,   // Nullable
    pub paused: Option<DateTime<Utc>>,    // Nullable
    pub stopped: Option<DateTime<Utc>>,   // Nullable
    pub source: Option<String>,           // min_len=2 max_len=255 default "obs"
    #[diesel(skip_update)]
    pub tags: Option<Vec<String>>,
}

impl ModifyStreamAndTags {
    pub fn is_empty(&self) -> bool {
        let is_title = self.title.is_none();
        let is_descript = self.descript.is_none();
        let is_logo = self.logo.is_none();
        let is_starttime = self.starttime.is_none();
        let is_state = self.state.is_none();
        let is_started = self.started.is_none();
        let is_paused = self.paused.is_none();
        let is_stopped = self.stopped.is_none();
        let is_source = self.source.is_none();

        is_title && is_descript && is_logo && is_starttime && is_state && is_started && is_paused && is_stopped && is_source
    }
    pub fn merge(&self, stream: StreamAndTags) -> StreamAndTags {
        let logo = if let Some(val) = self.logo.clone() {
            if val.len() > 0 {  Some(val) } else { None }
        } else {
            stream.logo.clone()
        };
        let state = self.state.clone().unwrap_or(stream.state.clone());
        StreamAndTags {
            id: stream.id.clone(),
            user_id: stream.user_id.clone(),
            title: self.title.clone().unwrap_or(stream.title.clone()),
            descript: self.descript.clone().unwrap_or(stream.descript.clone()),
            logo: logo,
            starttime: self.starttime.clone().unwrap_or(stream.starttime.clone()),
            live: StreamState::is_live(state),
            state: state,
            started: if self.started.is_some() { self.started.clone() } else { stream.started.clone() },
            paused: if self.paused.is_some() { self.paused.clone() } else { stream.paused.clone() },
            stopped: if self.stopped.is_some() { self.stopped.clone() } else { stream.stopped.clone() },
            source: self.source.clone().unwrap_or(stream.source.clone()),
            created_at: stream.created_at.clone(),
            updated_at: Utc::now(),
            tags: self.tags.clone().unwrap_or(stream.tags.clone()),
            old_logo: stream.logo.clone(),
        }
    }
}

// ** Model Dto: "ModifyStreamAndTagsDto". Used: "stream_controller::put_stream_and_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyStreamAndTagsDto {
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

impl ModifyStreamAndTagsDto {
    pub fn valid_names<'a>() -> Vec<&'a str> {
        vec!["title", "descript", "starttime", "source", "tags"]
    }
}

impl Validator for ModifyStreamAndTagsDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(value) = &self.title {
            errors.push(validate_title(&value).err());
        }
        if let Some(value) = &self.descript {
            // The field is optional and we check if there is a value.
            if value.len() > 0 {
                errors.push(validate_descript(&value).err());
            }
        }
        if let Some(value) = &self.starttime {
            errors.push(validate_starttime(value).err());
        }
        if let Some(value) = &self.source {
            // The field is optional and we check if there is a value.
            if value.len() > 0 {
                errors.push(validate_source(&value).err());
            }
        }
        if let Some(value) = &self.tags {
            errors.push(validate_tags(value).err());
        }

        let list_is_some = vec![
            self.title.is_some(),
            self.descript.is_some(),
            self.starttime.is_some(),
            self.source.is_some(),
            self.tags.is_some(),
        ];
        let valid_names = Self::valid_names().join(",");
        errors.push(ValidationChecks::no_fields_to_update(&list_is_some, &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err());

        self.filter_errors(errors)
    }
}

impl Into<ModifyStreamAndTags> for ModifyStreamAndTagsDto {
    fn into(self) -> ModifyStreamAndTags {
        ModifyStreamAndTags {
            title: self.title.clone(),
            descript: self.descript.clone(),
            logo: None,
            starttime: self.starttime.clone(),
            state: None,
            started: None,
            paused: None,
            stopped: None,
            source: self.source.clone(),
            tags: self.tags.clone(),
        }
    }
}

// ** Model Dto: "ToggleStreamStateDto". Used: "stream_controller::put_toggle_state()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToggleStreamStateDto {
    pub state: StreamState,
}

// ** Model: "SearchStreamAndTags". Used: StreamOrm::filter_stream_and_tags_by_pages() **

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FilterStream {
    Future,
    Past,
    Period,
}

impl fmt::Display for FilterStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

pub const SEARCH_STREAM_AND_TAGS_PAGE: u32 = 1;
pub const SEARCH_STREAM_AND_TAGS_LIMIT: u32 = 5;
pub const SEARCH_STREAM_AND_TAGS_LIMIT_MIN: u32 = 1;
pub const SEARCH_STREAM_AND_TAGS_LIMIT_MAX: u32 = 100;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchStreamAndTags {
    pub user_id: Option<i32>,
    pub live: Option<bool>,
    pub filter: Option<FilterStream>,
    pub starttime: Option<DateTime<Utc>>,
    pub finishtime: Option<DateTime<Utc>>,
    pub sort_desc: Option<bool>,
    pub tag: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl SearchStreamAndTags {
    pub fn new(user_id: Option<i32>, filter: Option<FilterStream>, starttime: Option<DateTime<Utc>>, finishtime: Option<DateTime<Utc>>) -> Self {
        Self {
            user_id,
            live: None,
            filter,
            starttime,
            finishtime,
            sort_desc: None,
            tag: None,
            page: None,
            limit: None,
        }
    }
}

// ** Model: "CountStreamAndTags". Used: StreamOrm::filter_stream_and_tags_by_pages(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::streams)]
pub struct CountStreamAndTags {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = "cnt")]
    pub cnt: i32,
}

// ** Model Dto: "SearchStreamAndTagsDto". Used: in "stream_controller::get_stream_and_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchStreamAndTagsDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterStream>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serial_datetime_option")]
    pub starttime: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serial_datetime_option")]
    pub finishtime: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_desc: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl From<SearchStreamAndTagsDto> for SearchStreamAndTags {
    fn from(item: SearchStreamAndTagsDto) -> Self {
        SearchStreamAndTags {
            user_id: item.user_id.clone(),
            live: item.live.clone(),
            filter: item.filter.clone(),
            starttime: item.starttime.clone(),
            finishtime: item.finishtime.clone(),
            sort_desc: item.sort_desc.clone(),
            tag: item.tag.clone(),
            page: item.page.clone(),
            limit: item.limit.clone(),
        }
    }
}

impl From<SearchStreamAndTags> for SearchStreamAndTagsDto {
    fn from(item: SearchStreamAndTags) -> Self {
        SearchStreamAndTagsDto {
            user_id: item.user_id.clone(),
            live: item.live.clone(),
            filter: item.filter.clone(),
            starttime: item.starttime.clone(),
            finishtime: item.finishtime.clone(),
            sort_desc: item.sort_desc.clone(),
            tag: item.tag.clone(),
            page: item.page.clone(),
            limit: item.limit.clone(),
        }
    }
}

impl Validator for SearchStreamAndTagsDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        if let Some(FilterStream::Future) = &self.filter {
            if self.starttime.is_none() {
                errors.push(ValidationChecks::required("", MSG_STARTTIME_REQUIRED).err());
            }
        } else if let Some(FilterStream::Past) = &self.filter {
            if self.starttime.is_none() {
                errors.push(ValidationChecks::required("", MSG_STARTTIME_REQUIRED).err());
            }
        } else if let Some(FilterStream::Period) = &self.filter {
            if self.starttime.is_none() {
                errors.push(ValidationChecks::required("", MSG_STARTTIME_REQUIRED).err());
            }
            if self.finishtime.is_none() {
                errors.push(ValidationChecks::required("", MSG_FINISHTIME_REQUIRED).err());
            }
        }
        if let Some(tag) = &self.tag {
            errors.push(validate_tag(tag).err());
        }
        self.filter_errors(errors)
    }
}

// ** Model Dto: "PageStreamAndTagsDto". Used: in "stream_controller::get_stream_and_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageStreamAndTagsDto {
    #[schema(example = json!(Self::create_stream_and_tags(1,2)))]
    pub list: Vec<StreamAndTagsDto>,
    #[schema(example = 5)]
    pub limit: u32,
    #[schema(example = 2)]
    pub count: u32,
    #[schema(example = 1)]
    pub page: u32,
    #[schema(example = 1)]
    pub pages: u32,
}

impl PageStreamAndTagsDto {
    pub fn create_stream_and_tags(user_id: i32, amount: i32) -> Vec<StreamAndTagsDto> {
        let mut result: Vec<StreamAndTagsDto> = Vec::new();
        let mut idx = 1;
        while idx <= amount {
            let title = &format!("title_{}", idx);
            let tags = &format!("tag01,tag{:0>2}", idx);
            let tags1: Vec<String> = tags.split(',').map(|val| val.to_string()).collect();
            let mut stream = StreamAndTags::new(idx, user_id, title, Utc::now(), &tags1);
            stream.descript = format!("descript_{}", idx);

            let stream_and_tags_dto: StreamAndTagsDto = stream.into();
            result.push(stream_and_tags_dto);
            idx += 1;
        }
        result
    }
}

// ** Model: "SearchStreamDate". Used: StreamOrm::filter_stream_dates()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchStreamDate {
    pub user_id: i32,
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

// ** Model: "StartStreamAndTags". Used: StreamOrm::filter_stream_dates(). **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::streams)]
pub struct StartStreamAndTags {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    #[diesel(column_name = "start")]
    pub start: DateTime<Utc>,
}

// ** Model Dto: "SearchStreamDateDto". Used: in "stream_controller::get_streams_period()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchStreamDateDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
    #[serde(with = "serial_datetime")]
    pub start: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub finish: DateTime<Utc>,
}

impl Into<SearchStreamDate> for SearchStreamDateDto {
    fn into(self) -> SearchStreamDate {
        let max_finish = self.start.clone() + Duration::days(31) - Duration::milliseconds(1);
        #[rustfmt::skip]
        let finish = 
        if self.start.cmp(&self.finish) == Ordering::Greater || self.finish.cmp(&max_finish) == Ordering::Greater {
            max_finish
        } else {
            self.finish.clone()
        };

        SearchStreamDate {
            user_id: self.user_id.unwrap_or_default(),
            start: self.start.clone(),
            finish: finish.clone(),
        }
    }
}

// ** Model: "SearchStreamTags". Used: StreamOrm::get_stream_tags_by_pages() **

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StreamTagSortColumn {
    Id,
    Name,
    CountLinks,
}

impl fmt::Display for StreamTagSortColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

pub const SEARCH_STREAM_TAGS_PAGE: u32 = 1;
pub const SEARCH_STREAM_TAGS_LIMIT: u32 = 12;
pub const SEARCH_STREAM_TAGS_LIMIT_MIN: u32 = 1;
pub const SEARCH_STREAM_TAGS_LIMIT_MAX: u32 = 100;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchStreamTag {
    pub sort_column: Option<StreamTagSortColumn>,
    pub sort_desc: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName)]
#[diesel(table_name = schema::stream_tags)]
pub struct StreamTag {
    pub id: i32,
    pub name: String,
    pub count_links: i32,
}

impl StreamTag {
    pub fn new(id: i32, name: &str, count_links: i32) -> Self {
        Self { id, name: name.to_owned(), count_links }
    }
}
// ** Model Dto: "SearchStreamTagDto". Used: in "stream_controller::get_stream_popural_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchStreamTagDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_column: Option<StreamTagSortColumn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_desc: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl From<SearchStreamTagDto> for SearchStreamTag {
    fn from(item: SearchStreamTagDto) -> Self {
        SearchStreamTag {
            sort_column: item.sort_column.clone(),
            sort_desc: item.sort_desc.clone(),
            page: item.page.clone(),
            limit: item.limit.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[rustfmt::skip]
pub struct StreamTagDto {
    pub id: i32,
    pub name: String,
    pub count_links: i32,
}

impl From<StreamTag> for StreamTagDto {
    fn from(stream_tag: StreamTag) -> Self {
        StreamTagDto {
            id: stream_tag.id.clone(),
            name: stream_tag.name.clone(),
            count_links: stream_tag.count_links.clone(),
        }
    }
}

// ** Model Dto: "PageStreamTagDto". Used: in "stream_controller::get_stream_popural_tags()" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageStreamTagDto {
    #[schema(example = json!(Self::create_stream_tags(4)))]
    pub list: Vec<StreamTagDto>,
    #[schema(example = 5)]
    pub limit: u32,
    #[schema(example = 1)]
    pub page: u32,
}

impl PageStreamTagDto {
    pub fn create_stream_tags(amount: i32) -> Vec<StreamTagDto> {
        let mut result: Vec<StreamTagDto> = Vec::new();
        let mut idx = 1;
        while idx <= amount {
            let strm_tag = StreamTagDto {
                id: idx,
                name: format!("tag_{}", idx),
                count_links: idx * 3,
            };
            result.push(strm_tag);
            idx += 1;
        }
        result
    }
}

// ** - **

#[cfg(all(test, feature = "mockdata"))]
pub struct StreamMock {}

#[cfg(all(test, feature = "mockdata"))]
impl StreamMock {
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

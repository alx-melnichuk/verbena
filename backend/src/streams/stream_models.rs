use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema;
use crate::utils::{serial_datetime, serial_datetime_option};
use crate::validators::{ValidationChecks, ValidationError, Validator};

pub const TITLE_MIN: u8 = 3;
pub const MSG_TITLE_MIN_LENGTH: &str = "title:min_length";
pub const TITLE_MAX: u8 = 255;
pub const MSG_TITLE_MAX_LENGTH: &str = "title:max_length";

pub const LOGO_MAX: u8 = 255;
pub const MSG_LOGO_MAX_LENGTH: &str = "logo:max_length";

pub const SOURCE_MAX: u8 = 255;
pub const MSG_SOURCE_MAX_LENGTH: &str = "source:max_length";

pub fn validate_title(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, TITLE_MIN.into(), MSG_TITLE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, TITLE_MAX.into(), MSG_TITLE_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_logo(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::max_length(value, LOGO_MAX.into(), MSG_LOGO_MAX_LENGTH)?;
    Ok(())
}
pub fn validate_source(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::max_length(&value, SOURCE_MAX.into(), MSG_SOURCE_MAX_LENGTH)?;
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

// ** Section: database "streams" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stream {
    pub id: i32,
    pub user_id: i32,
    pub title: String,        // max_length = 255
    pub descript: String,     // type: Text
    pub logo: Option<String>, // max_length = 255 Nullable
    pub starttime: DateTime<Utc>,
    pub live: bool,
    pub state: StreamState,
    pub started: Option<DateTime<Utc>>, // Nullable
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub status: bool,
    pub source: String, // max_length = 255
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::streams)]
pub struct StreamDto {
    pub id: i32,
    pub user_id: i32,
    pub title: String,    // max_length = 255
    pub descript: String, // type: Text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>, // max_length = 255 Nullable
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    pub live: bool,
    pub state: StreamState,
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>, // Nullable
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub status: bool,
    pub source: String, // max_length = 255
    #[serde(rename = "createdAt", with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl From<Stream> for StreamDto {
    fn from(stream: Stream) -> Self {
        StreamDto {
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
            status: stream.status,
            source: stream.source.to_owned(),
            created_at: stream.created_at.to_owned(),
            updated_at: stream.updated_at.to_owned(),
        }
    }
}

pub const DEF_SOURCE: &str = "obs";
pub const DEF_STATUS: bool = true;

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::streams)]
pub struct CreateStreamDto {
    pub user_id: i32,
    pub title: String, // max_length = 255
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descript: Option<String>, // type: Text DEFAULT ''
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>, // max_length = 255 Nullable
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>, // DEFAULT FALSE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<StreamState>, // DEFAULT 'waiting'
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>, // Nullable
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>, // Nullable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>, // DEFAULT TRUE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>, // max_length = 255 // NOT NULL DEFAULT 'obs'
}

impl Validator for CreateStreamDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_title(&self.title).err());
        if let Some(value) = &self.logo {
            errors.push(validate_logo(&value).err());
        }
        if let Some(value) = &self.source {
            errors.push(validate_source(&value).err());
        }

        self.filter_errors(errors)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = schema::streams)]
pub struct ModifyStreamDto {
    pub title: String,    // max_length = 255
    pub descript: String, // type: Text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>, // max_length = 255 Nullable
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    pub live: bool,
    pub state: StreamState,
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>, // Nullable
    #[rustfmt::skip]
    #[serde(with = "serial_datetime_option", skip_serializing_if = "Option::is_none" )]
    pub stopped: Option<DateTime<Utc>>, // Nullable
    pub status: bool,
    pub source: String, // max_length = 255
}

impl Validator for ModifyStreamDto {
    // Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<Option<ValidationError>> = vec![];

        errors.push(validate_title(&self.title).err());
        if let Some(value) = &self.logo {
            errors.push(validate_logo(&value).err());
        }
        errors.push(validate_source(&self.source).err());

        self.filter_errors(errors)
    }
}

use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, diesel_derive_enum::DbEnum, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::StreamState"]
#[DbValueStyle = "snake_case"] // BazQuxx => "baz_quxx"
#[serde(rename_all = "lowercase")]
pub enum StreamState {
    Waiting,   // (default)
    Preparing, // (is live)
    Started,   // (is live)
    Paused,    // (is live)
    Stopped,
}

impl fmt::Display for StreamState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

impl StreamState {
    pub fn is_live(stream_state: StreamState) -> bool {
        stream_state == StreamState::Preparing || stream_state == StreamState::Started || stream_state == StreamState::Paused
    }
}


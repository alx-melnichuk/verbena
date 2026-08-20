use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "stream_state", rename_all = "lowercase")]
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

impl Default for StreamState {
    fn default() -> Self {
        Self::Waiting
    }
}

impl StreamState {
    pub fn is_live(stream_state: StreamState) -> bool {
        stream_state == StreamState::Preparing || stream_state == StreamState::Started || stream_state == StreamState::Paused
    }
}

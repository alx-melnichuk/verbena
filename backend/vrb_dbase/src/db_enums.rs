use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, diesel_derive_enum::DbEnum, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl UserRole {
    pub fn all_values() -> Vec<UserRole> {
        vec![UserRole::Admin, UserRole::User, UserRole::Moderator]
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

impl TryFrom<&str> for UserRole {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let vec: Vec<UserRole> = UserRole::all_values();
        let value = value.to_lowercase();
        let res = vec.iter().position(|&ur| ur.to_string() == value);

        if let Some(index) = res {
            Ok(vec.get(index).unwrap().clone())
        } else {
            Err(())
        }
    }
}

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


use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db_enums::UserRole;
use crate::schema;

// ** Model: "User". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String, // max_len: 255
    pub email: String, // max_len: 255
    pub password: String, // max_len: 255
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: UserRole,
}

impl User {
    pub fn new(id: i32, nickname: &str, email: &str, password: &str, role: UserRole) -> Self {
        let now = Utc::now();
        User {
            id,
            nickname: nickname.into(), // max_len: 255
            email: email.into(), // max_len: 255
            password: password.into(), // max_len: 255
            created_at: now.clone(),
            updated_at: now.clone(),
            role,
        }
    }
}

// ** Model: "Session". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub user_id: i32,
    pub num_token: Option<i32>,
}

impl Session {
    pub fn new(user_id: i32, num_token: Option<i32>) -> Self {
        Session { user_id, num_token }
    }
}

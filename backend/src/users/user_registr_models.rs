use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema;
use crate::utils::date_time_rfc2822z;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::user_registration)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRegistr {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

// ** Section: DTO models. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::user_registration)]
pub struct UserRegistrDto {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "finalDate", with = "date_time_rfc2822z")]
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_registration)]
pub struct CreateUserRegistrDto {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use vrb_dbase::schema;

// * * * * Section: models for "UserRegistrOrm". * * * *

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

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_registration)]
pub struct CreateUserRegistr {
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub final_date: DateTime<Utc>,
}

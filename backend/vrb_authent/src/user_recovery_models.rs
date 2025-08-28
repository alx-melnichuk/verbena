use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use vrb_dbase::schema;

// ** Section: "UserRecovery" **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = schema::user_recovery)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRecovery {
    pub id: i32,
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name = schema::user_recovery)]
pub struct CreateUserRecovery {
    pub user_id: i32,
    pub final_date: DateTime<Utc>,
}

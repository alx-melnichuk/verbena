use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use vrb_dbase::{enm_user_role::UserRole, schema};

// ** Model: "User". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String, // max_len: 255
    pub email: String,    // max_len: 255
    pub password: String, // max_len: 255
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i32, nickname: &str, email: &str, password: &str, role: UserRole) -> Self {
        let now = Utc::now();
        User {
            id,
            nickname: nickname.into(), // max_len: 255
            email: email.into(),       // max_len: 255
            password: password.into(), // max_len: 255
            role,
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

// ** Used: UserOrm::create_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUser {
    pub nickname: String,       // min_len=3 max_len=64
    pub email: String,          // min_len=5 max_len=254
    pub password: String,       // min_len=6 max_len=64
    pub role: Option<UserRole>, // default "user"
}

impl CreateUser {
    pub fn new(nickname: &str, email: &str, password: &str, role: Option<UserRole>) -> Self {
        CreateUser {
            nickname: nickname.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
            role: role.clone(),
        }
    }
}

// ** Used: UserOrm::modify_user() **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ModifyUser {
    pub nickname: Option<String>, // min_len=3,max_len=64
    pub email: Option<String>,    // min_len=5,max_len=254,"email:email_type"
    pub password: Option<String>, // min_len=6,max_len=64
    pub role: Option<UserRole>,   // default "user"
}

impl ModifyUser {
    pub fn new(nickname: Option<String>, email: Option<String>, password: Option<String>, role: Option<UserRole>) -> Self {
        ModifyUser {
            nickname,
            email,
            password,
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

// ** Model: "Profile". **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, QueryableByName, Queryable, Selectable)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
    pub user_id: i32,
    pub avatar: Option<String>,
    pub descript: Option<String>,
    pub theme: Option<String>,
    pub locale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Profile {
    pub fn new(user_id: i32, avatar: Option<String>, descript: Option<String>, theme: Option<String>, locale: Option<String>) -> Self {
        let now = Utc::now();
        Profile {
            user_id,
            avatar,
            descript,
            theme,
            locale,
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

// * * * *   * * * *

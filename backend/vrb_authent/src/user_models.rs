use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(all(test, feature = "mockdata"))]
use vrb_common::user_validations;
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

// * * * *   * * * *

#[cfg(all(test, feature = "mockdata"))]
pub struct UserMock {}

#[cfg(all(test, feature = "mockdata"))]
impl UserMock {
    pub fn nickname_min() -> String {
        (0..(user_validations::NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn nickname_max() -> String {
        (0..(user_validations::NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn nickname_wrong() -> String {
        let nickname: String = (0..(user_validations::NICKNAME_MIN - 1)).map(|_| 'a').collect();
        format!("{}#", nickname)
    }
    pub fn email_min() -> String {
        let suffix = "@us".to_owned();
        let email_min: usize = user_validations::EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn email_max() -> String {
        let email_max: usize = user_validations::EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        format!("{}@{}{}", prefix, suffix, domain)
    }
    pub fn email_wrong() -> String {
        let suffix = "@".to_owned();
        let email_min: usize = user_validations::EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn password_min() -> String {
        (0..(user_validations::PASSWORD_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn password_max() -> String {
        (0..(user_validations::PASSWORD_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn password_wrong() -> String {
        (0..(user_validations::PASSWORD_MIN)).map(|_| 'a').collect()
    }
    pub fn role_wrong() -> String {
        let role = UserRole::all_values().get(0).unwrap().to_string();
        role[0..(role.len() - 1)].to_string()
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

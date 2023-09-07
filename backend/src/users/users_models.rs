use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::users::users_consts::EMAIL_NAME;
use crate::utils::errors::AppError;
use crate::utils::option_date_time;
use crate::utils::validations::Validations;
use crate::{schema, users::users_consts::PASSWORD_NAME};

use super::users_consts::{
    EMAIL_MAX, EMAIL_MIN, ERR_CODE_MODEL_IS_EMPTY, ERR_MSG_MODEL_IS_EMPTY, NICKNAME_MAX,
    NICKNAME_MIN, NICKNAME_NAME, PASSWORD_MAX, PASSWORD_MIN,
};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct UserDTO {
    pub id: Option<i32>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    #[serde(
        default = "UserDTO::option_none",
        rename = "createdAt",
        skip_serializing_if = "Option::is_none",
        with = "option_date_time"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        default = "UserDTO::option_none",
        rename = "updatedAt",
        skip_serializing_if = "Option::is_none",
        with = "option_date_time"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDTO {
    fn from(user: User) -> Self {
        UserDTO {
            id: Some(user.id),
            nickname: Some(user.nickname),
            email: Some(user.email),
            password: Some(user.password),
            created_at: Some(user.created_at),
            updated_at: Some(user.updated_at),
        }
    }
}

impl UserDTO {
    pub fn is_empty(user_dto: &UserDTO) -> bool {
        user_dto.id.is_none()
            && user_dto.nickname.is_none()
            && user_dto.email.is_none()
            && user_dto.password.is_none()
    }
    pub fn new() -> Self {
        UserDTO {
            id: None,
            nickname: None,
            email: None,
            password: None,
            created_at: None,
            updated_at: None,
        }
    }
    pub fn option_none() -> Option<DateTime<Utc>> {
        Option::None
    }
    pub fn clear_optional(user_dto: &mut UserDTO) {
        user_dto.id = None;
        user_dto.created_at = None;
        user_dto.updated_at = None;
    }
    // pub fn validation_for_add(user_dto: &UserDTO) -> Vec<AppError> {
    //     let mut result = vec![];

    //     let nickname = user_dto.nickname.clone().unwrap_or("".to_string());
    //     let res1 = Validations::required(&nickname, NICKNAME_NAME);
    //     if res1.len() > 0 {
    //         result.extend(res1);
    //     } else {
    //         result.extend(Validations::min_len(&nickname, NICKNAME_MIN, NICKNAME_NAME));
    //         result.extend(Validations::max_len(&nickname, NICKNAME_MAX, NICKNAME_NAME));
    //     }

    //     let email = user_dto.email.clone().unwrap_or("".to_string());
    //     let res2 = Validations::required(&email, EMAIL_NAME);
    //     if res2.len() > 0 {
    //         result.extend(res2);
    //     } else {
    //         result.extend(Validations::min_len(&email, EMAIL_MIN, EMAIL_NAME));
    //         result.extend(Validations::max_len(&email, EMAIL_MAX, EMAIL_NAME));
    //     }

    //     let password = user_dto.password.clone().unwrap_or("".to_string());
    //     let res3 = Validations::required(&password, PASSWORD_NAME);
    //     if res3.len() > 0 {
    //         result.extend(res3);
    //     } else {
    //         result.extend(Validations::min_len(&password, PASSWORD_MIN, PASSWORD_NAME));
    //         result.extend(Validations::max_len(&password, PASSWORD_MAX, PASSWORD_NAME));
    //     }

    //     result
    // }
    pub fn validation_for_edit(user_dto: &UserDTO) -> Vec<AppError> {
        let mut result = vec![];

        if let Some(nickname) = &user_dto.nickname {
            result.extend(Validations::min_len(&nickname, NICKNAME_MIN, NICKNAME_NAME));
            result.extend(Validations::max_len(&nickname, NICKNAME_MAX, NICKNAME_NAME));
        } else if let Some(email) = &user_dto.email {
            result.extend(Validations::min_len(&email, EMAIL_MIN, EMAIL_NAME));
            result.extend(Validations::max_len(&email, EMAIL_MAX, EMAIL_NAME));
        } else if let Some(password) = &user_dto.password {
            result.extend(Validations::min_len(&password, PASSWORD_MIN, PASSWORD_NAME));
            result.extend(Validations::max_len(&password, PASSWORD_MAX, PASSWORD_NAME));
        } else {
            result.push(AppError::InvalidField(
                ERR_CODE_MODEL_IS_EMPTY.to_string(),
                ERR_MSG_MODEL_IS_EMPTY.to_string(),
                HashMap::from([]),
            ));
        }

        result
    }
    pub fn validation_for_login(user_dto: &UserDTO) -> Vec<AppError> {
        let mut result = vec![];

        let nickname = user_dto.nickname.clone().unwrap_or("".to_string());
        let res1 = Validations::required(&nickname, NICKNAME_NAME);
        if res1.len() > 0 {
            result.extend(res1);
        } else {
            result.extend(Validations::min_len(&nickname, NICKNAME_MIN, NICKNAME_NAME));
            result.extend(Validations::max_len(&nickname, NICKNAME_MAX, NICKNAME_NAME));
        }

        let email = user_dto.email.clone().unwrap_or("".to_string());
        let res2 = Validations::required(&email, EMAIL_NAME);
        if res2.len() > 0 {
            result.extend(res2);
        } else {
            result.extend(Validations::min_len(&email, EMAIL_MIN, EMAIL_NAME));
            result.extend(Validations::max_len(&email, EMAIL_MAX, EMAIL_NAME));
        }

        let password = user_dto.password.clone().unwrap_or("".to_string());
        let res3 = Validations::required(&password, PASSWORD_NAME);
        if res3.len() > 0 {
            result.extend(res3);
        } else {
            result.extend(Validations::min_len(&password, PASSWORD_MIN, PASSWORD_NAME));
            result.extend(Validations::max_len(&password, PASSWORD_MAX, PASSWORD_NAME));
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = schema::users)]
pub struct CreateUserDTO {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

impl CreateUserDTO {
    pub fn validation(create_user_dto: &CreateUserDTO) -> Vec<AppError> {
        let mut result = vec![];

        let nickname = create_user_dto.nickname;
        let res1 = Validations::required(&nickname, NICKNAME_NAME);
        if res1.len() > 0 {
            result.extend(res1);
        } else {
            result.extend(Validations::min_len(&nickname, NICKNAME_MIN, NICKNAME_NAME));
            result.extend(Validations::max_len(&nickname, NICKNAME_MAX, NICKNAME_NAME));
        }

        let email = create_user_dto.email;
        let res2 = Validations::required(&email, EMAIL_NAME);
        if res2.len() > 0 {
            result.extend(res2);
        } else {
            result.extend(Validations::min_len(&email, EMAIL_MIN, EMAIL_NAME));
            result.extend(Validations::max_len(&email, EMAIL_MAX, EMAIL_NAME));
        }

        let password = create_user_dto.password;
        let res3 = Validations::required(&password, PASSWORD_NAME);
        if res3.len() > 0 {
            result.extend(res3);
        } else {
            result.extend(Validations::min_len(&password, PASSWORD_MIN, PASSWORD_NAME));
            result.extend(Validations::max_len(&password, PASSWORD_MAX, PASSWORD_NAME));
        }

        result
    }
}

impl From<UserDTO> for CreateUserDTO {
    fn from(user_dto: UserDTO) -> CreateUserDTO {
        CreateUserDTO {
            nickname: user_dto.nickname.clone().unwrap_or("".to_string()),
            email: user_dto.email.clone().unwrap_or("".to_string()),
            password: user_dto.password.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUserDTO {
    pub nickname: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfoDTO {
    pub username: String,
    pub login_session: String,
}

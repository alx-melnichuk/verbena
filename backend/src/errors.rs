use std::fmt;
use std::{borrow::Cow, collections::BTreeMap};

use actix_web::{http, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use validator::{ValidationError, ValidationErrors};

pub const ERR_CN_SERVER_ERROR: &str = "InternalServerError";
pub const ERR_MSG_SERVER_ERROR: &str = "An unexpected internal server error occurred.";
pub const ERR_CN_VALIDATION: &str = "Validation";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppError {
    #[serde(rename = "errCode")]
    pub code: Cow<'static, str>,
    #[serde(rename = "errMsg")]
    pub message: Cow<'static, str>,
    #[serde(
        skip_serializing_if = "BTreeMap::is_empty",
        default = "AppError::default_params"
    )]
    // Parameters must be sorted by key.
    pub params: BTreeMap<Cow<'static, str>, Value>,
    #[serde(skip, default = "AppError::default_status")]
    pub status: u16,
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl AppError {
    pub fn new<'a>(code: &'a str, message: &'a str) -> Self {
        #[rustfmt::skip]
        let code = if code.len() > 0 { code } else { ERR_CN_SERVER_ERROR };
        #[rustfmt::skip]
        let message = if message.len() > 0 { message } else { ERR_MSG_SERVER_ERROR };
        AppError {
            code: Cow::from(code.to_string()),
            message: Cow::from(message.to_string()),
            params: BTreeMap::new(),
            status: 500,
        }
    }

    pub fn set_status(&mut self, status: u16) -> Self {
        self.status = status;
        self.to_owned()
    }

    pub fn add_param<'a, T: Serialize>(&mut self, name: Cow<'a, str>, val: &T) -> Self {
        self.params.insert(name.to_string().into(), to_value(val).unwrap());
        self.to_owned()
    }

    pub fn status_code(&self) -> http::StatusCode {
        match self.status {
            400 => http::StatusCode::BAD_REQUEST,
            404 => http::StatusCode::NOT_FOUND,
            409 => http::StatusCode::CONFLICT,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn default_status() -> u16 {
        500
    }
    pub fn default_params() -> BTreeMap<Cow<'static, str>, Value> {
        BTreeMap::new()
    }
    #[rustfmt::skip]
    pub fn add_field_error_params(&mut self, prefix: &str, field_errors: &Vec<ValidationError>) -> Self {
        for validation_error in field_errors {
            let prefix_len = prefix.len();
            for (key, val) in validation_error.params.clone().into_iter() {
                let key_param = if prefix_len > 0 { Cow::from(format!("{}:{}", prefix, key)) } else { key };
                self.add_param(key_param, &val);
            }
        }
        self.to_owned()
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        self.status_code()
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        #[cfg(test)]
        eprintln!("AppError({}): {}", self.status_code(), self.to_string()); // #
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json(self)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errs: ValidationErrors) -> Self {
        let mut app_error = AppError::new(ERR_CN_VALIDATION, &errs.to_string()).set_status(400);
        for (key, val) in errs.field_errors().into_iter() {
            app_error.add_field_error_params(key, val);
        }
        app_error
    }
}

use std::fmt;
use std::{borrow::Cow, collections::BTreeMap};

use actix_web::{http, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use validator::ValidationErrors;

pub const CN_SERVER_ERROR: &str = "InternalServerError";
pub const MSG_SERVER_ERROR: &str = "An unexpected internal server error occurred.";
pub const CN_VALIDATION: &str = "Validation";

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
        let code = if code.len() > 0 { code } else { CN_SERVER_ERROR };
        #[rustfmt::skip]
        let message = if message.len() > 0 { message } else { MSG_SERVER_ERROR };
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
            401 => http::StatusCode::UNAUTHORIZED,
            403 => http::StatusCode::FORBIDDEN,
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
    pub fn validation_response(errors: ValidationErrors) -> HttpResponse {
        let status_code = http::StatusCode::BAD_REQUEST;
        let mut app_error_vec: Vec<AppError> = vec![];

        for (name, valid_error_vec) in errors.field_errors().into_iter() {
            let valid_error_opt = valid_error_vec.get(0);
            if let Some(valid_error) = valid_error_opt {
                let message = valid_error.message.clone().unwrap_or(std::borrow::Cow::Borrowed(""));
                eprintln!("## name: {} message: {}", name.clone(), message.clone());
                let mut app_error =
                    AppError::new(&valid_error.code, &message).set_status(status_code.into());
                for (key, val) in valid_error.params.clone().into_iter() {
                    eprintln!("## key: {} val: {}", key, val);
                    if key != "value" {
                        app_error.add_param(key, &val);
                    }
                }
                app_error_vec.push(app_error);
            }
        }
        let json = app_error_vec;
        HttpResponse::build(status_code)
            .insert_header(http::header::ContentType::json())
            .json(json)
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        self.status_code()
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        // #[cfg(test)]
        // #[rustfmt::skip]
        // eprintln!("AppError({} {}): {}", self.status, self.status_code(), self.to_string() ); // #
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json(self)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errs: ValidationErrors) -> Self {
        let mut app_error = AppError::new(CN_VALIDATION, &errs.to_string()).set_status(400);
        let mut err_msg_vec: Vec<String> = vec![];
        let len = errs.field_errors().len() - 1;
        for (idx, (key_error, val)) in errs.field_errors().into_iter().enumerate() {
            let valid_error_opt = val.get(0);
            if let Some(valid_error) = valid_error_opt {
                let code = valid_error.code.to_string();
                let message = valid_error.message.clone().unwrap_or_else(|| Cow::Borrowed(""));
                eprintln!("key_error: {key_error}, valid_error.code: {code}, message: {message}");
                let char = if idx < len { "\n" } else { "" };
                let err_msg = Cow::from(format!("{}{}", message, char));
                err_msg_vec.push(err_msg.to_string());

                for (key, val) in valid_error.params.clone().into_iter() {
                    let key_error_param = Cow::from(format!("{}:{}", key_error.to_string(), key));
                    app_error.add_param(key_error_param, &val);
                }
            }
            // app_error.add_field_error_params(key_error, val);
        }

        let messing = err_msg_vec.into_iter().collect();
        app_error.message = messing;

        app_error
    }
}

use std::{borrow::Cow, collections::BTreeMap};

use actix_web::{http, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use utoipa::ToSchema;

use crate::{settings::err, validators::ValidationError};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: Cow<'static, str>,
    pub message: Cow<'static, str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default = "AppError::default_params")]
    // Parameters must be sorted by key.
    pub params: BTreeMap<Cow<'static, str>, Value>,
    #[serde(skip, default = "AppError::default_status")]
    pub status: u16,
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl AppError {
    pub fn new<'a>(code: &'a str, message: &'a str) -> Self {
        #[rustfmt::skip]
        let code = if code.len() > 0 { code } else { err::CD_INTER_SRV_ERROR };
        #[rustfmt::skip]
        let message = if message.len() > 0 { message } else { err::MSG_INTER_SRV_ERROR };
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
            506 => http::StatusCode::VARIANT_ALSO_NEGOTIATES,
            507 => http::StatusCode::INSUFFICIENT_STORAGE,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn default_status() -> u16 {
        500
    }
    pub fn default_params() -> BTreeMap<Cow<'static, str>, Value> {
        BTreeMap::new()
    }
    pub fn validations_to_response(errors: Vec<ValidationError>) -> HttpResponse {
        let status_code = http::StatusCode::BAD_REQUEST; // 400
        let mut app_error_vec: Vec<AppError> = vec![];

        for error in errors.into_iter() {
            let code = err::CD_VALIDATION;
            let message = error.message.clone();
            let mut app_error = AppError::new(&code, &message).set_status(status_code.into());

            for (key, val) in error.params.into_iter() {
                app_error.add_param(key, &val);
            }
            app_error_vec.push(app_error);
        }

        let json = app_error_vec;
        HttpResponse::build(status_code)
            .insert_header(http::header::ContentType::json())
            .json(json)
    }
    /// Error while parsing data. (status_code=417)
    pub fn parse417(param: &str, message: &str) -> Self {
        let message = &format!("Failed conversion '{}': {}", param, message);
        AppError::new(err::CD_PARSE_ERROR, message).set_status(417)
    }
    /// Error while blocking process. (status_code=506)
    pub fn blocking506(err: &str) -> AppError {
        AppError::new(err::CD_BLOCKING, err).set_status(506)
    }
    /// Error when querying the database. (status_code=507)
    pub fn database507(message: &str) -> Self {
        AppError::new(err::CD_DATABASE, message).set_status(507)
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        self.status_code()
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json(self)
    }
}

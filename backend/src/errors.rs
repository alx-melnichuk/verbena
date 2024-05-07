use std::{borrow::Cow, collections::BTreeMap};

use actix_web::{http, HttpResponse};
use mime;
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
            415 => http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            417 => http::StatusCode::EXPECTATION_FAILED,
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
    /// List of errors when validating parameters.
    pub fn validations(errors: Vec<ValidationError>) -> Vec<Self> {
        let mut result: Vec<Self> = vec![];
        for error in errors.into_iter() {
            // let message = error.message.clone();
            let mut app_error = AppError::validation417(&error.message.clone());
            for (key, val) in error.params.into_iter() {
                app_error.add_param(key, &val);
            }
            result.push(app_error);
        }
        result
    }
    /// Converting the error vector into http-response.
    pub fn to_response(errors: &[Self]) -> HttpResponse {
        let default = AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_INTER_SRV_ERROR);
        let app_error = errors.get(0).unwrap_or(&default);
        let status_code = app_error.status_code();
        HttpResponse::build(status_code)
            .insert_header(http::header::ContentType(mime::APPLICATION_JSON))
            .insert_header((mime::CHARSET.as_str(), mime::UTF_8.as_str()))
            .json(errors)
    }
    /// Error while parsing data. (status=415)
    pub fn parse415(param: &str, message: &str) -> Self {
        let message = &format!("Failed conversion '{}': {}", param, message);
        AppError::new(err::CD_PARSE_ERROR, message).set_status(415)
    }
    /// Error when data validation. (status=417)
    pub fn validation417(message: &str) -> Self {
        AppError::new(err::CD_VALIDATION, message).set_status(417)
    }
    /// Error while blocking process. (status=506)
    pub fn blocking506(err: &str) -> AppError {
        AppError::new(err::CD_BLOCKING, err).set_status(506)
    }
    /// Error when querying the database. (status=507)
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
            // .insert_header(http::header::ContentType::json())
            .insert_header(http::header::ContentType(mime::APPLICATION_JSON))
            .insert_header((mime::CHARSET.as_str(), mime::UTF_8.as_str()))
            .json(self)
    }
}

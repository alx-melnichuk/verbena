use actix_web::{error::BlockingError, http, HttpResponse};
use diesel::{result::DatabaseErrorKind, result::Error as DieselError};
use serde::Serialize;
use std::{collections::HashMap, fmt};

pub type HashMapStringString = HashMap<String, String>;

#[derive(Serialize)]
pub struct AppErrorBody {
    #[serde(rename = "errType")]
    pub err_type: String,
    #[serde(rename = "errCode")]
    pub err_code: String,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    #[serde(rename = "errObj", skip_serializing_if = "Option::is_none")]
    pub err_obj: Option<HashMapStringString>,
}

impl AppErrorBody {
    pub fn new(
        err_type: String,
        err_code: String,
        err_msg: String,
        err_obj: Option<HashMapStringString>,
    ) -> Self {
        AppErrorBody {
            err_type,
            err_code,
            err_msg,
            err_obj,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    InvalidField(String, String, HashMapStringString), // 400 (code, msg, obj)
    BadRequest(String),                                // 400
    NotFound(String),                                  // 404
    R2D2Error(String),                                 // 500
    BlockingError(String),                             // 500
    DieselError(DieselError),                          // 500
    DBaseUnique(String),                               // 409
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::InvalidField(_, msg, _) => write!(f, "{}", msg),
            AppError::BadRequest(info) => write!(f, "{}", info),
            AppError::NotFound(info) => write!(f, "{}", info),
            AppError::R2D2Error(info) => write!(f, "{}", info),
            AppError::BlockingError(info) => write!(f, "{}", info),
            AppError::DieselError(info) => write!(f, "{:?}", info),
            AppError::DBaseUnique(info) => write!(f, "{}", info),
        }
    }
}

impl AppError {
    pub fn get_type(&self) -> String {
        match self {
            AppError::InvalidField(..) => "InvalidField".to_string(),
            AppError::BadRequest(..) => "BadRequest".to_string(),
            AppError::NotFound(..) => "NotFound".to_string(),
            AppError::R2D2Error(..) => "R2D2Error".to_string(),
            AppError::BlockingError(..) => "BlockingError".to_string(),
            AppError::DieselError(..) => "DieselError".to_string(),
            AppError::DBaseUnique(..) => "DBaseUnique".to_string(),
        }
    }
    pub fn get_code(&self) -> String {
        match self {
            AppError::InvalidField(code, _, _) => code.to_string(),
            _ => "".to_string(),
        }
    }
    pub fn get_obj(&self) -> Option<HashMapStringString> {
        match self {
            AppError::InvalidField(_, _, buff) => Some((*buff).clone()),
            _ => None,
        }
    }
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            AppError::InvalidField(..) => http::StatusCode::BAD_REQUEST,
            AppError::BadRequest(..) => http::StatusCode::BAD_REQUEST,
            AppError::NotFound(..) => http::StatusCode::NOT_FOUND,
            AppError::DBaseUnique(..) => http::StatusCode::CONFLICT,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn get_http_response(errors: &Vec<AppError>) -> HttpResponse {
        let opt_err = errors.get(0);

        if opt_err.is_none() {
            return HttpResponse::Ok().json("bad error".to_string());
        }
        let status_code = opt_err.unwrap().status_code();

        let list: Vec<AppErrorBody> = errors
            .iter()
            .map(|error| {
                AppErrorBody::new(
                    error.get_type(),
                    error.get_code(),
                    error.to_string(),
                    error.get_obj(),
                )
            })
            .collect();

        HttpResponse::build(status_code)
            .insert_header(http::header::ContentType::json())
            .json(list)
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        self.status_code()
    }
    fn error_response(&self) -> HttpResponse {
        let s1 = self.to_string();
        eprintln!("error_response(): {}", s1);
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json(AppErrorBody::new(
                self.get_type(),
                self.get_code(),
                self.to_string(),
                self.get_obj(),
            ))
    }
}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> AppError {
        match err {
            DieselError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return AppError::DBaseUnique(message);
                }
                AppError::DieselError(DieselError::DatabaseError(kind, info))
            }
            _ => AppError::DieselError(err),
        }
    }
}

impl From<BlockingError> for AppError {
    fn from(err: BlockingError) -> Self {
        AppError::BlockingError(err.to_string())
    }
}

impl std::convert::From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        AppError::R2D2Error(err.to_string())
    }
}

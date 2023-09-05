use actix_web::{error::BlockingError, http, HttpResponse};
use diesel::{result::DatabaseErrorKind, result::Error as DieselError};
use serde::Serialize;
use std::{collections::HashMap, fmt};

#[derive(Serialize)]
pub struct AppErrorBody {
    #[serde(rename = "errType")]
    pub err_type: String,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    #[serde(rename = "errObj", skip_serializing_if = "Option::is_none")]
    pub err_obj: Option<HashMap<&'static str, &'static str>>,
}

impl AppErrorBody {
    pub fn new(
        err_type: String,
        err_msg: String,
        err_obj: Option<HashMap<&'static str, &'static str>>,
    ) -> Self {
        AppErrorBody {
            err_type,
            err_msg,
            err_obj,
        }
    }
}
// AppError::InvalidData(String, HashMap<&'static str, &'static str>)
#[derive(Debug)]
pub enum AppError {
    InvalidData(String, HashMap<&'static str, &'static str>), // 400
    BadRequest(String),                                       // 400
    NotFound(String),                                         // 404
    R2D2Error(String),                                        // 500
    BlockingError(String),                                    // 500
    DieselError(DieselError),                                 // 500
    DBaseUnique(String),                                      // 409
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::InvalidData(info, _) => write!(f, "{}", info),
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
            AppError::InvalidData(..) => "InvalidData".to_string(),
            AppError::BadRequest(..) => "BadRequest".to_string(),
            AppError::NotFound(..) => "NotFound".to_string(),
            AppError::R2D2Error(..) => "R2D2Error".to_string(),
            AppError::BlockingError(..) => "BlockingError".to_string(),
            AppError::DieselError(..) => "DieselError".to_string(),
            AppError::DBaseUnique(..) => "DBaseUnique".to_string(),
        }
    }
    pub fn get_obj(&self) -> Option<HashMap<&'static str, &'static str>> {
        match self {
            AppError::InvalidData(_, buff) => Some((*buff).clone()),
            _ => None,
        }
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            AppError::InvalidData(..) => http::StatusCode::BAD_REQUEST,
            AppError::BadRequest(..) => http::StatusCode::BAD_REQUEST,
            AppError::NotFound(..) => http::StatusCode::NOT_FOUND,
            AppError::R2D2Error(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BlockingError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DieselError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DBaseUnique(..) => http::StatusCode::CONFLICT,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let s1 = self.to_string();
        eprintln!("error_response(): {}", s1);
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json({
                let error_body =
                    AppErrorBody::new(self.get_type(), self.to_string(), self.get_obj());
                error_body
            })
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

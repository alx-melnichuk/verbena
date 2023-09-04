use actix_web::{error::BlockingError, http, HttpResponse};
use diesel::{result::DatabaseErrorKind, result::Error as DieselError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),       // 400
    NotFound(String),         // 404
    R2D2Error(String),        // 500
    BlockingError(String),    // 500
    DieselError(DieselError), // 500
    DatabaseUnique(String),   // 409
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::BadRequest(info) => write!(f, "{}", info),
            AppError::NotFound(info) => write!(f, "{}", info),
            AppError::R2D2Error(info) => write!(f, "{}", info),
            AppError::BlockingError(info) => write!(f, "{}", info),
            AppError::DieselError(info) => write!(f, "{:?}", info),
            AppError::DatabaseUnique(info) => write!(f, "{}", info),
        }
    }
}

#[derive(Serialize)]
pub struct AppErrorBody {
    #[serde(rename = "errType")]
    pub err_type: String,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            AppError::BadRequest(..) => http::StatusCode::BAD_REQUEST,
            AppError::NotFound(..) => http::StatusCode::NOT_FOUND,
            AppError::R2D2Error(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BlockingError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DieselError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseUnique(..) => http::StatusCode::CONFLICT,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let s1 = self.to_string();
        eprintln!("error_response(): {}", s1);
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::json())
            .json(AppErrorBody {
                err_type: self.get_type(),
                err_msg: self.to_string(),
            })
    }
}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> AppError {
        match err {
            DieselError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return AppError::DatabaseUnique(message);
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

impl AppError {
    fn get_type(&self) -> String {
        match *self {
            AppError::BadRequest(..) => "BadRequest".to_string(),
            AppError::NotFound(..) => "NotFound".to_string(),
            AppError::R2D2Error(..) => "R2D2Error".to_string(),
            AppError::BlockingError(..) => "BlockingError".to_string(),
            AppError::DieselError(..) => "DieselError".to_string(),
            AppError::DatabaseUnique(..) => "DatabaseUnique".to_string(),
        }
    }
}

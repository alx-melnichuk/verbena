use std::{borrow, collections::BTreeMap, error, fmt};

use actix_web::{http, HttpResponse};
use mime;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};
use utoipa::ToSchema;

// 500 Internal Server Error - Internal error when accessing the server API.
pub const MSG_INTER_SRV_ERROR: &str = "internal_error_accessing_server_api";

fn code_to_str(status_code: http::StatusCode) -> String {
    status_code
        .canonical_reason()
        .map(|v| v.replace(" ", ""))
        .unwrap_or("".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: borrow::Cow<'static, str>,
    pub message: borrow::Cow<'static, str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default = "ApiError::default_params")]
    // Parameters must be sorted by key.
    pub params: BTreeMap<borrow::Cow<'static, str>, Value>,
    #[serde(skip, default = "ApiError::default_status")]
    pub status: u16,
}

impl error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", to_string(&self).unwrap())
    }
}

impl ApiError {
    pub fn new<'a>(code: &'a str, message: &'a str) -> Self {
        #[rustfmt::skip]
        let code = if code.len() > 0 { code } else { &code_to_str(http::StatusCode::INTERNAL_SERVER_ERROR) };
        #[rustfmt::skip]
        let message = if message.len() > 0 { message } else { MSG_INTER_SRV_ERROR };
        ApiError {
            code: borrow::Cow::from(code.to_string()),
            message: borrow::Cow::from(message.to_string()),
            params: BTreeMap::new(),
            status: 500,
        }
    }

    pub fn set_status(&mut self, status: u16) -> Self {
        self.status = status;
        self.to_owned()
    }

    pub fn add_param<'a, T: Serialize>(&mut self, name: borrow::Cow<'a, str>, val: &T) -> Self {
        self.params.insert(name.to_string().into(), to_value(val).unwrap());
        self.to_owned()
    }

    pub fn status_code(&self) -> http::StatusCode {
        http::StatusCode::from_u16(self.status).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn default_status() -> u16 {
        500
    }
    pub fn default_params() -> BTreeMap<borrow::Cow<'static, str>, Value> {
        BTreeMap::new()
    }

    /// Converting the error vector into http-response.
    pub fn to_response(errors: &[Self]) -> HttpResponse {
        let default = ApiError::internal_err500(MSG_INTER_SRV_ERROR);
        let app_error = errors.get(0).unwrap_or(&default);
        let status_code = app_error.status_code();
        HttpResponse::build(status_code)
            .insert_header(http::header::ContentType(mime::APPLICATION_JSON))
            .insert_header((mime::CHARSET.as_str(), mime::UTF_8.as_str()))
            .json(errors)
    }
    /// Authorization required. (status=401)
    pub fn unauthorized401(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::UNAUTHORIZED), message).set_status(401)
    }
    /// Insufficient access rights (i.e. access denied). (status=403)
    pub fn forbidden403(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::FORBIDDEN), message).set_status(403)
    }
    /// Resource is not found. (status=404)
    pub fn not_found404(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::NOT_FOUND), message).set_status(404)
    }
    /// The value provided is not valid. (status=406)
    pub fn not_acceptable406(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::NOT_ACCEPTABLE), message).set_status(406)
    }
    /// A conflict situation has arisen.(status=409)
    pub fn conflict409(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::CONFLICT), message).set_status(409)
    }
    /// The request object exceeds the limits defined by the server. (status=413)
    pub fn content_large413(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::PAYLOAD_TOO_LARGE), message).set_status(413)
    }
    /// Error: Data type is not supported. (status=415)
    pub fn unsupported_type415(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::UNSUPPORTED_MEDIA_TYPE), message).set_status(415)
    }
    /// Error, requested range not satisfiable. (status=416)
    pub fn range_not_satisfiable416(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::RANGE_NOT_SATISFIABLE), message).set_status(416)
    }
    /// Error, expectation failed (when data validation). (status=417)
    pub fn validation417(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::EXPECTATION_FAILED), message).set_status(417)
    }
    /// Error, data cannot be processed. (status=422)
    pub fn unprocessable422(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::UNPROCESSABLE_ENTITY), message).set_status(422)
    }
    /// Internal Server Error. (status=500)
    pub fn internal_err500(message: &str) -> ApiError {
        ApiError::new(&code_to_str(http::StatusCode::INTERNAL_SERVER_ERROR), message).set_status(500)
    }
    /// Error, Variant Also Negotiates (while blocking process). (status=506)
    pub fn blocking506(message: &str) -> ApiError {
        ApiError::new(&code_to_str(http::StatusCode::VARIANT_ALSO_NEGOTIATES), message).set_status(506)
    }
    /// Error, Insufficient Storage (when querying the database). (status=507)
    pub fn database507(message: &str) -> Self {
        ApiError::new(&code_to_str(http::StatusCode::INSUFFICIENT_STORAGE), message).set_status(507)
    }
    // Error: Not expanded. (status=510)
    pub fn not_extended510(message: &str) -> ApiError {
        ApiError::new(&code_to_str(http::StatusCode::NOT_EXTENDED), message).set_status(510)
    }
}

impl actix_web::ResponseError for ApiError {
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

use std::{borrow, collections::BTreeMap, error, fmt};

use actix_web::{
    http::{header, StatusCode},
    HttpResponse,
};
use mime;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};
use utoipa::ToSchema;

use crate::validators::ValidationError;

// 500 Internal Server Error - Internal error when accessing the server API.
pub const MSG_INTER_SRV_ERROR: &str = "internal_error_accessing_server_api";

pub fn code_to_str(status_code: StatusCode) -> String {
    status_code.canonical_reason().map(|v| v.replace(" ", "")).unwrap_or("".to_string())
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
    /// Create a new instance of the ApiError structure.
    pub fn new<'a>(status: u16, message: &'a str) -> Self {
        let status = Self::u16_to_status_code_or_default(status);
        #[rustfmt::skip]
        let message = if message.len() > 0 { message } else { Self::default_message() };
        ApiError {
            code: borrow::Cow::from(code_to_str(status)),
            message: borrow::Cow::from(message.to_string()),
            params: BTreeMap::new(),
            status: status.as_u16(),
        }
    }
    /// Create a new instance of the ApiError structure from the parameters.
    pub fn create<'a>(status: u16, message: &'a str, text: &'a str) -> Self {
        ApiError::new(status, &format!("{}; {}", message, text))
    }
    /// Convert value from u16 to StatusCode (or default value).
    pub fn u16_to_status_code_or_default(status: u16) -> StatusCode {
        let default_value = StatusCode::INTERNAL_SERVER_ERROR;
        if status > 0 {
            StatusCode::from_u16(status).unwrap_or(default_value)
        } else {
            default_value
        }
    }
    /// Default value of the "status" field.
    pub fn default_status() -> u16 {
        Self::u16_to_status_code_or_default(0).as_u16()
    }
    /// The default value of the "params" field.
    pub fn default_params() -> BTreeMap<borrow::Cow<'static, str>, Value> {
        BTreeMap::new()
    }
    /// The default value of the "message" field.
    pub fn default_message() -> &'static str {
        MSG_INTER_SRV_ERROR
    }
    /// Set the value of the "status" field.
    pub fn set_status(&mut self, status: u16) -> Self {
        if self.status != status {
            self.status = status;
            self.code = borrow::Cow::from(code_to_str(self.status_code()));
        }
        self.to_owned()
    }
    /// Add a new parameter to the "params" field.
    pub fn add_param<'a, T: Serialize>(&mut self, name: borrow::Cow<'a, str>, val: &T) -> Self {
        self.params.insert(name.to_string().into(), to_value(val).unwrap());
        self.to_owned()
    }
    /// Convert the "status" field to "StatusCode" type.
    pub fn status_code(&self) -> StatusCode {
        Self::u16_to_status_code_or_default(self.status)
    }
    /// List of errors when validating parameters.
    pub fn validations(errors: Vec<ValidationError>) -> Vec<Self> {
        let mut result: Vec<Self> = vec![];
        for error in errors.into_iter() {
            // let message = error.message.clone();
            let mut app_error = ApiError::new(417, &error.message.clone());
            for (key, val) in error.params.into_iter() {
                app_error.add_param(key, &val);
            }
            result.push(app_error);
        }
        result
    }
    /// Converting the error vector into http-response.
    pub fn to_response(errors: &[Self]) -> HttpResponse {
        let default = ApiError::new(Self::default_status(), Self::default_message());
        let api_error = errors.get(0).unwrap_or(&default);
        let status_code = api_error.status_code();
        HttpResponse::build(status_code)
            .insert_header(header::ContentType(mime::APPLICATION_JSON))
            .insert_header((mime::CHARSET.as_str(), mime::UTF_8.as_str()))
            .json(errors)
    }

    /// Bad Request. (status=400)
    pub fn bad_request400(message: &str) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST.as_u16(), message).set_status(400)
    }
    /// Authorization required. (status=401)
    pub fn unauthorized401(message: &str) -> Self {
        ApiError::new(StatusCode::UNAUTHORIZED.as_u16(), message).set_status(401)
    }
    /// Payment Required. (status=402)
    pub fn payment_required402(message: &str) -> Self {
        ApiError::new(StatusCode::PAYMENT_REQUIRED.as_u16(), message).set_status(402)
    }
    /// Forbidden. (status=403) Insufficient access rights.
    pub fn forbidden403(message: &str) -> Self {
        ApiError::new(StatusCode::FORBIDDEN.as_u16(), message).set_status(403)
    }
    /// Not Found. (status=404) Resource is not found.
    pub fn not_found404(message: &str) -> Self {
        ApiError::new(StatusCode::NOT_FOUND.as_u16(), message).set_status(404)
    }
    /// Method Not Allowed. (status=405)
    pub fn method_not_allowed405(message: &str) -> Self {
        ApiError::new(StatusCode::METHOD_NOT_ALLOWED.as_u16(), message).set_status(405)
    }
    /// Not Acceptable. (status=406) The value provided is not valid.
    pub fn not_acceptable406(message: &str) -> Self {
        ApiError::new(StatusCode::NOT_ACCEPTABLE.as_u16(), message).set_status(406)
    }
    /// Proxy Authentication Required. (status=407)
    pub fn not_acceptable407(message: &str) -> Self {
        ApiError::new(StatusCode::PROXY_AUTHENTICATION_REQUIRED.as_u16(), message).set_status(407)
    }
    /// Request Timeout. (status=408)
    pub fn request_timeout408(message: &str) -> Self {
        ApiError::new(StatusCode::REQUEST_TIMEOUT.as_u16(), message).set_status(408)
    }
    /// Conflict. (status=409)
    pub fn conflict409(message: &str) -> Self {
        ApiError::new(StatusCode::CONFLICT.as_u16(), message).set_status(409)
    }
    /// Gone. (status=410)
    pub fn gone410(message: &str) -> Self {
        ApiError::new(StatusCode::GONE.as_u16(), message).set_status(410)
    }
    /// Length Required. (status=411)
    pub fn length_required411(message: &str) -> Self {
        ApiError::new(StatusCode::LENGTH_REQUIRED.as_u16(), message).set_status(411)
    }
    /// Precondition Failed. (status=412)
    pub fn precondition_failed412(message: &str) -> Self {
        ApiError::new(StatusCode::PRECONDITION_FAILED.as_u16(), message).set_status(412)
    }
    /// Payload Too Large. (status=413) The request object exceeds the limits defined by the server.
    pub fn payload_too_large413(message: &str) -> Self {
        ApiError::new(StatusCode::PAYLOAD_TOO_LARGE.as_u16(), message).set_status(413)
    }
    /// URI Too Long. (status=414)
    pub fn uri_too_long414(message: &str) -> Self {
        ApiError::new(StatusCode::URI_TOO_LONG.as_u16(), message).set_status(414)
    }
    /// Unsupported Media Type. (status=415)
    pub fn unsupported_media_type415(message: &str) -> Self {
        ApiError::new(StatusCode::UNSUPPORTED_MEDIA_TYPE.as_u16(), message).set_status(415)
    }
    /// Range Not Satisfiable. (status=416)
    pub fn range_not_satisfiable416(message: &str) -> Self {
        ApiError::new(StatusCode::RANGE_NOT_SATISFIABLE.as_u16(), message).set_status(416)
    }
    /// Expectation Failed. (status=417) (validation)
    pub fn expectation_failed417(message: &str) -> Self {
        ApiError::new(StatusCode::EXPECTATION_FAILED.as_u16(), message).set_status(417)
    }
    /// I'm a teapot. (status=418)
    pub fn im_a_teapot418(message: &str) -> Self {
        ApiError::new(StatusCode::IM_A_TEAPOT.as_u16(), message).set_status(418)
    }
    /// Misdirected Request. (status=421)
    pub fn misdirected_request421(message: &str) -> Self {
        ApiError::new(StatusCode::GONE.as_u16(), message).set_status(421)
    }
    /// Unprocessable Entity. (status=422)
    pub fn unprocessable_entity422(message: &str) -> Self {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY.as_u16(), message).set_status(422)
    }
    /// Locked. (status=423)
    pub fn locked423(message: &str) -> Self {
        ApiError::new(StatusCode::LOCKED.as_u16(), message).set_status(423)
    }
    /// Failed Dependency. (status=424)
    pub fn failed_dependency424(message: &str) -> Self {
        ApiError::new(StatusCode::FAILED_DEPENDENCY.as_u16(), message).set_status(424)
    }
    /// Upgrade Required. (status=426)
    pub fn upgrade_required426(message: &str) -> Self {
        ApiError::new(StatusCode::UPGRADE_REQUIRED.as_u16(), message).set_status(426)
    }
    /// Precondition Required. (status=428)
    pub fn precondition_required428(message: &str) -> Self {
        ApiError::new(StatusCode::PRECONDITION_REQUIRED.as_u16(), message).set_status(428)
    }
    /// Too Many Requests. (status=429)
    pub fn too_many_requests429(message: &str) -> Self {
        ApiError::new(StatusCode::TOO_MANY_REQUESTS.as_u16(), message).set_status(429)
    }
    /// Request Header Fields Too Large. (status=431)
    pub fn request_header_fields_too_large431(message: &str) -> Self {
        ApiError::new(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE.as_u16(), message).set_status(431)
    }
    /// Unavailable For Legal Reasons. (status=451)
    pub fn unavailable_for_legal_reasons451(message: &str) -> Self {
        ApiError::new(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS.as_u16(), message).set_status(451)
    }

    /// Internal Server Error. (status=500)
    pub fn internal_server_error500(message: &str) -> ApiError {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), message).set_status(500)
    }
    /// Not Implemented. (status=501)
    pub fn not_implemented501(message: &str) -> Self {
        ApiError::new(StatusCode::NOT_IMPLEMENTED.as_u16(), message).set_status(501)
    }
    /// Bad Gateway. (status=502)
    pub fn bad_gateway502(message: &str) -> Self {
        ApiError::new(StatusCode::BAD_GATEWAY.as_u16(), message).set_status(502)
    }
    /// Service Unavailable. (status=503)
    pub fn service_unavailable503(message: &str) -> Self {
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE.as_u16(), message).set_status(503)
    }
    /// Gateway Timeout. (status=504)
    pub fn gateway_timeout504(message: &str) -> Self {
        ApiError::new(StatusCode::GATEWAY_TIMEOUT.as_u16(), message).set_status(504)
    }
    /// HTTP Version Not Supported. (status=505)
    pub fn http_version_not_supported505(message: &str) -> Self {
        ApiError::new(StatusCode::HTTP_VERSION_NOT_SUPPORTED.as_u16(), message).set_status(505)
    }
    /// Variant Also Negotiates. (status=506) (when process is blocked).
    pub fn variant_also_negotiates506(message: &str) -> ApiError {
        ApiError::new(StatusCode::VARIANT_ALSO_NEGOTIATES.as_u16(), message).set_status(506)
    }
    /// Insufficient Storage. (status=507) Error, (when querying the database)
    pub fn insufficient_storage507(message: &str) -> Self {
        ApiError::new(StatusCode::INSUFFICIENT_STORAGE.as_u16(), message).set_status(507)
    }
    /// Loop Detected. (status=508)
    pub fn loop_detected508(message: &str) -> Self {
        ApiError::new(StatusCode::LOOP_DETECTED.as_u16(), message).set_status(508)
    }
    /// Not Extended. (status=510)
    pub fn not_extended510(message: &str) -> ApiError {
        ApiError::new(StatusCode::NOT_EXTENDED.as_u16(), message).set_status(510)
    }
    /// Network Authentication Required. (status=511)
    pub fn network_authentication_required511(message: &str) -> Self {
        ApiError::new(StatusCode::NETWORK_AUTHENTICATION_REQUIRED.as_u16(), message).set_status(511)
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            // .insert_header(header::ContentType::json())
            .insert_header(header::ContentType(mime::APPLICATION_JSON))
            .insert_header((mime::CHARSET.as_str(), mime::UTF_8.as_str()))
            .json(self)
    }
}

pub fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
    assert_eq!(app_err_vec.len(), msgs.len());
    for (idx, msg) in msgs.iter().enumerate() {
        let app_err = app_err_vec.get(idx).unwrap();
        assert_eq!(app_err.code, code);
        assert_eq!(app_err.message, msg.to_string());
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body,
        http::header::{HeaderValue, CONTENT_TYPE},
    };
    use serde_json::json;

    use super::*;

    pub const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** ApiError::new() **

    #[actix_web::test]
    async fn test_create_error_and_convert_to_string() {
        let text = "Error text 400.";
        let err = ApiError::new(400, &text);
        let json = json!({ "code": code_to_str(StatusCode::BAD_REQUEST), "message": text });
        assert_eq!(err.to_string(), json.to_string());
    }
    #[actix_web::test]
    async fn test_create_error_with_400_and_text() {
        let text = "Error text 400.";
        let err = ApiError::new(400, &text);

        assert_eq!(err.status, StatusCode::BAD_REQUEST.as_u16());
        assert_eq!(err.code, code_to_str(StatusCode::BAD_REQUEST));
        assert_eq!(err.message, text);
        assert!(err.params.is_empty());
    }
    #[actix_web::test]
    async fn test_create_error_with_0_and_text() {
        let text = "Error text 0.";
        let err = ApiError::new(0, &text);

        assert_eq!(err.status, ApiError::default_status());
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(err.code, code_to_str(StatusCode::INTERNAL_SERVER_ERROR));
        assert_eq!(err.message, text);
        assert!(err.params.is_empty());
    }
    #[actix_web::test]
    async fn test_create_error_with_0_and_empty_string() {
        let text = "";
        let err = ApiError::new(0, &text);

        assert_eq!(err.status, ApiError::default_status());
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(err.code, code_to_str(StatusCode::INTERNAL_SERVER_ERROR));
        assert_eq!(err.message, ApiError::default_message());
        assert_eq!(err.message, MSG_INTER_SRV_ERROR);
        assert!(err.params.is_empty());
    }

    // ** ApiError::u16_to_status_code_or_default **

    #[actix_web::test]
    async fn test_u16_to_status_code_or_default() {
        let status_code0 = ApiError::u16_to_status_code_or_default(99);
        assert_eq!(status_code0, StatusCode::INTERNAL_SERVER_ERROR);
        let status_code1 = ApiError::u16_to_status_code_or_default(100);
        assert_eq!(status_code1, StatusCode::CONTINUE);
        let status_code2 = ApiError::u16_to_status_code_or_default(200);
        assert_eq!(status_code2, StatusCode::OK);
        let status_code3 = ApiError::u16_to_status_code_or_default(300);
        assert_eq!(status_code3, StatusCode::MULTIPLE_CHOICES);
        let status_code4 = ApiError::u16_to_status_code_or_default(400);
        assert_eq!(status_code4, StatusCode::BAD_REQUEST);
        let status_code5 = ApiError::u16_to_status_code_or_default(500);
        assert_eq!(status_code5, StatusCode::INTERNAL_SERVER_ERROR);
        let status_code6 = ApiError::u16_to_status_code_or_default(1000);
        assert_eq!(status_code6, StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ** ApiError::default_status() **

    #[actix_web::test]
    async fn test_default_status() {
        let status1 = ApiError::default_status();
        assert_eq!(status1, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        let status2 = ApiError::u16_to_status_code_or_default(0).as_u16();
        assert_eq!(status2, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
    }

    // ** ApiError::default_params() **

    #[actix_web::test]
    async fn test_default_params() {
        let params = ApiError::default_params();
        assert_eq!(params.len(), 0);
    }

    // ** ApiError::default_message() **

    #[actix_web::test]
    async fn test_default_message() {
        let message = ApiError::default_message();
        assert_eq!(message, MSG_INTER_SRV_ERROR);
    }

    // ** ApiError::set_status() **
    // ** ApiError::status_code() **

    #[actix_web::test]
    async fn test_create_error_with_401_and_modify_to_402() {
        let text = "Error text 401.";
        let mut err = ApiError::new(401, &text);

        assert_eq!(err.status, StatusCode::UNAUTHORIZED.as_u16());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(err.message, text);

        err.set_status(402);

        assert_eq!(err.status, StatusCode::PAYMENT_REQUIRED.as_u16());
        assert_eq!(err.status_code(), StatusCode::PAYMENT_REQUIRED);
        assert_eq!(err.code, code_to_str(StatusCode::PAYMENT_REQUIRED));
        assert_eq!(err.message, text);
    }

    // ** ApiError::add_param() **

    #[actix_web::test]
    async fn test_create_error_with_401_and_text_and_param() {
        let text = "Error text 401.";
        let param1 = borrow::Cow::Borrowed("param1");
        let json = json!({ "key1": "value1", "key2": 10 });
        let err = ApiError::new(401, &text).add_param(param1.clone(), &json);

        assert_eq!(err.status, StatusCode::UNAUTHORIZED.as_u16());
        assert_eq!(err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(err.message, text);
        assert_eq!(err.params.get(&param1).unwrap(), &json);
    }
    #[actix_web::test]
    async fn test_create_error_with_401_and_text_and_params() {
        let text = "Error text 401.";
        let param1 = borrow::Cow::Borrowed("param1");
        let json1 = json!({ "key1": "value1", "key2": 2 });
        let param2 = borrow::Cow::Borrowed("param2");
        let json2 = json!({ "key11": "value11", "key12": 12 });
        let err = ApiError::new(401, &text)
            .add_param(param1.clone(), &json1)
            .add_param(param2.clone(), &json2);

        assert_eq!(err.status, StatusCode::UNAUTHORIZED.as_u16());
        assert_eq!(err.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(err.message, text);
        assert_eq!(err.params.get(&param1).unwrap(), &json1);
        assert_eq!(err.params.get(&param2).unwrap(), &json2);
    }

    // ** ApiError::to_response() **

    #[actix_web::test]
    async fn test_to_response() {
        let text1 = "Error text_1 401.";
        let err1 = ApiError::new(401, &text1);
        let text2 = "Error text_2 402.";
        let err2 = ApiError::new(402, &text2);
        let response = ApiError::to_response(&[err1, err2]);

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 401
        #[rustfmt::skip]
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        #[rustfmt::skip]
        assert_eq!(response.headers().get(mime::CHARSET.as_str()).unwrap(), HeaderValue::from_static("utf-8"));

        let body = body::to_bytes(response.into_body()).await.unwrap();
        let api_errs: Vec<ApiError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(api_errs.len(), 2);
        let api_err1 = api_errs.get(0).unwrap();
        assert_eq!(api_err1.code, code_to_str(StatusCode::UNAUTHORIZED));
        assert_eq!(api_err1.message, text1);
        let api_err2 = api_errs.get(1).unwrap();
        assert_eq!(api_err2.code, code_to_str(StatusCode::PAYMENT_REQUIRED));
        assert_eq!(api_err2.message, text2);
    }
}

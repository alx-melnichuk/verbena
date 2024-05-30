// 200 Ok - Request processed successfully.
// 201 Created - A new entry has been created.
// 204 No Content - Data not found.

// 401 Unauthorized (user_auth_controller, user_registr_controller)
pub const CD_UNAUTHORIZED: &str = "Unauthorized"; /*+*/
// 401 Unauthorized - Authorization token is missing. (authentication, user_auth_controller, user_controller)
pub const MSG_MISSING_TOKEN: &str = "token_missing"; /*+*/
// 401 Unauthorized - Error when decoding token or expired token (authentication, user_auth_controller, user_registr_controller)
pub const MSG_INVALID_OR_EXPIRED_TOKEN: &str = "invalid_or_expired_token"; /*+*/
// 401 Unauthorized - User's "num" does not match "num" from token. (authentication, user_auth_controller)
pub const MSG_UNACCEPTABLE_TOKEN_NUM: &str = "unacceptable_token_num"; /*+*/

// 403 Forbidden - (authentication, user_auth_controller)
pub const CD_FORBIDDEN: &str = "Forbidden";
// 403 Forbidden - Access denied - insufficient rights (authentication)
pub const MSG_ACCESS_DENIED: &str = "access_is_denied";

// 404 Not Found (user_registr_controller)
pub const CD_NOT_FOUND: &str = "NotFound";

// 406 Not Acceptable (user_auth_controller)
pub const CD_NOT_ACCEPTABLE: &str = "NotAcceptable"; /*+*/
// 406 Not Acceptable - There is no session for this user. (authentication, user_auth_controller)
pub const MSG_SESSION_NOT_EXIST: &str = "session_not_exist"; /*+*/

// 409 Conflict (user_auth_controller, user_registr_controller)
pub const CD_CONFLICT: &str = "Conflict"; /*+*/

// 413 Content too large // The request object exceeds the limits defined by the server.
pub const CD_CONTENT_TOO_LARGE: &str = "ContentTooLarge"; /*+*/

// 415 Unsupported Media Type (stream_controller)
pub const CD_UNSUPPORTED_TYPE: &str = "UnsupportedType"; /*+*/
// 415 Unsupported Media Type - The specified type could not be converted. (stream_controller)
pub const MSG_PARSING_TYPE_NOT_SUPPORTED: &str = "parsing_type_not_supported"; /*+*/

// 416 Requested Range Not Satisfiable
pub const CD_RANGE_NOT_SATISFIABLE: &str = "RangeNotSatisfiable";

// 417 Expectation Failed (user_auth_controller, user_registr_controller)
pub const CD_VALIDATION: &str = "Validation"; /*+*/

// 422 Unprocessable Entity (user_auth_controller, user_registr_controller)
pub const CD_UNPROCESSABLE_ENTITY: &str = "UnprocessableEntity";

// 500 Internal Server Error (user_registr_controller)
pub const CD_INTERNAL_ERROR: &str = "InternalServerError";

// 506 Variant Also Negotiates /*+*/
pub const CD_BLOCKING: &str = "Blocking"; /*+*/
// 506 Variant Also Negotiates - Error web::block for waiting for synchronous operations to complete. /*+*/
pub const MSG_BLOCKING: &str = "error_waiting_for_operations"; /*+*/

// 507 Insufficient Storage /*+*/
pub const CD_DATABASE: &str = "Database"; /*+*/
// 507 Insufficient Storage - An error occurred while executing a database query. /*+*/
pub const MSG_DATABASE: &str = "database_query_error"; /*+*/

// 510 Not Extended (user_registr_controller)
pub const CD_NOT_EXTENDED: &str = "NotExtended"; /*+*/

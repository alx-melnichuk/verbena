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

// 413 Content too large
// The request object exceeds the limits defined by the server. (status=413)
pub const CD_CONTENT_TOO_LARGE: &str = "ContentTooLarge"; /*+*/

// 415 Unsupported Media Type (stream_controller)
pub const CD_UNSUPPORTED_TYPE: &str = "UnsupportedType"; /*+*/
// 415 Unsupported Media Type - The specified type could not be converted. (stream_controller)
pub const MSG_PARSING_TYPE_NOT_SUPPORTED: &str = "parsing_type_not_supported"; /*+*/

// 417 Expectation Failed (user_auth_controller, user_registr_controller)
pub const CD_VALIDATION: &str = "Validation"; /*+*/

// 422 Unprocessable Entity (user_auth_controller, user_registr_controller)
pub const CD_UNPROCESSABLE_ENTITY: &str = "UnprocessableEntity";

// 500 Internal Server Error (user_registr_controller)
pub const CD_INTER_ERROR: &str = "InternalServerError";
pub const CD_INTERNAL_SERVER_ERROR: &str = "InternalServerError"; /* # del */

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

// OLD

// 400 Bad Request

pub const CD_NO_ACCESS_TO_STREAMS: &str = "NoAccessToStreams";
pub const MSG_NO_ACCESS_TO_STREAMS: &str = "No access to other user's streams.";
// Error when end date is less than start date
pub const CD_FINISH_LESS_START: &str = "FinishLessStart";
pub const MSG_FINISH_LESS_START: &str = "The finish date is less than start date.";
// Error when the finish period is greater than the maximum.
pub const CD_FINISH_GREATER_MAX: &str = "FinishGreaterMax";
pub const MSG_FINISH_GREATER_MAX: &str = "The finish date of the search period exceeds the limit";

//
//
pub const CD_NO_CONFIRM: &str = "No Confirmation";
pub const MSG_CONFIRM_NOT_FOUND: &str = "Confirmation not found!";

// 500
pub const CD_INTER_SRV_ERROR: &str = "InternalServerError";
pub const MSG_INTER_SRV_ERROR: &str = "internal_server_error";

// Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

// 200 Ok - Request processed successfully.
// 201 Created - A new entry has been created.
// 204 No Content - Data not found.

// 401 Unauthorized
pub const CD_UNAUTHORIZED: &str = "Unauthorized"; /*+*/
// 401 Unauthorized - Authorization token is missing. (authentication)
pub const MSG_MISSING_TOKEN: &str = "token_missing"; /*+*/
// 401 Unauthorized - Error when decoding token or expired token (authentication)
pub const MSG_INVALID_OR_EXPIRED_TOKEN: &str = "invalid_or_expired_token"; /*+*/
// 401 Unauthorized - User's "num" does not match "num" from token. (authentication, user_auth_controller)
pub const MSG_UNACCEPTABLE_TOKEN_NUM: &str = "unacceptable_token_num"; /*+*/
// 401 Unauthorized - According to "user_id" in the token, the user was not found. (authentication)
// The token "ID" value is unacceptable.
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id"; /*+*/

// 403 Forbidden - (authentication)
pub const CD_FORBIDDEN: &str = "Forbidden";
// 403 Forbidden - Access denied - insufficient rights (authentication)
pub const MSG_ACCESS_DENIED: &str = "access_is_denied";

// 406 Not Acceptable
pub const CD_NOT_ACCEPTABLE: &str = "NotAcceptable"; /*+*/
// 406 Not Acceptable - There is no session for this user. (authentication, user_auth_controller)
pub const MSG_SESSION_NOT_EXIST: &str = "session_not_exist"; /*+*/

// 409 Conflict (user_auth_controller)
pub const CD_CONFLICT: &str = "Conflict"; /*+*/
// 409 Conflict - Error encoding web token. (user_auth_controller)
pub const MSG_JSON_WEB_TOKEN_ENCODE: &str = "json_web_token_encode"; /*+*/
// 409 Conflict - Error decoding web token. /*+*/
pub const MSG_JSON_WEB_TOKEN_DECODE: &str = "json_web_token_decode"; /*+*/

// 415 Unsupported Media Type /*+*/
pub const CD_PARSE_ERROR: &str = "ParseError"; /*+*/
// 415 Unsupported Media Type - The specified type could not be converted. /*+*/
pub const MSG_FAILED_CONVERSION: &str = "failed_conversion"; /*+*/

// 417 Expectation Failed /*+*/
pub const CD_VALIDATION: &str = "Validation"; /*+*/

// 422 Unprocessable Entity (user_registr_controller)
pub const CD_UNPROCESSABLE_ENTITY: &str = "UnprocessableEntity"; /*+*/

// 500 Internal Server Error /*+*/
pub const CD_INTER_ERROR: &str = "InternalServerError";
pub const CD_INTERNAL_SERVER_ERROR: &str = "InternalServerError"; /* # del */
// 500 Internal Server Error - Error creating password hash. (user_registr_controller)
pub const MSG_ERROR_HASHING_PASSWORD: &str = "error_hashing_password";

// 506 Variant Also Negotiates /*+*/
pub const CD_BLOCKING: &str = "Blocking"; /*+*/
// 506 Variant Also Negotiates - Error web::block for waiting for synchronous operations to complete. /*+*/
pub const MSG_BLOCKING: &str = "error_waiting_for_operations"; /*+*/

// 507 Insufficient Storage /*+*/
pub const CD_DATABASE: &str = "Database"; /*+*/
// 507 Insufficient Storage - An error occurred while executing a database query. /*+*/
pub const MSG_DATABASE: &str = "database_query_error"; /*+*/

// 510 Not Extended /*+*/
pub const CD_NOT_EXTENDED: &str = "NotExtended"; /*+*/

// OLD

// 400 Bad Request

pub const CD_INVALID_TAGS_FIELD: &str = "InvalidTagsField";
pub const MSG_INVALID_TAGS_FIELD: &str = "Error deserializing the \"tags\" field:";
// Uploading Image Files
// Mime file type is not valid.
pub const CD_INVALID_FILE_TYPE: &str = "InvalidFileType";
pub const MSG_INVALID_IMAGE_FILE: &str = "Invalid image file type.";
// The file size does not meet the maximum size.
pub const CD_INVALID_FILE_SIZE: &str = "InvalidFileSize";
pub const MSG_INVALID_FILE_SIZE: &str = "The file size exceeds the max size.";
// Error uploading file
pub const CD_ERROR_FILE_UPLOAD: &str = "ErrorUploadFile";
pub const MSG_ERROR_FILE_UPLOAD: &str = "Error while upload file:";
// Error convert file
pub const CD_ERROR_CONVERT_FILE: &str = "ErrorConvertFile";
pub const MSG_ERROR_CONVERT_FILE: &str = "Error converting file:";
// Error when receiving another user's streams.
pub const CD_NO_ACCESS_TO_STREAMS: &str = "NoAccessToStreams";
pub const MSG_NO_ACCESS_TO_STREAMS: &str = "No access to other user's streams.";
// Error when end date is less than start date
pub const CD_FINISH_LESS_START: &str = "FinishLessStart";
pub const MSG_FINISH_LESS_START: &str = "The finish date is less than start date.";
// Error when the finish period is greater than the maximum.
pub const CD_FINISH_GREATER_MAX: &str = "FinishGreaterMax";
pub const MSG_FINISH_GREATER_MAX: &str = "The finish date of the search period exceeds the limit";

// 404 Not Found
pub const CD_NOT_FOUND: &str = "NotFound";
// Registration record not found.
pub const MSG_REGISTR_NOT_FOUND: &str = "registration_not_found";
// Recovery record not found.
pub const MSG_RECOVERY_NOT_FOUND: &str = "recovery_not_found";
// User not found.
pub const MSG_USER_NOT_FOUND: &str = "user_not_found";

//
//
// User_ID from the header does not match the user_ID from the parameters
pub const CD_UNALLOWABLE_TOKEN: &str = "UnallowableToken";
pub const MSG_UNALLOWABLE_TOKEN: &str = "oken value is unallowable";

pub const CD_NO_CONFIRM: &str = "No Confirmation";
pub const MSG_CONFIRM_NOT_FOUND: &str = "Confirmation not found!";

//
pub const MSG_STREAM_NOT_FOUND_BY_ID: &str = "The stream with the specified ID was not found.";

// 500
pub const CD_INTER_SRV_ERROR: &str = "InternalServerError";
pub const MSG_INTER_SRV_ERROR: &str = "internal_server_error";

// Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

// Error creating password hash.
pub const CD_HASHING_PASSWD: &str = "HashingPassword";
// Error creating token.
pub const CD_JSON_WEB_TOKEN: &str = "JsonWebToken";

// pub const MSG_SERVER_ERROR: &str = "An unexpected internal server error occurred.";

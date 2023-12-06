// 400 Bad Request
pub const CD_VALIDATION: &str = "Validation";

// 401 Unauthorized
pub const CD_UNAUTHORIZED: &str = "Unauthorized";
pub const CD_MISSING_TOKEN: &str = "MissingToken";
pub const MSG_MISSING_TOKEN: &str = "Token value not provided";

// 403 Forbidden
pub const CD_FORBIDDEN: &str = "Forbidden";
// Error when decoding token or expired token
pub const MSG_INVALID_OR_EXPIRED_TOKEN: &str = "invalid_or_expired_token";
// According to "user_id" in the token, the user was not found // "The token "ID" value is unacceptable."
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id";
// According to "num" in the token, the user was not found // "The token "NUM" value is unacceptable."
pub const MSG_UNACCEPTABLE_TOKEN_NUM: &str = "unacceptable_token_num";
//
// User_ID from the header does not match the user_ID from the parameters
pub const CD_UNALLOWABLE_TOKEN: &str = "UnallowableToken";
pub const MSG_UNALLOWABLE_TOKEN: &str = "oken value is unallowable";

// pub const CD_WRONG_TOKEN: &str = "WrongToken";
// pub const MSG_WRONG_TOKEN: &str = "Wrong token value";

// pub const CD_BAD_TOKEN: &str = "Bad Token";
// pub const MSG_BAD_TOKEN: &str = "Bad token value";

pub const CD_PERMISSION_DENIED: &str = "PermissionDenied";
pub const MSG_PERMISSION_DENIED: &str = "You are not allowed to perform this action";

// 404
pub const CD_NOT_FOUND: &str = "NotFound";
// Registration record not found
pub const MSG_REGISTR_NOT_FOUND: &str = "registration_not_found";
// Recovery record not found
pub const MSG_RECOVERY_NOT_FOUND: &str = "recovery_not_found";

pub const CD_NO_CONFIRM: &str = "No Confirmation";
pub const MSG_CONFIRM_NOT_FOUND: &str = "Confirmation not found!";

pub const MSG_NOT_FOUND_BY_ID: &str = "The user with the specified ID was not found.";
pub const MSG_NOT_FOUND_BY_EMAIL: &str = "The user with the specified email was not found.";

// 409
pub const CD_CONFLICT: &str = "Conflict";

// 500
pub const CD_INTER_SRV_ERROR: &str = "InternalServerError";
pub const MSG_INTER_SRV_ERROR: &str = "internal_server_error";
// Error web::block for waiting for database query to complete.
pub const CD_BLOCKING: &str = "Blocking";
// An error occurred while executing a database query.
pub const CD_DATABASE: &str = "Database";
// Error checking hash value.
pub const MSG_INVALID_HASH: &str = "invalid_hash";
// Error encoding web token.
pub const MSG_JSON_WEB_TOKEN_ENCODE: &str = "json_web_token_encode";
// Error decoding web token.
pub const MSG_JSON_WEB_TOKEN_DECODE: &str = "json_web_token_decode";
// There is no session for this user.
pub const MSG_SESSION_NOT_EXIST: &str = "session_not_exist";
// Error when sending email
pub const MSG_ERROR_SENDING_EMAIL: &str = "error_sending_email";
// Error creating password hash.
pub const MSG_ERROR_HASHING_PASSWORD: &str = "error_hashing_password";
// Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

// Error creating password hash.
pub const CD_HASHING_PASSWD: &str = "HashingPassword";
// Error creating token.
pub const CD_JSON_WEB_TOKEN: &str = "JsonWebToken";

// pub const MSG_SERVER_ERROR: &str = "An unexpected internal server error occurred.";

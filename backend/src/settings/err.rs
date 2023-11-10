// 401 Unauthorized
pub const CD_MISSING_TOKEN: &str = "MissingToken";
pub const MSG_MISSING_TOKEN: &str = "Token value not provided";

// 403 Forbidden
pub const CD_FORBIDDEN: &str = "Forbidden";
// Error when decoding token or expired token
pub const MSG_INVALID_OR_EXPIRED_TOKEN: &str = "invalid_or_expired_token";
// According to AD in the token, the user was not found
pub const CD_UNACCEPTABLE_TOKEN: &str = "UnacceptableToken";
pub const MSG_UNACCEPTABLE_TOKEN: &str = "Token value is unacceptable";
// User_ID from the header does not match the user_ID from the parameters
pub const CD_UNALLOWABLE_TOKEN: &str = "UnallowableToken";
pub const MSG_UNALLOWABLE_TOKEN: &str = "oken value is unallowable";

// pub const CD_WRONG_TOKEN: &str = "WrongToken";
// pub const MSG_WRONG_TOKEN: &str = "Wrong token value";

// pub const CD_BAD_TOKEN: &str = "Bad Token";
// pub const MSG_BAD_TOKEN: &str = "Bad token value";

// wrong token

pub const CD_PERMISSION_DENIED: &str = "PermissionDenied";
pub const MSG_PERMISSION_DENIED: &str = "You are not allowed to perform this action";

// 404
pub const CD_NO_CONFIRM: &str = "No Confirmation";
pub const MSG_CONFIRM_NOT_FOUND: &str = "Confirmation not found!";

pub const CD_NOT_FOUND: &str = "NotFound";
pub const MSG_NOT_FOUND_BY_ID: &str = "The user with the specified ID was not found.";
pub const MSG_NOT_FOUND_BY_EMAIL: &str = "The user with the specified email was not found.";

// 409
pub const CD_CONFLICT: &str = "Conflict";

// 500
// Error web::block for waiting for database query to complete.
pub const CD_BLOCKING: &str = "Blocking";
// An error occurred while executing a database query.
pub const CD_DATABASE: &str = "Database";
// Error creating password hash.
pub const CD_HASHING_PASSWD: &str = "HashingPassword";
// Error creating token.
pub const CD_JSON_WEB_TOKEN: &str = "JsonWebToken";

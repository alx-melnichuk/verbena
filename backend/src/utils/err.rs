// 401 Unauthorized
pub const CD_MISSING_TOKEN: &str = "MissingToken";
pub const MSG_MISSING_TOKEN: &str = "Token value not provided";

// 403 Forbidden
// Error when decoding token or expired token
pub const CD_INVALID_TOKEN: &str = "InvalidToken";
pub const MSG_INVALID_TOKEN: &str = "Invalid or expired token";
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

// 500
// Error web::block for waiting for database query to complete.
pub const CD_BLOCKING: &str = "Blocking";
// An error occurred while executing a database query.
pub const CD_DATABASE: &str = "Database";
// Error creating password hash.
pub const CD_HASHING_PASSWD: &str = "HashingPassword";
// Error creating token.
pub const CD_JSONWEBTOKEN: &str = "jsonwebtoken";

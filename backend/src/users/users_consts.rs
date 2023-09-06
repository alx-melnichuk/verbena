// Entity "user"

pub const NICKNAME_NAME: &str = "nickname";
pub const NICKNAME_MIN: usize = 3;
pub const NICKNAME_MAX: usize = 255;

pub const EMAIL_NAME: &str = "email";
pub const EMAIL_MIN: usize = 3;
pub const EMAIL_MAX: usize = 255;

pub const PASSWORD_NAME: &str = "password";
pub const PASSWORD_MIN: usize = 5;
pub const PASSWORD_MAX: usize = 255;

pub const ERR_CODE_REQUIRED: &str = "required";
pub const ERR_MSG_REQUIRED: &str = "The field is required.";

pub const ERR_CODE_MIN_LENGTH: &str = "min_length";
pub const ERR_MSG_MIN_LENGTH: &str = "failure to meet the minimum field length.";

pub const ERR_CODE_MAX_LENGTH: &str = "max_length";
pub const ERR_MSG_MAX_LENGTH: &str = "failure to meet the maximum field length.";

pub const ERR_CODE_MODEL_IS_EMPTY: &str = "model_is_empty";
pub const ERR_MSG_MODEL_IS_EMPTY: &str = "The data model does not contain information to update.";

// Error messages
pub const ERR_CASTING_TO_TYPE: &str = "Error casting value \"{}\" to type {}.";
pub const ERR_NOT_FOUND_BY_ID: &str = "User with ID \"{}\" was not found.";
pub const ERR_INCORRECT_VALUE: &str = "Incorrect value of the \"{}\" parameter.";

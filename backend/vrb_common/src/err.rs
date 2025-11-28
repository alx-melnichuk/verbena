// 200 Ok - Request processed successfully.
// 201 Created - A new entry has been created.
// 204 No Content - Data not found.

// 400 Bad Request - The parameter value is not defined.
pub const MSG_PARAMETER_NOT_DEFINED: &str = "parameter_not_defined";

// 401 Unauthorized
pub const MSG_WRONG_NICKNAME_EMAIL: &str = "nickname_or_email_incorrect";
// 401 Unauthorized
pub const MSG_PASSWORD_INCORRECT: &str = "password_incorrect";
// 401(a) Unauthorized - Authorization token is missing. (authentication, profile_auth_controller, user_controller)
pub const MSG_MISSING_TOKEN: &str = "token_missing";
// 401(b) Unauthorized - Error when decoding token or expired token (authentication, profile_auth_controller, user_registr_controller)
pub const MSG_INVALID_OR_EXPIRED_TOKEN: &str = "invalid_or_expired_token";
// 401(c) Unauthorized - User's "num" does not match "num" from token. (authentication, profile_auth_controller)
pub const MSG_UNACCEPTABLE_TOKEN_NUM: &str = "unacceptable_token_num";
// 401(d) Unauthorized - According to "user_id" in the token, the user was not found.
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id";

// 403 Forbidden - Access denied - insufficient rights (authentication)
pub const MSG_ACCESS_DENIED: &str = "access_denied";
// 403 Forbidden - There is a block on sending messages
pub const MSG_BLOCK_ON_SEND_MESSAGES: &str = "block_on_sending_messages";
// 403 Forbidden - Stream owner rights are missing
pub const MSG_STREAM_OWNER_RIGHTS_MISSING: &str = "stream_owner_rights_missing";

// 404 Not Found - Stream not found.
pub const MSG_STREAM_NOT_FOUND: &str = "stream_not_found";
// 404 Not Found - User not found.
pub const MSG_USER_NOT_FOUND: &str = "user_not_found";
// 404 Not Found - ChatMessage not found
pub const MSG_CHAT_MESSAGE_NOT_FOUND: &str = "chat_message_not_found";

// 406 Not Acceptable - There is no session for this user. (authentication, user_authent_controller)
pub const MSG_SESSION_NOT_FOUND: &str = "session_not_found";
// 406 Not Acceptable - There is no profile for this user. (user_authent_controller)
pub const MSG_PROFILE_NOT_FOUND: &str = "profile_not_found";
// 406 Not Acceptable - None of the parameters are specified.
pub const MSG_PARAMS_NOT_SPECIFIED: &str = "params_not_specified";
// 406 Not Acceptable - The parameter value is unacceptable.
pub const MSG_PARAMETER_UNACCEPTABLE: &str = "parameter_value_unacceptable";
// 406 Not Acceptable - There was no 'join' command.
pub const MSG_THERE_WAS_NO_JOIN: &str = "was_no_join_command";
// 406 Not Acceptable - There was no 'name' command
pub const MSG_THERE_WAS_NO_NAME: &str = "was_no_name_command";

// 409 Conflict - Error checking hash value.
pub const MSG_INVALID_HASH: &str = "invalid_hash";
// 409 Conflict - The specified "email" is already registered.
pub const MSG_EMAIL_ALREADY_USE: &str = "email_already_use";
// 409 Conflict - The specified "nickname" is already registered.
pub const MSG_NICKNAME_ALREADY_USE: &str = "nickname_already_use";
// 409 Conflict - There was already a 'join' to the room
pub const MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM: &str = "was_already_join_to_room";
// 409 Conflict - This stream is not active
pub const MSG_STREAM_NOT_ACTIVE: &str = "stream_not_active";
// 409 Conflict - Error encoding web token.
pub const MSG_JSON_WEB_TOKEN_ENCODE: &str = "json_web_token_encode";

// 413 Content too large - File size exceeds max.
pub const MSG_INVALID_FILE_SIZE: &str = "invalid_file_size";

// 415 Unsupported Media Type - Uploading Image Files. Mime file type is not valid.
pub const MSG_INVALID_FILE_TYPE: &str = "invalid_file_type";

// 416 Requested Range Not Satisfiable - The specified type could not be converted.
pub const MSG_PARSING_TYPE_NOT_SUPPORTED: &str = "parsing_type_not_supported";

// 417 Validation
// 417 Expectation Failed - No fields specified for update.
pub const MSG_NO_FIELDS_TO_UPDATE: &str = "no_fields_to_update";
// 417 Expectation Failed - One of the optional fields must be present.
pub const MSG_ONE_OPTIONAL_FIELDS_MUST_PRESENT: &str = "one_optional_fields_must_present";

// 500 Internal Server Error - Error creating password hash. (user_registr_controller, user_registr_controller)
pub const MSG_ERROR_HASHING_PASSWORD: &str = "error_hashing_password";
// 500 Internal Server Error - Error uploading file
pub const MSG_ERROR_UPLOAD_FILE: &str = "error_upload_file";

// 506 Blocking
// 506 Variant Also Negotiates - Error web::block for waiting for synchronous operations to complete.
pub const MSG_BLOCKING: &str = "error_waiting_for_operations";

// 507 Database
// 507 Insufficient Storage - An error occurred while executing a database query.
pub const MSG_DATABASE: &str = "database_query_error";

// 510 Not Extended - Error while converting file.
pub const MSG_ERROR_CONVERT_FILE: &str = "error_convert_file";
// 510 Not Extended - Error when sending email.
pub const MSG_ERROR_SENDING_EMAIL: &str = "error_sending_email";

use std::borrow::Cow::Borrowed;
use std::convert::From;

use crate::errors::AppError;
use crate::settings::err;

use super::upload_file::ErrUpload;

///
///  let app_error = AppError::from(err_upload);
///

impl From<ErrUpload> for AppError {
    fn from(err_upload: ErrUpload) -> Self {
        match err_upload {
            ErrUpload::InvalidFileSize { actual_size, max_size } => {
                let json = serde_json::json!({ "actualFileSize": actual_size, "maxFileSize": max_size });
                AppError::content_large413(err::MSG_INVALID_FILE_SIZE).add_param(Borrowed("invalidFileSize"), &json)
                // 413
            }
            #[rustfmt::skip]
            ErrUpload::InvalidFileType { actual_type, valid_types } => {
                let json = serde_json::json!({ "actualFileType": &actual_type, "validFileType": &valid_types });
                AppError::unsupported_type415(err::MSG_INVALID_FILE_TYPE).add_param(Borrowed("invalidFileType"), &json)
                // 415
            }
            ErrUpload::ErrorSavingFile { path_file, err } => {
                let message = format!("{}; {} - {}", err::MSG_ERROR_UPLOAD_FILE, &path_file, &err);
                AppError::internal_err500(&message)
                // 500
            }
        }
    }
}

use std::collections::HashMap;

use crate::users::users_consts::{
    ERR_CODE_MAX_LENGTH, ERR_CODE_MIN_LENGTH, ERR_CODE_REQUIRED, ERR_MSG_MAX_LENGTH,
    ERR_MSG_MIN_LENGTH, ERR_MSG_REQUIRED,
};

use super::errors::AppError;

pub const FIELD: &str = "field";
pub const MIN_LENGTH: &str = "minLength";
pub const MAX_LENGTH: &str = "maxLength";

pub struct Validations {}

impl Validations {
    pub fn required(value: &str, field_name: &str) -> Vec<AppError> {
        let mut result = Vec::new();
        if value.len() == 0 {
            result.push(AppError::InvalidField(
                ERR_CODE_REQUIRED.to_string(),
                ERR_MSG_REQUIRED.to_string(),
                HashMap::from([(FIELD.to_string(), field_name.to_string())]),
            ));
        }
        result
    }

    pub fn min_len(value: &str, min: usize, field_name: &str) -> Vec<AppError> {
        let mut result = vec![];
        if value.len() < min {
            result.push(AppError::InvalidField(
                ERR_CODE_MIN_LENGTH.to_string(),
                ERR_MSG_MIN_LENGTH.to_string(),
                HashMap::from([
                    (MIN_LENGTH.to_string(), min.to_string()),
                    (FIELD.to_string(), field_name.to_string()),
                ]),
            ));
        }
        result
    }

    pub fn max_len(value: &str, max: usize, field_name: &str) -> Vec<AppError> {
        let mut result = vec![];
        if value.len() > max {
            result.push(AppError::InvalidField(
                ERR_CODE_MAX_LENGTH.to_string(),
                ERR_MSG_MAX_LENGTH.to_string(),
                HashMap::from([
                    (MAX_LENGTH.to_string(), max.to_string()),
                    (FIELD.to_string(), field_name.to_string()),
                ]),
            ));
        }
        result
    }
}

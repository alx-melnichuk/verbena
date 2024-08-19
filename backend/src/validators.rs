use std::{borrow::Cow, collections::HashMap};

use chrono::{DateTime, SecondsFormat, Utc};
use email_address;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ValidationError {
    pub message: Cow<'static, str>,
    pub params: HashMap<Cow<'static, str>, Value>,
}

impl std::error::Error for ValidationError {}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl ValidationError {
    pub fn new<'a>(message: &'a str) -> Self {
        ValidationError {
            message: Cow::from(message.to_string()),
            params: HashMap::new(),
        }
    }
    pub fn add_param<'a, T: Serialize>(&mut self, name: Cow<'a, str>, val: &T) -> Self {
        self.params.insert(name.to_string().into(), to_value(val).unwrap());
        self.to_owned()
    }
}

pub fn msg_validation(validation_errors: &Vec<ValidationError>) -> String {
    validation_errors
        .iter()
        .map(|v| format!("{} # ", v.message.to_string()))
        .collect()
}

pub trait Validator {
    /// Check the model against the required conditions.
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
    /// filter the list of errors
    fn filter_errors(&self, errors: Vec<Option<ValidationError>>) -> Result<(), Vec<ValidationError>> {
        let result: Vec<ValidationError> = errors.into_iter().filter_map(|err| err).collect();
        if result.len() > 0 {
            return Err(result);
        }
        Ok(())
    }
}

pub struct ValidationChecks {}

impl ValidationChecks {
    /// Checking if a string is complete.
    pub fn required(value: &str, msg: &'static str) -> Result<(), ValidationError> {
        let len: usize = value.len();
        if len == 0 {
            let mut err = ValidationError::new(msg);
            let data = true;
            err.add_param(Cow::Borrowed("required"), &data);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the length of a string with a minimum value.
    pub fn min_length(value: &str, min: usize, msg: &'static str) -> Result<(), ValidationError> {
        let len: usize = value.len();
        if len < min {
            let mut err = ValidationError::new(msg);
            let json = serde_json::json!({ "actualLength": len, "requiredLength": min });
            err.add_param(Cow::Borrowed("minlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the length of a string with a maximum value.
    pub fn max_length(value: &str, max: usize, msg: &'static str) -> Result<(), ValidationError> {
        let len: usize = value.len();
        if max < len {
            let mut err = ValidationError::new(msg);
            let json = serde_json::json!({ "actualLength": len, "requiredLength": max });
            err.add_param(Cow::Borrowed("maxlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the amount of elements of a array with a minimum value.
    #[rustfmt::skip]
    pub fn min_amount(amount: usize, min: usize, msg: &'static str) -> Result<(), ValidationError> {
        if amount < min {
            let mut err = ValidationError::new(msg);
            let json =
                serde_json::json!({ "actualAmount": amount, "requiredAmount": min });
            err.add_param(Cow::Borrowed("minAmount"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the amount of elements of a array with a maximum value.
    #[rustfmt::skip]
    pub fn max_amount(amount: usize, max: usize, msg: &'static str) -> Result<(), ValidationError> {
        if max < amount {
            let mut err = ValidationError::new(msg);
            let json =
                serde_json::json!({ "actualAmount": amount, "requiredAmount": max });
            err.add_param(Cow::Borrowed("maxAmount"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking whether a string matches a regular expression.
    pub fn regexp(value: &str, reg_exp: &str, msg: &'static str) -> Result<(), ValidationError> {
        let regex = Regex::new(reg_exp).unwrap();
        let result = regex.captures(value);
        if result.is_none() {
            let mut err = ValidationError::new(msg);
            let json = serde_json::json!({ "actualValue": value, "requiredPattern": reg_exp });
            err.add_param(Cow::Borrowed("pattern"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking whether the string matches the email structure.
    pub fn email(value: &str, msg: &'static str) -> Result<(), ValidationError> {
        let is_valid = email_address::EmailAddress::is_valid(value);
        if !is_valid {
            let mut err = ValidationError::new(msg);
            let data = true;
            err.add_param(Cow::Borrowed("email"), &data);
            return Err(err);
        }
        Ok(())
    }
    /// Check date against minimum valid date.
    pub fn min_valid_date(
        value: &DateTime<Utc>,
        min_date_time: &DateTime<Utc>,
        msg: &'static str,
    ) -> Result<(), ValidationError> {
        if *value < *min_date_time {
            let mut err = ValidationError::new(msg);
            let value_s = (*value).to_rfc3339_opts(SecondsFormat::Millis, true);
            let min_date_time_s = (*min_date_time).to_rfc3339_opts(SecondsFormat::Millis, true);
            let json = serde_json::json!({ "actualDateTime": value_s, "minDateTime": min_date_time_s });
            err.add_param(Cow::Borrowed("minValidDateTime"), &json);
            return Err(err);
        }
        Ok(())
    }
    // Check for a list of valid values.
    pub fn valid_value(value: &str, valid_values: Vec<&str>, msg: &'static str) -> Result<(), ValidationError> {
        let res = valid_values.iter().position(|&val| val == value);
        if res.is_none() {
            let mut err = ValidationError::new(msg);
            let json = serde_json::json!({ "actualValue": value });
            err.add_param(Cow::Borrowed("invalid"), &json);
            return Err(err);
        } else {
            Ok(())
        }
    }
}

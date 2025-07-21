use std::{borrow::Cow, collections::HashMap};

use chrono::{DateTime, SecondsFormat, Utc};
use email_address;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value, Value};

pub const NM_NO_FIELDS_TO_UPDATE: &str = "noFieldsToUpdate";
pub const NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT: &str = "oneOptionalFieldMustPresent";

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
    pub fn required<'a>(value: &'a str, msg: &'a str) -> Result<(), ValidationError> {
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
    pub fn min_length<'a>(value: &'a str, min: usize, msg: &'a str) -> Result<(), ValidationError> {
        let len: usize = value.len();
        if len < min {
            let mut err = ValidationError::new(msg);
            let json = json!({ "actualLength": len, "requiredLength": min });
            err.add_param(Cow::Borrowed("minlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the length of a string with a maximum value.
    pub fn max_length<'a>(value: &'a str, max: usize, msg: &'a str) -> Result<(), ValidationError> {
        let len: usize = value.len();
        if len > max {
            let mut err = ValidationError::new(msg);
            let json = json!({ "actualLength": len, "requiredLength": max });
            err.add_param(Cow::Borrowed("maxlength"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the amount of elements of a array with a minimum value.
    #[rustfmt::skip]
    pub fn min_amount<'a>(amount: usize, min: usize, msg: &'a str) -> Result<(), ValidationError> {
        if amount < min {
            let mut err = ValidationError::new(msg);
            let json =
                json!({ "actualAmount": amount, "requiredAmount": min });
            err.add_param(Cow::Borrowed("minAmount"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking the amount of elements of a array with a maximum value.
    #[rustfmt::skip]
    pub fn max_amount<'a>(amount: usize, max: usize, msg: &'a str) -> Result<(), ValidationError> {
        if amount > max{
            let mut err = ValidationError::new(msg);
            let json =
                json!({ "actualAmount": amount, "requiredAmount": max });
            err.add_param(Cow::Borrowed("maxAmount"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking whether a string matches a regular expression.
    pub fn regexp<'a>(value: &'a str, reg_exp: &'a str, msg: &'a str) -> Result<(), ValidationError> {
        let regex = Regex::new(reg_exp).unwrap();
        let result = regex.is_match(value);
        if !result {
            let mut err = ValidationError::new(msg);
            let json = json!({ "actualValue": value, "requiredPattern": reg_exp });
            err.add_param(Cow::Borrowed("pattern"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Checking whether the string matches the email structure.
    pub fn email<'a>(value: &'a str, msg: &'a str) -> Result<(), ValidationError> {
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
    #[rustfmt::skip]
    pub fn min_valid_date<'a>(value: &DateTime<Utc>, min_date_time: &DateTime<Utc>, msg: &'a str) -> Result<(), ValidationError> {
        if *value < *min_date_time {
            let mut err = ValidationError::new(msg);
            let value_s = (*value).to_rfc3339_opts(SecondsFormat::Millis, true);
            let min_date_time_s = (*min_date_time).to_rfc3339_opts(SecondsFormat::Millis, true);
            let json = json!({ "actualDateTime": value_s, "minDateTime": min_date_time_s });
            err.add_param(Cow::Borrowed("minValidDateTime"), &json);
            return Err(err);
        }
        Ok(())
    }
    /// Check date against maximum valid date.
    #[rustfmt::skip]
    pub fn max_valid_date<'a>(value: &DateTime<Utc>, max_date_time: &DateTime<Utc>, msg: &'a str) -> Result<(), ValidationError> {
        if *value > *max_date_time {
            let mut err = ValidationError::new(msg);
            let value_s = (*value).to_rfc3339_opts(SecondsFormat::Millis, true);
            let max_date_time_s = (*max_date_time).to_rfc3339_opts(SecondsFormat::Millis, true);
            let json = json!({ "actualDateTime": value_s, "maxDateTime": max_date_time_s });
            err.add_param(Cow::Borrowed("maxValidDateTime"), &json);
            return Err(err);
        }
        Ok(())
    }
    // Check for a list of valid values.
    pub fn valid_value<'a>(value: &'a str, valid_values: &[&'a str], msg: &'a str) -> Result<(), ValidationError> {
        let res = valid_values.iter().position(|&val| val == value);
        if res.is_none() {
            let mut err = ValidationError::new(msg);
            let json = json!({ "actualValue": value });
            err.add_param(Cow::Borrowed("invalid"), &json);
            return Err(err);
        } else {
            Ok(())
        }
    }
    /// Checking for at least one required field.
    #[rustfmt::skip]
    pub fn no_fields_to_update<'a>(list_is_some: &[bool], valid_names: &'a str, msg: &'a str) -> Result<(), ValidationError> {
        let field_value_exists = list_is_some.iter().any(|&val| val == true);
        if !field_value_exists {
            let mut err = ValidationError::new(msg);
            let json = json!({ "validNames": valid_names });
            err.add_param(Cow::Borrowed(NM_NO_FIELDS_TO_UPDATE), &json);
            return Err(err);
        }
        Ok(())
    }
    // Checking, one of the optional fields must be present.
    #[rustfmt::skip]
    pub fn one_optional_fields_must_present<'a>(list_is_some: &[bool], fields: &'a str, msg: &'a str) -> Result<(), ValidationError> {
        let field_value_exists = list_is_some.iter().any(|&val| val == true);
        if !field_value_exists {
            let mut err = ValidationError::new(msg);
            let json = json!({ "optionalFields": fields });
            err.add_param(Cow::Borrowed(NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT), &json);
            return Err(err);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use actix_web;
    use chrono::{DateTime, Duration, SecondsFormat, Utc};
    use serde_json::{json, to_value};

    use crate::validators::{self, ValidationChecks, ValidationError};

    // ** ValidationChecks::required **
    #[actix_web::test]
    async fn test_validation_checks_required_valid() {
        let result = ValidationChecks::required("demo", "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_required_invalid() {
        let msg = "error";
        let result = ValidationChecks::required("", msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("required");
        assert!(err.params.get(param_name).is_some());
        assert_eq!(err.params.get(param_name).unwrap(), &to_value(true).unwrap());
    }

    // ** ValidationChecks::min_length **
    #[actix_web::test]
    async fn test_validation_checks_min_length_valid() {
        let result = ValidationChecks::min_length("demo", 3, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_min_length_invalid() {
        let value = "demo";
        let msg = "error";
        let min_len = 5;
        let result = ValidationChecks::min_length(value, min_len, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("minlength");
        assert!(err.params.get(param_name).is_some());
        let json = json!({ "actualLength": value.len(), "requiredLength": min_len });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::max_length **
    #[actix_web::test]
    async fn test_validation_checks_max_length_valid() {
        let result = ValidationChecks::max_length("demo", 5, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_max_length_invalid() {
        let value = "demonstration";
        let msg = "error";
        let max_len = 5;
        let result = ValidationChecks::max_length(value, max_len, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("maxlength");
        assert!(err.params.get(param_name).is_some());
        let json = json!({ "actualLength": value.len(), "requiredLength": max_len });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::min_amount **
    #[actix_web::test]
    async fn test_validation_checks_min_amount_valid() {
        let result = ValidationChecks::min_amount(3, 2, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_min_amount_invalid() {
        let amount = 4;
        let msg = "error";
        let min_amount = 5;
        let result = ValidationChecks::min_amount(amount, min_amount, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("minAmount");
        assert!(err.params.get(param_name).is_some());
        let json = json!({ "actualAmount": amount, "requiredAmount": min_amount });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }
    // ** ValidationChecks::max_amount **
    #[actix_web::test]
    async fn test_validation_checks_max_amount_valid() {
        let result = ValidationChecks::max_amount(2, 3, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_max_amount_invalid() {
        let amount = 6;
        let msg = "error";
        let max_amount = 5;
        let result = ValidationChecks::max_amount(amount, max_amount, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("maxAmount");
        assert!(err.params.get(param_name).is_some());
        let json = json!({ "actualAmount": amount, "requiredAmount": max_amount });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::regexp **
    #[actix_web::test]
    async fn test_validation_checks_regexp_valid() {
        let result = ValidationChecks::regexp("demo", "^[a-z]+$", "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_regexp_invalid() {
        let value = "Demo";
        let reg_exp = "^[a-z]+$";
        let msg = "error";
        let result = ValidationChecks::regexp(value, reg_exp, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("pattern");
        assert!(err.params.get(param_name).is_some());
        let json = json!({ "actualValue": value, "requiredPattern": reg_exp });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::email **
    #[actix_web::test]
    async fn test_validation_checks_email_valid() {
        let result = ValidationChecks::email("demo@mail", "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_email_invalid() {
        let value = "demo";
        let msg = "error";
        let result = ValidationChecks::email(value, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("email");
        assert!(err.params.get(param_name).is_some());
        assert_eq!(err.params.get(param_name).unwrap(), &to_value(true).unwrap());
    }

    // ** ValidationChecks::min_valid_date **
    #[actix_web::test]
    async fn test_validation_checks_min_valid_date_valid() {
        let value: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let min_date_time: DateTime<Utc> = Utc::now();
        let result = ValidationChecks::min_valid_date(&value, &min_date_time, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_min_valid_date_invalid() {
        let value: DateTime<Utc> = Utc::now();
        let min_date_time: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let msg = "error";
        let result = ValidationChecks::min_valid_date(&value, &min_date_time, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("minValidDateTime");
        assert!(err.params.get(param_name).is_some());
        let value_s = value.to_rfc3339_opts(SecondsFormat::Millis, true);
        let min_date_time_s = min_date_time.to_rfc3339_opts(SecondsFormat::Millis, true);
        let json = json!({ "actualDateTime": value_s, "minDateTime": min_date_time_s });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::max_valid_date **
    #[actix_web::test]
    async fn test_validation_checks_max_valid_date_valid() {
        let value: DateTime<Utc> = Utc::now();
        let max_date_time: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let result = ValidationChecks::max_valid_date(&value, &max_date_time, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_max_valid_date_invalid() {
        let value: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let max_date_time: DateTime<Utc> = Utc::now();
        let msg = "error";
        let result = ValidationChecks::max_valid_date(&value, &max_date_time, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("maxValidDateTime");
        assert!(err.params.get(param_name).is_some());
        let value_s = value.to_rfc3339_opts(SecondsFormat::Millis, true);
        let max_date_time_s = max_date_time.to_rfc3339_opts(SecondsFormat::Millis, true);
        let json = json!({ "actualDateTime": value_s, "maxDateTime": max_date_time_s });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::valid_value **
    #[actix_web::test]
    async fn test_validation_checks_valid_value_valid() {
        let result = ValidationChecks::valid_value("demo", &["demo"], "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_valid_value_invalid() {
        let value = "demo";
        let msg = "error";
        let result = ValidationChecks::valid_value(value, &[], msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed("invalid");
        let json = json!({ "actualValue": value });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::no_fields_to_update **
    #[actix_web::test]
    async fn test_validation_checks_no_fields_to_update_valid() {
        let fields = "field1, field2, field3";
        let result = ValidationChecks::no_fields_to_update(&[true, false, false], fields, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_no_fields_to_update_invalid() {
        let valid_names = "field1, field2, field3";
        let msg = "error";
        let result = ValidationChecks::no_fields_to_update(&[false, false, false], valid_names, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed(validators::NM_NO_FIELDS_TO_UPDATE);
        let json = json!({ "validNames": valid_names });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }

    // ** ValidationChecks::no_fields_to_update **
    #[actix_web::test]
    async fn test_validation_checks_one_optional_fields_must_present_valid() {
        let fields = "field1, field2, field3";
        let result = ValidationChecks::one_optional_fields_must_present(&[true, false, false], fields, "error");
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), ());
    }
    #[actix_web::test]
    async fn test_validation_checks_one_optional_fields_must_present_invalid() {
        let valid_names = "field1, field2, field3";
        let msg = "error";
        let result = ValidationChecks::one_optional_fields_must_present(&[false, false, false], valid_names, msg);
        assert!(result.is_err());
        let err: ValidationError = result.err().unwrap();
        assert_eq!(err.message, msg);
        let param_name = &Cow::Borrowed(validators::NM_ONE_OPTIONAL_FIELDS_MUST_PRESENT);
        let json = json!({ "optionalFields": valid_names });
        assert_eq!(err.params.get(param_name).unwrap(), &json);
    }
}

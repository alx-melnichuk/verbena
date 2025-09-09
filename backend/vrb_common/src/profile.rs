use crate::validators::{ValidationChecks, ValidationError};

// ** Section: "Profile.descript" **

pub const DESCRIPT_MIN: u8 = 2;
pub const DESCRIPT_MAX: u16 = 2048; // 2*1024
pub const MSG_DESCRIPT_MIN_LENGTH: &str = "descript:min_length";
pub const MSG_DESCRIPT_MAX_LENGTH: &str = "descript:max_length";

// MIN=2, MAX=2048
pub fn validate_descript(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, DESCRIPT_MIN.into(), MSG_DESCRIPT_MIN_LENGTH)?;
    ValidationChecks::max_length(value, DESCRIPT_MAX.into(), MSG_DESCRIPT_MAX_LENGTH)?;
    Ok(())
}

// ** Section: "Profile.theme" **

pub const THEME_MIN: u8 = 2;
pub const THEME_MAX: u8 = 32;
pub const MSG_THEME_MIN_LENGTH: &str = "theme:min_length";
pub const MSG_THEME_MAX_LENGTH: &str = "theme:max_length";
pub const PROFILE_THEME_LIGHT_DEF: &str = "light";
pub const PROFILE_THEME_DARK: &str = "dark";

// MIN=2, MAX=32
pub fn validate_theme(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, THEME_MIN.into(), MSG_THEME_MIN_LENGTH)?;
    ValidationChecks::max_length(value, THEME_MAX.into(), MSG_THEME_MAX_LENGTH)?;
    Ok(())
}

// ** Section: "Profile.locale" **

pub const LOCALE_MIN: u8 = 2;
pub const LOCALE_MAX: u8 = 32;
pub const MSG_LOCALE_MIN_LENGTH: &str = "locale:min_length";
pub const MSG_LOCALE_MAX_LENGTH: &str = "locale:max_length";
pub const PROFILE_LOCALE_DEF: &str = "default";

// MIN=2, MAX=32
pub fn validate_locale(value: &str) -> Result<(), ValidationError> {
    ValidationChecks::min_length(value, LOCALE_MIN.into(), MSG_LOCALE_MIN_LENGTH)?;
    ValidationChecks::max_length(value, LOCALE_MAX.into(), MSG_LOCALE_MAX_LENGTH)?;
    Ok(())
}


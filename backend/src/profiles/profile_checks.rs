use actix_web::web;
use log;

use crate::errors::AppError;
use crate::hash_tools;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{profile_models::Profile, profile_orm::ProfileOrm};
use crate::settings::err;

// Check the password value with the available hash.
pub fn check_password_with_its_hash(opt_profile: Option<Profile>, password: &str) -> Result<Profile, AppError> {
    // Get user profile with password.
    let profile_pwd = opt_profile.ok_or_else(|| {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
    })?;

    // Get a hash of the current password.
    let curr_profile_hashed_password = profile_pwd.password.to_string();
    // Check whether the hash for the specified password value matches the old password hash.
    let password_matches = hash_tools::compare_hash(password, &curr_profile_hashed_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_HASH, &e);
        log::error!("{}: {}", err::CD_CONFLICT, &message);
        AppError::conflict409(&message) // 409
    })?;

    // If the hash for the specified password does not match the old password hash, then return an error.
    if !password_matches {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_PASSWORD_INCORRECT);
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    Ok(profile_pwd)
}

// Check user password (Search profile by ID.).
pub async fn find_profile_by_id_and_check_password(
    profile_id: i32,
    password: &str,
    profile_orm: ProfileOrmApp,
) -> Result<Profile, AppError> {
    // Get the user profile including the password value.
    let opt_profile = web::block(move || {
        // Find profile by id.
        let existing_profile = profile_orm.get_profile_user_by_id(profile_id, true).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        existing_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    // Check the password value with the available hash.
    check_password_with_its_hash(opt_profile, password)
}

// Check user password (Search profile by nickname or email).
pub async fn find_profile_by_nickname_or_email_and_check_password(
    nickname: &str,
    email: &str,
    password: &str,
    profile_orm: ProfileOrmApp,
) -> Result<Profile, AppError> {
    let nickname2 = nickname.to_string();
    let email2 = email.to_string();
    // Get the user profile including the password value.
    let opt_profile = web::block(move || {
        // Find profile by id.
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(Some(&nickname2), Some(&email2), true)
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        existing_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    // Check the password value with the available hash.
    check_password_with_its_hash(opt_profile, password)
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use crate::profiles::profile_models::Profile;
    use crate::users::user_models::UserRole;

    use super::*;

    fn create_profile(nickname: &str, email: &str) -> Profile {
        let profile = ProfileOrmApp::new_profile(1, nickname, email, UserRole::User);
        profile
    }

    // ** check_password_with_its_hash **
    #[actix_web::test]
    async fn test_check_password_with_its_hash_profile_none() {
        let opt_profile: Option<Profile> = None;
        let result = check_password_with_its_hash(opt_profile, "demo");

        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED); // 401 (A)
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_check_password_with_its_hash_password_empty() {
        let nickname = "Oliver_Taylor";
        let profile = create_profile(nickname, &format!("{}@gmail.com", nickname));
        let opt_profile: Option<Profile> = Some(profile);
        let result = check_password_with_its_hash(opt_profile, "demo");

        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_CONFLICT); // 409
        let error = "Invalid parameter hash format - password hash string missing field";
        assert_eq!(app_err.message, format!("{}; {}", err::MSG_INVALID_HASH, error));
    }
    #[actix_web::test]
    async fn test_check_password_with_its_hash_password_bad_hash() {
        let nickname = "Oliver_Taylor";
        let mut profile = create_profile(nickname, &format!("{}@gmail.com", nickname));
        profile.password = "bad_hash_password".to_string();
        let opt_profile: Option<Profile> = Some(profile);
        let result = check_password_with_its_hash(opt_profile, "demo");

        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_CONFLICT); // 409
        let error = "Invalid parameter hash format - password hash string missing field";
        assert_eq!(app_err.message, format!("{}; {}", err::MSG_INVALID_HASH, error));
    }
    #[actix_web::test]
    async fn test_check_password_with_its_hash_incorrect_password() {
        let nickname = "Oliver_Taylor";
        let mut profile = create_profile(nickname, &format!("{}@gmail.com", nickname));
        let password = "password_D1T3";
        profile.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let opt_profile: Option<Profile> = Some(profile);
        let result = check_password_with_its_hash(opt_profile, &format!("{}a", password));

        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED); // 401 (B)
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_check_password_with_its_hash_password_ok() {
        let nickname = "Oliver_Taylor";
        let mut profile = create_profile(nickname, &format!("{}@gmail.com", nickname));
        let password = "password_D1T3";
        profile.password = hash_tools::encode_hash(password).unwrap(); // hashed
        let opt_profile: Option<Profile> = Some(profile.clone());
        let result = check_password_with_its_hash(opt_profile, password);

        assert!(result.is_ok());
        let profile2 = result.ok().unwrap();
        assert_eq!(profile, profile2);
    }
}

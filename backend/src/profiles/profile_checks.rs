use std::borrow::Cow;

use actix_web::web;

use crate::errors::AppError;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::profile_orm::ProfileOrm;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::user_registr_orm::UserRegistrOrm;

// None of the parameters are specified.
pub const MSG_PARAMETERS_NOT_SPECIFIED: &str = "parameters_not_specified";

pub async fn find_profile_for_nickname_or_email(
    opt_nickname: Option<String>,
    opt_email: Option<String>,
    profile_orm: ProfileOrmApp,
    user_registr_orm: UserRegistrOrmApp,
) -> Result<Option<(bool, bool)>, AppError> {
    let nickname = opt_nickname.unwrap_or("".to_string());
    let email = opt_email.unwrap_or("".to_string());
    #[rustfmt::skip]
    let opt_nickname = if nickname.len() > 0 { Some(nickname) } else { None };
    let opt_email = if email.len() > 0 { Some(email) } else { None };

    if opt_nickname.is_none() && opt_email.is_none() {
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        // #[rustfmt::skip]
        // #log::error!("{}: {}: {}", err::CD_NOT_ACCEPTABLE, MSG_PARAMETERS_NOT_SPECIFIED, json.to_string());
        return Err(AppError::not_acceptable406(MSG_PARAMETERS_NOT_SPECIFIED) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json));
    }

    let opt_nickname2 = opt_nickname.clone();
    let opt_email2 = opt_email.clone();

    // Find in the "profile" table an entry by nickname or email.
    let opt_profile = web::block(move || {
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(opt_nickname2.as_deref(), opt_email2.as_deref(), false)
            .map_err(|e| {
                // #log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            })
            .ok()?;
        existing_profile
    })
    .await
    .map_err(|e| {
        // #log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })?;
    // If such an entry exists in the "profiles" table, then exit.
    if opt_profile.is_some() {
        return Ok(Some((true, false)));
    }

    let opt_nickname2 = opt_nickname.clone();
    let opt_email2 = opt_email.clone();

    // Find in the "user_registr" table an entry with an active date, by nickname or email.
    let opt_user_registr = web::block(move || {
        let existing_user_registr = user_registr_orm
            .find_user_registr_by_nickname_or_email(opt_nickname2.as_deref(), opt_email2.as_deref())
            .map_err(|e| {
                // #log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            })
            .ok()?;
        existing_user_registr
    })
    .await
    .map_err(|e| {
        // #log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })?;

    // If such a record exists in the "registration" table, then exit.
    if opt_user_registr.is_some() {
        return Ok(Some((false, true)));
    }

    Ok(None)
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use chrono::{DateTime, Duration, Utc};

    use crate::profiles::profile_models::Profile;
    use crate::settings::err;
    use crate::users::user_models::{UserRegistr, UserRole};

    use super::*;

    fn get_data() -> (Vec<Profile>, Vec<UserRegistr>) {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);

        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);
        let user_registr =
            UserRegistrOrmApp::new_user_registr(1, "Robert_Brown", "Robert_Brown@gmail.com", "passwdR2B2", final_date);

        (vec![profile], vec![user_registr])
    }

    fn get_orm(data: (Vec<Profile>, Vec<UserRegistr>)) -> (ProfileOrmApp, UserRegistrOrmApp) {
        let profile_orm = ProfileOrmApp::create(&data.0);
        let user_registr_orm = UserRegistrOrmApp::create(&data.1);

        (profile_orm, user_registr_orm)
    }

    // ** find_profile_for_nickname_or_email **

    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_non_params() {
        let data = get_data();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = None;

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_nickname_empty() {
        let data = get_data();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some("".to_string());
        let opt_email: Option<String> = None;

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_email_empty() {
        let data = get_data();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some("".to_string());

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_nickname_profile() {
        let data = get_data();
        let nickname_profile = data.0.get(0).unwrap().nickname.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname_profile);
        let opt_email: Option<String> = None;

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        let (is_profile, is_registr) = result.ok().unwrap().unwrap();
        assert_eq!(is_profile, true);
        assert_eq!(is_registr, false);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_email_profile() {
        let data = get_data();
        let email_profile = data.0.get(0).unwrap().email.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email_profile);

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        let (is_profile, is_registr) = result.ok().unwrap().unwrap();
        assert_eq!(is_profile, true);
        assert_eq!(is_registr, false);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_nickname_registr() {
        let data = get_data();
        let nickname_registr = data.1.get(0).unwrap().nickname.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname_registr);
        let opt_email: Option<String> = None;

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        let (is_profile, is_registr) = result.ok().unwrap().unwrap();
        assert_eq!(is_profile, false);
        assert_eq!(is_registr, true);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_email_registr() {
        let data = get_data();
        let email_registr = data.1.get(0).unwrap().email.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email_registr);

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        let (is_profile, is_registr) = result.ok().unwrap().unwrap();
        assert_eq!(is_profile, false);
        assert_eq!(is_registr, true);
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_new_nickname() {
        let data = get_data();
        let nickname1 = format!("a{}", data.0.get(0).unwrap().nickname.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname1);
        let opt_email: Option<String> = None;

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(None, result.ok().unwrap());
    }
    #[actix_web::test]
    async fn test_find_profile_for_nickname_or_email_by_new_email() {
        let data = get_data();
        let email1 = format!("a{}", data.0.get(0).unwrap().email.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email1);

        let result = find_profile_for_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(None, result.ok().unwrap());
    }
}

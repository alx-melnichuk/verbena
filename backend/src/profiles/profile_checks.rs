use std::borrow::Cow;

use actix_web::web;
use vrb_tools::{api_error::ApiError, err};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::profile_orm::ProfileOrm;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::user_registr_orm::UserRegistrOrm;

pub async fn uniqueness_nickname_or_email(
    opt_nickname: Option<String>,
    opt_email: Option<String>,
    profile_orm: ProfileOrmApp,
    user_registr_orm: UserRegistrOrmApp,
) -> Result<Option<(bool, bool)>, ApiError> {
    let nickname = opt_nickname.unwrap_or("".to_string());
    let email = opt_email.unwrap_or("".to_string());
    #[rustfmt::skip]
    let opt_nickname = if nickname.len() > 0 { Some(nickname.clone()) } else { None };
    let opt_email = if email.len() > 0 { Some(email.clone()) } else { None };

    if opt_nickname.is_none() && opt_email.is_none() {
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        return Err(ApiError::new(406, err::MSG_PARAMS_NOT_SPECIFIED) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json));
    }

    let opt_nickname2 = opt_nickname.clone();
    let opt_email2 = opt_email.clone();

    // Find in the "profile" table an entry by nickname or email.
    let opt_profile = web::block(move || {
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(opt_nickname2.as_deref(), opt_email2.as_deref(), false)
            .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
            .ok()?;
        existing_profile
    })
    .await
    .map_err(|e| ApiError::create(506, err::MSG_BLOCKING, &e.to_string()))?; // 506

    // If such an entry exists in the "profiles" table, then exit.
    if let Some(profile) = opt_profile {
        return Ok(Some((nickname == profile.nickname, email == profile.email)));
    }

    // Find in the "user_registr" table an entry with an active date, by nickname or email.
    let opt_user_registr = web::block(move || {
        let existing_user_registr = user_registr_orm
            .find_user_registr_by_nickname_or_email(opt_nickname.as_deref(), opt_email.as_deref())
            .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
            .ok()?;
        existing_user_registr
    })
    .await
    .map_err(|e| ApiError::create(506, err::MSG_BLOCKING, &e.to_string()))?; // 506

    // If such a record exists in the "registration" table, then exit.
    if let Some(user_registr) = opt_user_registr {
        return Ok(Some((nickname == user_registr.nickname, email == user_registr.email)));
    }

    Ok(None)
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::http::StatusCode;
    use chrono::{DateTime, Duration, Utc};
    use vrb_tools::{api_error::code_to_str, err};

    use crate::profiles::profile_models::Profile;
    use crate::users::user_models::{UserRegistr, UserRole};

    use super::*;

    fn get_data() -> (Vec<Profile>, Vec<UserRegistr>) {
        let nickname = "Oliver_Taylor";
        let email = format!("{}@gmail.com", nickname);
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, nickname, &email, role);

        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);
        let nickname = "Robert_Brown";
        let email = format!("{}@gmail.com", nickname);
        let password = "passwdR2B2";
        let user_registr = UserRegistrOrmApp::new_user_registr(1, nickname, &email, password, final_date);

        (vec![profile], vec![user_registr])
    }

    fn get_orm(data: (Vec<Profile>, Vec<UserRegistr>)) -> (ProfileOrmApp, UserRegistrOrmApp) {
        let profile_orm = ProfileOrmApp::create(&data.0);
        let user_registr_orm = UserRegistrOrmApp::create(&data.1);

        (profile_orm, user_registr_orm)
    }

    // ** uniqueness_nickname_or_email **

    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_non_params() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_empty() {
        let data = get_data();
        let opt_nickname: Option<String> = Some("".to_string());
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_empty() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some("".to_string());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_profile() {
        let data = get_data();
        let opt_nickname: Option<String> = Some(data.0.get(0).unwrap().nickname.clone());
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((true, false)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_profile() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(data.0.get(0).unwrap().email.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((false, true)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_registr() {
        let data = get_data();
        let nickname_registr = data.1.get(0).unwrap().nickname.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname_registr);
        let opt_email: Option<String> = None;

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((true, false)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_registr() {
        let data = get_data();
        let email_registr = data.1.get(0).unwrap().email.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email_registr);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((false, true)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_new_nickname() {
        let data = get_data();
        let nickname1 = format!("a{}", data.0.get(0).unwrap().nickname.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname1);
        let opt_email: Option<String> = None;

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), None);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_new_email() {
        let data = get_data();
        let email1 = format!("a{}", data.0.get(0).unwrap().email.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email1);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), None);
    }
}

use std::borrow::Cow;

use actix_web::web;
use vrb_common::{api_error::ApiError, err};

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

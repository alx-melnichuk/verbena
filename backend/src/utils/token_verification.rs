use std::time::Instant as tm;

use actix_web::http::StatusCode;
use log::{error, info, log_enabled, Level::Info};
use vrb_tools::{
    api_error::{code_to_str, ApiError},
    err,
};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{profile_models::Profile, profile_orm::ProfileOrm};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::session_orm::SessionOrm;

// 401 Unauthorized - According to "user_id" in the token, the user was not found.
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id";

/** Check the token for correctness and get the user profile. */
pub async fn check_token_and_get_profile(
    user_id: i32,
    num_token: i32,
    session_orm: &SessionOrmApp,
    profile_orm: &ProfileOrmApp,
) -> Result<Profile, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Find a session for a given user.
    let opt_session = session_orm.get_session_by_id(user_id).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
        return ApiError::create(507, err::MSG_DATABASE, &e); // 507
    })?;
    let session = opt_session.ok_or_else(|| {
        // There is no session for this user.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg) // 406
    })?;
    // Each session contains an additional numeric value.
    let session_num_token = session.num_token.unwrap_or(0);
    // Compare an additional numeric value from the session and from the token.
    if session_num_token != num_token {
        // If they do not match, then this is an error.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg); // 401
        return Err(ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg));
    }
    let result = profile_orm.get_profile_user_by_id(user_id, false).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
        ApiError::create(507, err::MSG_DATABASE, &e) // 507
    })?;

    let profile = result.ok_or_else(|| {
        let message = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), MSG_UNACCEPTABLE_TOKEN_ID, &message);
        ApiError::create(401, MSG_UNACCEPTABLE_TOKEN_ID, &message) // 401+
    })?;

    if let Some(timer) = timer {
        let s1 = format!("{:.2?}", timer.elapsed());
        #[rustfmt::skip]
        info!("check_token_and_get_profile() time: {}, user_id: {}, nickname: {}", s1, profile.user_id, &profile.nickname);
    }
    Ok(profile)
}

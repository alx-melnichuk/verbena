use log::{debug, error, log_enabled, Level::Debug};

use crate::errors::AppError;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{profile_models::Profile, profile_orm::ProfileOrm};
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::session_orm::SessionOrm;
use crate::settings::err;

// 401 Unauthorized - According to "user_id" in the token, the user was not found.
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id";

/** Check the token for correctness and get the user profile. */
pub async fn check_token_and_get_profile(
    user_id: i32,
    num_token: i32,
    session_orm: &SessionOrmApp,
    profile_orm: &ProfileOrmApp,
) -> Result<Profile, AppError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };

    // let timer1 = std::time::Instant::now();
    // Find a session for a given user.
    let opt_session = session_orm.get_session_by_id(user_id).map_err(|e| {
        error!("{}: {}", err::CD_DATABASE, e.to_string());
        return AppError::database507(&e.to_string()); // 507
    })?;
    // let timer1s = format!("{:.2?}", timer1.elapsed());
    let session = opt_session.ok_or_else(|| {
        // There is no session for this user.
        let message = format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user_id);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        AppError::not_acceptable406(&message) // 406
    })?;
    // Each session contains an additional numeric value.
    let session_num_token = session.num_token.unwrap_or(0);
    // Compare an additional numeric value from the session and from the token.
    if session_num_token != num_token {
        // If they do not match, then this is an error.
        let message = format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user_id);
        error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        return Err(AppError::unauthorized401(&message)); // 401
    }
    // let timer2 = std::time::Instant::now();
    let result = profile_orm.get_profile_user_by_id(user_id, false).map_err(|e| {
        error!("{}: {}", err::CD_DATABASE, e.to_string());
        AppError::database507(&e.to_string()) // 507
    })?;
    // let timer2s = format!("{:.2?}", timer2.elapsed());
    // eprintln!("## timer1: {}, timer2: {}", timer1s, timer2s);

    let profile = result.ok_or_else(|| {
        let message = format!("{}; user_id: {}", MSG_UNACCEPTABLE_TOKEN_ID, user_id);
        error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401+
    })?;

    if let Some(timer0) = opt_timer0 {
        #[rustfmt::skip]
        debug!("timer0: {}, user_id: {}, nickname: {}", format!("{:.2?}", timer0.elapsed()), profile.user_id, &profile.nickname);
    }
    Ok(profile)
}

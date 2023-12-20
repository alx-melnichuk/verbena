use std::borrow;

use actix_web::{get, post, put, web, HttpResponse};
use chrono::{Duration, Utc};

use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::send_email::mailer::inst::MailerApp;
#[cfg(feature = "mockdata")]
use crate::send_email::mailer::tests::MailerApp;
use crate::send_email::mailer::Mailer;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{
    config_jwt,
    session_models::Session,
    session_orm::SessionOrm,
    tokens::{decode_token, encode_token, generate_num_token},
};
use crate::settings::{config_app, err};
#[cfg(not(feature = "mockdata"))]
use crate::users::user_recovery_orm::inst::UserRecoveryOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_recovery_orm::tests::UserRecoveryOrmApp;
use crate::users::{
    user_models, user_orm::UserOrm, user_recovery_orm::UserRecoveryOrm,
    user_registr_orm::UserRegistrOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::users::{user_orm::inst::UserOrmApp, user_registr_orm::inst::UserRegistrOrmApp};
#[cfg(feature = "mockdata")]
use crate::users::{user_orm::tests::UserOrmApp, user_registr_orm::tests::UserRegistrOrmApp};
use crate::validators::{msg_validation, Validator};
use crate::{errors::AppError, extractors::authentication::RequireAuth};

pub const MSG_EMAIL_ALREADY_USE: &str = "email_already_use";
pub const MSG_NICKNAME_ALREADY_USE: &str = "nickname_already_use";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/registration
    cfg.service(registration)
        // PUT api/registration/{registr_token}
        .service(confirm_registration)
        // POST api/recovery
        .service(recovery)
        // PUT api/recovery/{recovery_token}
        .service(confirm_recovery);
}

fn err_database(err: String) -> AppError {
    log::error!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_wrong_email_or_nickname(is_nickname: bool) -> AppError {
    let val = if is_nickname {
        MSG_NICKNAME_ALREADY_USE
    } else {
        MSG_EMAIL_ALREADY_USE
    };
    log::error!("{}: {}", err::CD_CONFLICT, val);
    AppError::new(err::CD_CONFLICT, val).set_status(409)
}
fn err_jsonwebtoken_encode(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_INTER_SRV_ERROR, err::MSG_JSON_WEB_TOKEN_ENCODE, err);
    let app_err = AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_JSON_WEB_TOKEN_ENCODE)
        .set_status(500)
        .add_param(borrow::Cow::Borrowed("error"), &err);
    app_err
}
fn err_jsonwebtoken_decode(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_FORBIDDEN, err::MSG_INVALID_OR_EXPIRED_TOKEN, err);
    AppError::new(err::CD_FORBIDDEN, err::MSG_INVALID_OR_EXPIRED_TOKEN).set_status(403)
}
fn err_sending_email(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_INTER_SRV_ERROR, err::MSG_ERROR_SENDING_EMAIL, &err);
    AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_ERROR_SENDING_EMAIL)
        .set_status(500)
        .add_param(borrow::Cow::Borrowed("error"), &err)
}
fn err_encode_hash(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_INTER_SRV_ERROR, err::MSG_ERROR_HASHING_PASSWORD, err);
    AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_ERROR_HASHING_PASSWORD)
        .set_status(500)
        .add_param(borrow::Cow::Borrowed("error"), &err)
}
fn err_registr_not_found(user_registr_id: i32) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - user_registr_id - {}", err::CD_NOT_FOUND, err::MSG_REGISTR_NOT_FOUND, user_registr_id);
    let name = borrow::Cow::Borrowed("user_registr_id");
    AppError::new(err::CD_NOT_FOUND, err::MSG_REGISTR_NOT_FOUND)
        .set_status(404)
        .add_param(name, &user_registr_id)
}
fn err_recovery_not_found(user_recovery_id: i32) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - user_recovery_id - {}", err::CD_NOT_FOUND, err::MSG_RECOVERY_NOT_FOUND, user_recovery_id);
    AppError::new(err::CD_NOT_FOUND, err::MSG_RECOVERY_NOT_FOUND)
        .set_status(404)
        .add_param(borrow::Cow::Borrowed("user_recovery_id"), &user_recovery_id)
}

// Send a confirmation email to register the user.
// POST api/registration
#[post("/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_body: web::Json<user_models::RegistrUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        #[rustfmt::skip]
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let mut registr_user_dto: user_models::RegistrUserDto = json_body.0.clone();
    registr_user_dto.nickname = registr_user_dto.nickname.to_lowercase();
    registr_user_dto.email = registr_user_dto.email.to_lowercase();

    let password = registr_user_dto.password.clone();
    let password_hashed = hash_tools::encode_hash(&password).map_err(|err| err_encode_hash(err))?;

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();

    // Find in the "user" table an entry by nickname or email.
    let user_opt = web::block(move || {
        let existing_user = user_orm
            .find_user_by_nickname_or_email(Some(&nickname), Some(&email))
            .map_err(|e| err_database(e.to_string()));
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let nickname = registr_user_dto.nickname.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user) = user_opt {
        return Err(err_wrong_email_or_nickname(user.nickname == nickname));
    }

    let email = registr_user_dto.email.clone();
    let user_registr_orm2 = user_registr_orm.clone();

    // Find in the "user_registr" table an entry with an active date, by nickname or email.
    let user_registr_opt = web::block(move || {
        let existing_user_registr = user_registr_orm2
            .find_user_registr_by_nickname_or_email(Some(&nickname), Some(&email))
            .map_err(|e| err_database(e.to_string()));
        existing_user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let nickname = registr_user_dto.nickname.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user_registr) = user_registr_opt {
        let is_match_nickname = user_registr.nickname == nickname;
        return Err(err_wrong_email_or_nickname(is_match_nickname));
    }

    // If there is no such record, then add the specified data to the "user_registr" table.

    let app_registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
    // Waiting time for registration confirmation (in seconds).
    let final_date_utc = Utc::now() + Duration::seconds(app_registr_duration.into());

    let create_user_registr_dto = user_models::CreateUserRegistrDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        password: password_hashed,
        final_date: final_date_utc,
    };
    // Create a new entity (user).
    let user_registr = web::block(move || {
        let user_registr = user_registr_orm
            .create_user_registr(create_user_registr_dto)
            .map_err(|e| err_database(e));
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_registr.id, num_token) into a registr_token.
    let registr_token = encode_token(user_registr.id, num_token, jwt_secret, app_registr_duration)
        .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;

    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let subject = format!("Account registration in {}", &config_app.app_name);
    let nickname = registr_user_dto.nickname.clone();
    let receiver = registr_user_dto.email.clone();
    let target = registr_token.clone();
    let registr_duration = app_registr_duration.clone();
    let result = mailer.send_verification_code(
        &receiver,
        &domain,
        &subject,
        &nickname,
        &target,
        registr_duration,
    );

    if result.is_err() {
        return Err(err_sending_email(result.unwrap_err()));
    }

    let registr_user_response_dto = user_models::RegistrUserResponseDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        registr_token: registr_token.clone(),
    };

    Ok(HttpResponse::Created().json(registr_user_response_dto))
}

// Confirm user registration.
// PUT api/registration/{registr_token}
#[put("/registration/{registr_token}")]
pub async fn confirm_registration(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let registr_token = request.match_info().query("registr_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “registr_token".
    let dual_token =
        decode_token(&registr_token, jwt_secret).map_err(|err| err_jsonwebtoken_decode(err))?;

    let user_registr_orm2 = user_registr_orm.clone();

    // Get "user_registr ID" from "registr_token".
    let (user_registr_id, _) = dual_token;

    // Find a record with the specified ID in the “user_registr" table.
    let user_registr_opt = web::block(move || {
        let user_registr = user_registr_orm2
            .find_user_registr_by_id(user_registr_id)
            .map_err(|e| err_database(e));
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user_registr = user_registr_opt.ok_or_else(|| err_registr_not_found(user_registr_id))?;

    let user_registr_orm2 = user_registr_orm.clone();
    // If such an entry exists, then add a new user.
    let create_user_dto = user_models::CreateUserDto {
        nickname: user_registr.nickname,
        email: user_registr.email,
        password: user_registr.password,
    };

    let user = web::block(move || {
        // Create a new entity (user).
        let user_res =
            user_orm.create_user(create_user_dto).map_err(|e| err_database(e.to_string()));
        user_res
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let session = Session {
        user_id: user.id,
        num_token: None,
    };
    let _ = web::block(move || {
        // Create a new entity (session).
        let session_res =
            session_orm.create_session(session).map_err(|e| err_database(e.to_string()));

        user_registr_orm2.delete_user_registr(user_registr_id).ok();

        session_res
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    // Delete entries in the "user_registr" table, that are already expired.
    let _ = web::block(move || user_registr_orm.delete_inactive_final_date(None))
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let user_dto = user_models::UserDto::from(user);

    Ok(HttpResponse::Created().json(user_dto))
}

// Send a confirmation email to recover the user's password.
// POST api/recovery
// errorCode: [400-"Validation",500-"Database",500-"Blocking",404-"NotFound",500-"ErrorSendingEmail,500-"JsonWebToken"]
#[post("/recovery")]
pub async fn recovery(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    user_orm: web::Data<UserOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    json_body: web::Json<user_models::RecoveryUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        #[rustfmt::skip]
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let mut recovery_user_dto: user_models::RecoveryUserDto = json_body.0.clone();
    recovery_user_dto.email = recovery_user_dto.email.to_lowercase();

    let email = recovery_user_dto.email.clone();

    // Find in the "user" table an entry by email.
    let user_opt = web::block(move || {
        let existing_user = user_orm
            .find_user_by_nickname_or_email(None, Some(&email))
            .map_err(|e| err_database(e.to_string()));
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If such an entry does not exist, then exit with code 404.
    let user = match user_opt {
        Some(v) => v,
        None => {
            log::error!("{}: {}", err::CD_NOT_FOUND, err::MSG_NOT_FOUND_BY_EMAIL);
            #[rustfmt::skip]
            return Err(AppError::new(err::CD_NOT_FOUND, err::MSG_NOT_FOUND_BY_EMAIL).set_status(404));
        }
    };
    let user_id = user.id;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is a user with this ID, then move on to the next stage.

    // For this user, find an entry in the "user_recovery" table.
    let user_recovery_opt = web::block(move || {
        let existing_user_recovery = user_recovery_orm2
            .find_user_recovery_by_user_id(user_id)
            .map_err(|e| err_database(e.to_string()));
        existing_user_recovery
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // Prepare data for writing to the "user_recovery" table.
    let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
    // Waiting time for password recovery confirmation (in seconds).
    let final_date_utc = Utc::now() + Duration::seconds(app_recovery_duration.into());

    let create_user_recovery_dto = user_models::CreateUserRecoveryDto {
        user_id: user_id,
        final_date: final_date_utc,
    };
    let user_recovery_id: i32;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is an entry for this user in the "user_recovery" table, then update it with a new token.
    if let Some(user_recovery) = user_recovery_opt {
        user_recovery_id = user_recovery.id;
        let _ = web::block(move || {
            let user_recovery = user_recovery_orm2
                .modify_user_recovery(user_recovery_id, create_user_recovery_dto)
                .map_err(|e| err_database(e));
            user_recovery
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))??;
    } else {
        // If there is no entry for this user in the "user_recovery" table, then add a new entry.
        // Create a new entity (user_recovery).
        let user_recovery = web::block(move || {
            let user_recovery = user_recovery_orm2
                .create_user_recovery(create_user_recovery_dto)
                .map_err(|e| err_database(e));
            user_recovery
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))??;

        user_recovery_id = user_recovery.id;
    }

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_recovery_id, num_token) into a recovery_token.
    let recovery_token = encode_token(
        user_recovery_id,
        num_token,
        jwt_secret,
        app_recovery_duration,
    )
    .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;

    // Prepare a letter confirming this recovery.
    let domain = &config_app.app_domain;
    let subject = format!("Account recovery on {}", &config_app.app_name);
    let nickname = user.nickname.clone();
    let receiver = user.email.clone();
    let target = recovery_token.clone();
    let recovery_duration = app_recovery_duration.clone();
    // Send an email to this user.
    let result = mailer.send_password_recovery(
        &receiver,
        &domain,
        &subject,
        &nickname,
        &target,
        recovery_duration,
    );

    if result.is_err() {
        return Err(err_sending_email(result.unwrap_err()));
    }

    let recovery_user_response_dto = user_models::RecoveryUserResponseDto {
        id: user_recovery_id,
        email: user.email.clone(),
        recovery_token: recovery_token.clone(),
    };

    Ok(HttpResponse::Created().json(recovery_user_response_dto))
}

// Confirm user password recovery.
// PUT api/recovery/{recovery_token}
#[put("/recovery/{recovery_token}")]
pub async fn confirm_recovery(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<user_models::RecoveryDataDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        #[rustfmt::skip]
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let recovery_data_dto: user_models::RecoveryDataDto = json_body.0.clone();

    // Prepare a password hash.
    let password_hashed =
        hash_tools::encode_hash(&recovery_data_dto.password).map_err(|err| err_encode_hash(err))?;

    let recovery_token = request.match_info().query("recovery_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “recovery_token".
    let dual_token =
        decode_token(&recovery_token, jwt_secret).map_err(|err| err_jsonwebtoken_decode(err))?;

    let user_recovery_orm2 = user_recovery_orm.clone();

    // Get "user_recovery ID" from "recovery_token".
    let (user_recovery_id, _) = dual_token;

    // Find a record with the specified ID in the “user_recovery" table.
    let user_recovery_opt = web::block(move || {
        let user_recovery = user_recovery_orm2
            .find_user_recovery_by_id(user_recovery_id)
            .map_err(|e| err_database(e));
        user_recovery
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user_recovery =
        user_recovery_opt.ok_or_else(|| err_recovery_not_found(user_recovery_id))?;
    let user_id = user_recovery.user_id;

    // If there is "user_recovery" with this ID, then move on to the next step.
    let user_orm2 = user_orm.clone();
    // Find an entry in "user" with the specified ID.
    let user_opt = web::block(move || {
        let user = user_orm2.find_user_by_id(user_id).map_err(|e| err_database(e));
        user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user = user_opt.ok_or_else(|| err_recovery_not_found(user_recovery_id))?;

    let user_orm2 = user_orm.clone();
    let modify_user_dto = user_models::ModifyUserDto {
        nickname: None,
        email: None,
        password: Some(password_hashed),
        role: None,
    };
    // Update the password hash for the entry in the "user" table.
    let user_opt = web::block(move || {
        let user = user_orm2.modify_user(user.id, modify_user_dto).map_err(|e| err_database(e));
        user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    let _ = web::block(move || {
        // Delete entries in the “user_recovery" table.
        let user_recovery_res = user_recovery_orm2
            .delete_user_recovery(user_recovery_id)
            .map_err(|e| err_database(e));

        // Clear the user session in the "session" table.
        let session_res = session_orm
            .modify_session(user_id, None)
            .map_err(|e| err_database(e.to_string()));

        (user_recovery_res, session_res)
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    // Delete entries in the "user_recovery" table, that are already expired.
    let _ = web::block(move || {
        user_recovery_orm.delete_inactive_final_date(None).map_err(|e| err_database(e))
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let user = user_opt.ok_or_else(|| err_recovery_not_found(user_recovery_id))?;
    let user_dto = user_models::UserDto::from(user);

    Ok(HttpResponse::Ok().json(user_dto))
}

// Clean up expired requests
// GET api/clear_for_expired
#[rustfmt::skip]
#[get("/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn clear_for_expired(
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {

    // Delete entries in the "user_registr" table, that are already expired.
    let count_inactive_registr_res = 
        web::block(move || user_registr_orm.delete_inactive_final_date(None))
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let count_inactive_registr = count_inactive_registr_res.unwrap_or(0);

    // Delete entries in the "user_recovery" table, that are already expired.
    let count_inactive_recover_res = 
        web::block(move || user_recovery_orm.delete_inactive_final_date(None)
        .map_err(|e| err_database(e)))
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let count_inactive_recover = count_inactive_recover_res.unwrap_or(0);

    let clear_for_expired_response_dto = user_models::ClearForExpiredResponseDto {
        count_inactive_registr,
        count_inactive_recover,
    };
    
    Ok(HttpResponse::Ok().json(clear_for_expired_response_dto))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;

    use crate::errors::AppError;
    use crate::extractors::authentication::BEARER;
    use crate::send_email::config_smtp;
    use crate::sessions::{config_jwt, tokens::decode_token};
    use crate::settings::{config_app, err};
    use crate::users::{
        user_models::{
            RecoveryDataDto, RecoveryUserDto, RecoveryUserResponseDto, RegistrUserDto, User,
            UserDto, UserModelsTest, UserRecovery, UserRegistr, UserRole,
        },
        user_orm::tests::{UserOrmApp, USER_ID},
        user_recovery_orm::tests::USER_RECOVERY_ID,
        user_registr_orm::tests::{UserRegistrOrmApp, USER_REGISTR_ID},
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user() -> User {
        let mut user =
            UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }
    fn create_user_registr() -> UserRegistr {
        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);

        let user_registr = UserRegistrOrmApp::new_user_registr(
            1,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
            final_date,
        );
        user_registr
    }
    fn user_registr_with_id(user_registr: UserRegistr) -> UserRegistr {
        let user_reg_orm = UserRegistrOrmApp::create(vec![user_registr]);
        user_reg_orm.user_registr_vec.get(0).unwrap().clone()
    }
    fn create_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
        UserRecoveryOrmApp::new_user_recovery(id, user_id, final_date)
    }
    fn create_user_recovery_with_id(user_recovery: UserRecovery) -> UserRecovery {
        let user_recovery_orm = UserRecoveryOrmApp::create(vec![user_recovery]);
        user_recovery_orm.user_recovery_vec.get(0).unwrap().clone()
    }

    async fn call_service1(
        config_jwt: config_jwt::ConfigJwt,
        vec: (Vec<User>, Vec<UserRegistr>, Vec<Session>, Vec<UserRecovery>),
        token: &str,
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_app = web::Data::new(config_app::get_test_config());
        let data_config_jwt = web::Data::new(config_jwt);
        let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));

        let data_user_orm = web::Data::new(UserOrmApp::create(vec.0));
        let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(vec.1));
        let data_session_orm = web::Data::new(SessionOrmApp::create(vec.2));
        let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(vec.3));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm))
                .service(factory),
        )
        .await;
        let test_request = if token.len() > 0 {
            request.insert_header((http::header::AUTHORIZATION, format!("{}{}", BEARER, token)))
        } else {
            request
        };
        let req = test_request.to_request();

        test::call_service(&app, req).await
    }

    // ** registration **
    #[test]
    async fn test_registration_no_data() {
        let token = "";

        let request = test::TestRequest::post().uri("/registration"); // POST /registration
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_registration_empty_json_object() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(json!({}));
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_empty() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REQUIRED);
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: UserModelsTest::nickname_min(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MIN_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: UserModelsTest::nickname_max(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MAX_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: UserModelsTest::nickname_wrong(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REGEX);
    }
    #[test]
    async fn test_registration_invalid_dto_email_empty() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_REQUIRED);
    }
    #[test]
    async fn test_registration_invalid_dto_email_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserModelsTest::email_min(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MIN_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_email_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserModelsTest::email_max(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MAX_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_email_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: UserModelsTest::email_wrong(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_EMAIL_TYPE);
    }
    #[test]
    async fn test_registration_invalid_dto_password_empty() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REQUIRED);
    }
    #[test]
    async fn test_registration_invalid_dto_password_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserModelsTest::password_min(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MIN_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_password_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserModelsTest::password_max(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MAX_LENGTH);
    }
    #[test]
    async fn test_registration_invalid_dto_password_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: UserModelsTest::password_wrong(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REGEX);
    }
    #[test]
    async fn test_registration_if_nickname_exists_in_users() {
        let user1 = user_with_id(create_user());
        let nickname1: String = user1.nickname.to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: nickname1,
                email: "Mary_Williams@gmail.com".to_string(),
                password: "passwordD2T2".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, MSG_NICKNAME_ALREADY_USE);
    }
    #[test]
    async fn test_registration_if_email_exists_in_users() {
        let user1 = user_with_id(create_user());
        let email1: String = user1.email.to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Mary_Williams".to_string(),
                email: email1,
                password: "passwordD2T2".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, MSG_EMAIL_ALREADY_USE);
    }
    #[test]
    async fn test_registration_if_nickname_exists_in_registr() {
        let user_registr1: UserRegistr = create_user_registr();
        let nickname1: String = user_registr1.nickname.to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: nickname1,
                email: "Mary_Williams@gmail.com".to_string(),
                password: "passwordD2T2".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![user_registr1], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, MSG_NICKNAME_ALREADY_USE);
    }
    #[test]
    async fn test_registration_if_email_exists_in_registr() {
        let user_registr1: UserRegistr = create_user_registr();
        let email1: String = user_registr1.email.to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Mary_Williams".to_string(),
                email: email1,
                password: "passwordD2T2".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![user_registr1], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, MSG_EMAIL_ALREADY_USE);
    }
    #[test]
    async fn test_login_err_jsonwebtoken_encode() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: "Mary_Williams".to_string(),
                email: "Mary_Williams@gmail.com".to_string(),
                password: "passwordD2T2".to_string(),
            });

        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_JSON_WEB_TOKEN_ENCODE);
    }
    #[test]
    async fn test_registration_new_user() {
        let user_registr1: UserRegistr = create_user_registr();

        let nickname = "Mary_Williams".to_string();
        let email = "Mary_Williams@gmail.com".to_string();
        let password = "passwordD2T2".to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/registration") // POST /registration
            .set_json(RegistrUserDto {
                nickname: nickname.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![user_registr1], vec![], vec![]);
        let factory = registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;

        let registr_user_resp: user_models::RegistrUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let final_date: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let user_registr = UserRegistrOrmApp::new_user_registr(
            1,
            &nickname.to_string(),
            &email.to_string(),
            &password.to_string(),
            final_date,
        );

        assert_eq!(user_registr.nickname, registr_user_resp.nickname);
        assert_eq!(user_registr.email, registr_user_resp.email);

        let registr_token = registr_user_resp.registr_token;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let (user_registr_id, _) = decode_token(&registr_token, jwt_secret).unwrap();
        assert_eq!(USER_REGISTR_ID + 1, user_registr_id);
    }

    // ** confirm_registration **
    #[test]
    async fn test_registration_confirm_invalid_registr_token() {
        let registr_token = "invalid_registr_token";

        let token = "";

        // PUT /registration/{registr_token}
        let request = test::TestRequest::put().uri(&format!("/registration/{}", registr_token));

        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = confirm_registration;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_registration_confirm_final_date_has_expired() {
        let user_reg1 = user_registr_with_id(create_user_registr());
        let user_reg1_id = user_reg1.id;

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token =
            encode_token(user_reg1_id, num_token, jwt_secret, -reg_duration).unwrap();

        let token = "";

        //PUT /registration/{registr_token}
        let request = test::TestRequest::put().uri(&format!("/registration/{}", registr_token));

        let factory = confirm_registration;
        let vec = (vec![], vec![user_reg1], vec![], vec![]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_registration_confirm_no_exists_in_user_regist() {
        let user_reg1 = user_registr_with_id(create_user_registr());
        let user_reg1_id = user_reg1.id;

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token =
            encode_token(user_reg1_id + 1, num_token, jwt_secret, reg_duration).unwrap();

        let token = "";

        //PUT /registration/{registr_token}
        let request = test::TestRequest::put().uri(&format!("/registration/{}", registr_token));

        let factory = confirm_registration;
        let vec = (vec![], vec![user_reg1], vec![], vec![]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_REGISTR_NOT_FOUND);
    }
    #[test]
    async fn test_registration_confirm_exists_in_user_regist() {
        let user_reg1 = user_registr_with_id(create_user_registr());
        let nickname = user_reg1.nickname.to_string();
        let email = user_reg1.email.to_string();

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token =
            encode_token(user_reg1.id, num_token, jwt_secret, reg_duration).unwrap();

        let token = "";

        //PUT /registration/{registr_token}
        let request = test::TestRequest::put().uri(&format!("/registration/{}", registr_token));

        let factory = confirm_registration;
        let vec = (vec![], vec![user_reg1], vec![], vec![]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, USER_ID);
        assert_eq!(user_dto_res.nickname, nickname);
        assert_eq!(user_dto_res.email, email);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, UserRole::User);
    }

    // ** recovery **
    #[test]
    async fn test_recovery_no_data() {
        let token = "";
        let request = test::TestRequest::post().uri("/recovery"); //POST /recovery
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_recovery_empty_json_object() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(json!({}));
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_recovery_invalid_dto_email_empty() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: "".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_REQUIRED);
    }
    #[test]
    async fn test_recovery_invalid_dto_email_min() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: UserModelsTest::email_min(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MIN_LENGTH);
    }
    #[test]
    async fn test_recovery_invalid_dto_email_max() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: UserModelsTest::email_max(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MAX_LENGTH);
    }
    #[test]
    async fn test_recovery_invalid_dto_email_wrong() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: UserModelsTest::email_wrong(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_EMAIL_TYPE);
    }
    #[test]
    async fn test_recovery_if_user_with_email_not_exist() {
        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: "Oliver_Taylor@gmail.com".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_NOT_FOUND_BY_EMAIL);
    }
    #[test]
    async fn test_recovery_if_user_recovery_not_exist() {
        let user1 = user_with_id(create_user());
        let user1_email = user1.email.to_string();

        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: user1_email.to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![], vec![], vec![]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_recov_res: RecoveryUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, USER_RECOVERY_ID);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) =
            decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(USER_RECOVERY_ID, user_recovery_id);
    }
    #[test]
    async fn test_recovery_if_user_recovery_already_exists() {
        let user1 = user_with_id(create_user());
        let user1_email = user1.email.to_string();

        let config_app = config_app::get_test_config();
        let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(app_recovery_duration.into());

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_utc));
        let user_recovery1_id = user_recovery1.id;

        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: user1_email.to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_recov_res: RecoveryUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) =
            decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery1_id, user_recovery_id);
    }
    #[test]
    async fn test_recovery_err_jsonwebtoken_encode() {
        let user1 = user_with_id(create_user());
        let user1_email = user1.email.to_string();

        let config_app = config_app::get_test_config();
        let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(app_recovery_duration.into());

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_utc));

        let token = "";
        let request = test::TestRequest::post()
            .uri("/recovery") //POST /recovery
            .set_json(RecoveryUserDto {
                email: user1_email.to_string(),
            });
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let factory = recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_JSON_WEB_TOKEN_ENCODE);
    }

    // ** confirm recovery **
    #[test]
    async fn test_recovery_confirm_invalid_dto_password_empty() {
        let recovery_token = "recovery_token";
        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = confirm_recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REQUIRED);
    }
    #[test]
    async fn test_recovery_confirm_invalid_dto_password_min() {
        let recovery_token = "recovery_token";
        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: UserModelsTest::password_min(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = confirm_recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MIN_LENGTH);
    }
    #[test]
    async fn test_recovery_confirm_invalid_dto_password_max() {
        let recovery_token = "recovery_token";
        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: UserModelsTest::password_max(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = confirm_recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MAX_LENGTH);
    }
    #[test]
    async fn test_recovery_confirm_invalid_recovery_token() {
        let recovery_token = "invalid_recovery_token";
        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "passwordQ2V2".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![], vec![], vec![]);
        let factory = confirm_recovery;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_recovery_confirm_final_date_has_expired() {
        let user1 = user_with_id(create_user());

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(-recovery_duration);

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_utc));

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token =
            encode_token(user_recovery1.id, num_token, jwt_secret, -recovery_duration).unwrap();

        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "passwordQ2V2".to_string(),
            });
        let factory = confirm_recovery;
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_recovery_confirm_no_exists_in_user_recovery() {
        let user1 = user_with_id(create_user());

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_utc));

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = encode_token(
            user_recovery1.id + 1,
            num_token,
            jwt_secret,
            recovery_duration,
        )
        .unwrap();

        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "passwordQ2V2".to_string(),
            });
        let factory = confirm_recovery;
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_RECOVERY_NOT_FOUND);
    }
    #[test]
    async fn test_recovery_confirm_no_exists_in_user() {
        let user1 = user_with_id(create_user());

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id + 1, final_date_utc));

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = encode_token(
            user_recovery1.id + 1,
            num_token,
            jwt_secret,
            recovery_duration,
        )
        .unwrap();

        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "passwordQ2V2".to_string(),
            });
        let factory = confirm_recovery;
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_RECOVERY_NOT_FOUND);
    }
    #[test]
    async fn test_recovery_confirm_success() {
        let user1 = user_with_id(create_user());
        let user1b = user1.clone();

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_utc));

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token =
            encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();

        let token = "";
        let request = test::TestRequest::put()
            .uri(&format!("/recovery/{}", recovery_token)) //PUT /recovery/{recovery_token}
            .set_json(RecoveryDataDto {
                password: "passwordQ2V2".to_string(),
            });
        let factory = confirm_recovery;
        let vec = (vec![user1], vec![], vec![], vec![user_recovery1]);
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1b.id);
        assert_eq!(user_dto_res.nickname, user1b.nickname);
        assert_eq!(user_dto_res.email, user1b.email);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1b.role);
    }

    // ** clear_for_expired **
    #[test]
    async fn test_clear_for_expired_user_recovery() {
        let mut user = create_user();
        user.role = UserRole::Admin;
        let user1: User = user_with_id(user);

        let config_app = config_app::get_test_config();

        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_recovery = Utc::now() - Duration::seconds(recovery_duration);

        let user_recovery1 =
            create_user_recovery_with_id(create_user_recovery(1, user1.id, final_date_recovery));

        let registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
        let final_date_registr = Utc::now() - Duration::seconds(registr_duration);

        let mut user_registr: UserRegistr = create_user_registr();
        user_registr.final_date = final_date_registr;
        let user_registr1 = user_registr_with_id(user_registr);

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().uri(&"/clear_for_expired"); // GET api/clear_for_expired

        let factory = clear_for_expired;
        let vec = (
            vec![user1],
            vec![user_registr1],
            vec![session1],
            vec![user_recovery1],
        );
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response_dto: user_models::ClearForExpiredResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(response_dto.count_inactive_registr, 1);
        assert_eq!(response_dto.count_inactive_recover, 1);
    }
}

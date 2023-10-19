use actix_web::{post, web, HttpResponse};
use chrono::{Duration, Utc};
use std::error;
use validator::{Validate, ValidationErrors};

use crate::email::{self, config_smtp};
use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::sessions::hash_tools;
use crate::users::user_models;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::user_orm::UserOrm;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::user_registr_models::CreateUserRegistrDto;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::user_registr_orm::UserRegistrOrm;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::UserRegistrOrmApp;
use crate::utils::{config_app, err};

pub const CD_WRONG_EMAIL: &str = "WrongEmail";
pub const MSG_WRONG_EMAIL: &str = "The specified email is incorrect!";

pub const CD_WRONG_NICKNAME2: &str = "WrongNickname";
pub const MSG_WRONG_NICKNAME2: &str = "The specified nickname is incorrect!";

pub const CD_ERROR_SENDING_EMAIL: &str = "ErrorSendingEmail";
pub const MSG_ERROR_SENDING_EMAIL: &str = "Error sending registration confirmation email.";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/registration
    cfg.service(registration);
}

fn err_database(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_wrong_email_or_nickname(is_email: bool) -> AppError {
    let val = if is_email {
        (CD_WRONG_EMAIL, MSG_WRONG_EMAIL)
    } else {
        (CD_WRONG_NICKNAME2, MSG_WRONG_NICKNAME2)
    };
    log::debug!("{}: {}", val.0, val.1);
    AppError::new(val.0, val.1).set_status(409)
}

// Send a confirmation email to register the user.
// POST api/registration
#[post("/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    config_smtp: web::Data<config_smtp::ConfigSmtp>,
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_user_dto: web::Json<user_models::RegistrationUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let mut registr_user_dto: user_models::RegistrationUserDto = json_user_dto.0.clone();
    registr_user_dto.nickname = registr_user_dto.nickname.to_lowercase();
    registr_user_dto.email = registr_user_dto.email.to_lowercase();

    let password = registr_user_dto.password.clone();
    let password_hashed = hash_tools::hash(&password).map_err(|e| {
        log::debug!("{}: {}", hash_tools::CD_HASHING_PASSWD, e.to_string());
        AppError::new(hash_tools::CD_HASHING_PASSWD, &e.to_string()).set_status(500)
    })?;

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();

    // Find in the "user" table an entry by nickname or email.
    let user_opt = web::block(move || {
        let existing_user = user_orm
            .find_user_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()));
        eprintln!("#_existing_user: {:#?}", existing_user); // #-
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let email = registr_user_dto.email.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user) = user_opt {
        eprintln!("#_ user is exist"); // #-
        return Err(err_wrong_email_or_nickname(user.email == email));
    }

    let nickname = registr_user_dto.nickname.clone();
    let user_registr_orm2 = user_registr_orm.clone();

    // Find in the "user_registration" table an entry with an active date, by nickname or email.
    let user_registr_opt = web::block(move || {
        let existing_user_registr = user_registr_orm
            .find_user_registr_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()));
        eprintln!("#_existing_user_registr: {:#?}", existing_user_registr); // #-
        existing_user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let email = registr_user_dto.email.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user_registr) = user_registr_opt {
        eprintln!("#_ user_registr is exist"); // #-
        return Err(err_wrong_email_or_nickname(user_registr.email == email));
    }

    // If there is no such record, then add the specified data to the "user_registration" table.

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();
    // let dt = Local::now().dt.naive_utc();
    // let final_date_utc = Utc::now();
    let app_registr_duration = config_app.app_registr_duration.try_into().unwrap();
    let final_date_utc = Utc::now() + Duration::minutes(app_registr_duration);

    let create_user_registr_dto = CreateUserRegistrDto {
        nickname,
        email,
        password: password_hashed,
        final_date: final_date_utc,
    };

    let user_registr = web::block(move || {
        // Create a new entity (user).
        let user_registr = user_registr_orm2
            .create_user_registr(&create_user_registr_dto)
            .map_err(|e| err_database(e));

        eprintln!("#_user_registr: {:#?}", user_registr); // #-
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let target = format!("target_{}", user_registr.id);
    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let nickname = registr_user_dto.nickname.clone();

    // Create an instance of Mailer.
    let mailer = email::mailer::Mailer::new(config_smtp.get_ref().clone());
    // let receiver = email;
    let receiver = "lg2aam@gmail.com";

    let result = mailer.send_verification_code(receiver, &domain, &nickname, &target);
    if result.is_err() {
        let err = result.unwrap_err();
        eprintln!("Failed to send email: {:?}", err);
        log::debug!("{CD_ERROR_SENDING_EMAIL}: {MSG_ERROR_SENDING_EMAIL}");
        return Err(AppError::new(CD_ERROR_SENDING_EMAIL, MSG_ERROR_SENDING_EMAIL).set_status(409));
    }

    Ok(HttpResponse::Ok().into())
}

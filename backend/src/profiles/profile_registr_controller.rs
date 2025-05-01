use actix_web::{get, post, put, web, HttpResponse};
use chrono::{Duration, Utc};
use utoipa;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    profile_checks,
    profile_models::{
        self, Profile, ProfileDto, RegistrProfileDto, RegistrProfileResponseDto, PROFILE_THEME_LIGHT_DEF, RecoveryProfileResponseDto, RecoveryProfileDto, RecoveryDataDto, PROFILE_THEME_DARK, ClearForExpiredResponseDto,
    },
    profile_err as p_err,
    profile_orm::ProfileOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::send_email::mailer::impls::MailerApp;
#[cfg(feature = "mockdata")]
use crate::send_email::mailer::tests::MailerApp;
use crate::send_email::mailer::Mailer;
use crate::sessions::{
    config_jwt,
    tokens::{decode_token, encode_token, generate_num_token},
};
use crate::settings::{config_app, err};
#[cfg(not(feature = "mockdata"))]
use crate::users::user_recovery_orm::impls::UserRecoveryOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_recovery_orm::tests::UserRecoveryOrmApp;
use crate::hash_tools;
use crate::users::{
    user_models::{self,UserRole},
    user_recovery_orm::UserRecoveryOrm,
    user_registr_orm::UserRegistrOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::validators::{msg_validation, Validator};
use crate::errors::AppError;
use crate::extractors::authentication::RequireAuth;

// 510 Not Extended - Error when sending email.
pub const MSG_ERROR_SENDING_EMAIL: &str = "error_sending_email";
// 404 Not Found - Registration record not found.
pub const MSG_REGISTR_NOT_FOUND: &str = "registration_not_found";
// 404 Not Found - Recovery record not found.
pub const MSG_RECOVERY_NOT_FOUND: &str = "recovery_not_found";
// 404 Not Found - User not found.
pub const MSG_USER_NOT_FOUND: &str = "user_not_found";

const TOKEN_REGISTR: &str = concat!(
    "BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT_allv-RMwbJsI67Aufl3gCD7pkSNU4zJWr2eROQ-6xGjDwibujOTTf6xZXV29k3E8ODzFqbkk2Pty4W",
    "puSI6YG-d6XDi4s4yczfWgfYM65kKjrD3FxGSa15zBXOR2yxrEXeziVyV2bbnDu1-Uex0_Pqg0zkHvu_k--L2D0xP4o_m5WeBV539-1YTNeExQ0A9_Pv8=",
);

const TOKEN_RECOVERY: &str = concat!(
    "BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT_allv-RMwbJsI67Aufl3gCD7pyis6TI9VxbRkc_jx3yORspBCqVJ8yuNNRcZox0sr9XLWlbIYKLR7yv",
    "H16LMFEmE1fuP7fu4hdc4XRk7j9XTd2pOxry-jHiPMJ-ijWKhZrXORmac2KFLMf6dUO2qWh7LcmpD73WWfqoAqMPUnAUDreY_xwkZ3wofSUXHsloucUbk=",
);

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/registration
            .service(registration)
            // PUT /api/registration/{registr_token}
            .service(confirm_registration)
            // POST /api/recovery
            .service(recovery)
            // PUT /api/recovery/{recovery_token}
            .service(confirm_recovery)
            // GET /api/clear_for_expired
            .service(clear_for_expired);
    }
}

/// registration
///
/// Send an email confirming user registration.
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/registration \
/// -d '{"nickname": "user", "email": "user@email", "password": "password"}' \
/// -H 'Content-Type: application/json'
/// ```
///
/// Return new user registration parameters (`RegistrProfileResponseDto`) with status 201.
///
#[utoipa::path(
    responses(
        (status = 201, description = "New user registration parameters and registration token.", body = RegistrProfileResponseDto,
            example = json!(RegistrProfileResponseDto { nickname:"Emma_Johnson".to_string(), email:"Emma_Johnson@gmail.us".to_string(),registr_token: TOKEN_REGISTR.to_string() })
        ),
        (status = 409, description = "Error: nickname (email) is already in use.", body = AppError, examples(
            ("Nickname" = (summary = "Nickname already used",
                description = "The nickname value has already been used.",
                value = json!(AppError::conflict409(err::MSG_NICKNAME_ALREADY_USE)))),
            ("Email" = (summary = "Email already used", 
                description = "The email value has already been used.",
                value = json!(AppError::conflict409(err::MSG_EMAIL_ALREADY_USE))))
        )),
        (status = 417, body = [AppError], description = "Validation error. `curl -i
             -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"email\": \"us_email\", \"password\": \"pas\" }'`",
            example = json!(AppError::validations(
                (RegistrProfileDto { nickname: "us".to_string(), email: "us_email".to_string(), password: "pas".to_string() })
                    .validate().err().unwrap()) )),
        (status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
        (status = 500, description = "Error while calculating the password hash.", body = AppError, 
            example = json!(AppError::internal_err500(&format!("{}; {}", err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
        (status = 510, description = "Error sending email.", body = AppError,
            example = json!(AppError::not_extended510(&format!("{}: {}", MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded.")))),
    ),
)]
#[post("/api/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_body: web::Json<RegistrProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let mut registr_profile_dto: RegistrProfileDto = json_body.into_inner();
    registr_profile_dto.nickname = registr_profile_dto.nickname.to_lowercase();
    registr_profile_dto.email = registr_profile_dto.email.to_lowercase();

    let password = registr_profile_dto.password.clone();
    let password_hashed = hash_tools::encode_hash(&password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_ERROR_HASHING_PASSWORD, e.to_string());
        log::error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
        AppError::internal_err500(&message) // 500
    })?;

    let nickname = registr_profile_dto.nickname.clone();
    let email = registr_profile_dto.email.clone();

    let profile_orm2 = profile_orm.get_ref().clone();
    let registr_orm2 = user_registr_orm.get_ref().clone();

    let res_search = profile_checks::uniqueness_nickname_or_email(
        Some(nickname), Some(email), profile_orm2, registr_orm2)
        .await
        .map_err(|err| {
            #[rustfmt::skip]
            let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
            log::error!("{}:{}; {}", &err.code, &err.message, &prm1);
            err
        })?;

    // Since the specified "nickname" or "email" is not unique, return an error.
    if let Some((is_nickname, _)) = res_search {
        #[rustfmt::skip]
        let message = if is_nickname { err::MSG_NICKNAME_ALREADY_USE } else { err::MSG_EMAIL_ALREADY_USE };
        log::error!("{}: {}", err::CD_CONFLICT, &message);
        return Err(AppError::conflict409(&message)); // 409
    }

    // If there is no such record, then add the specified data to the "user_registr" table.

    let app_registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
    // Waiting time for registration confirmation (in seconds).
    let final_date_utc = Utc::now() + Duration::seconds(app_registr_duration.into());

    let create_profile_registr_dto = user_models::CreateUserRegistr {
        nickname: registr_profile_dto.nickname.clone(),
        email: registr_profile_dto.email.clone(),
        password: password_hashed,
        final_date: final_date_utc,
    };
    // Create a new entity (user).
    let user_registr = web::block(move || {
        let user_registr = user_registr_orm.create_user_registr(create_profile_registr_dto).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        user_registr
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_registr.id, num_token) into a registr_token.
    let registr_token = encode_token(user_registr.id, num_token, jwt_secret, app_registr_duration).map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, e.to_string());
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let subject = format!("Account registration in {}", &config_app.app_name);
    let nickname = registr_profile_dto.nickname.clone();
    let receiver = registr_profile_dto.email.clone();
    let target = registr_token.clone();
    let registr_duration = app_registr_duration.clone() / 60; // Convert from seconds to minutes.
    let result = mailer.send_verification_code(&receiver, &domain, &subject, &nickname, &target, registr_duration);

    if result.is_err() {
        let message = format!("{}: {}", MSG_ERROR_SENDING_EMAIL, result.unwrap_err());
        log::error!("{}: {}", err::CD_NOT_EXTENDED, &message);
        return Err(AppError::not_extended510(&message)); // 510
    }

    let registr_profile_response_dto = RegistrProfileResponseDto {
        nickname: registr_profile_dto.nickname.clone(),
        email: registr_profile_dto.email.clone(),
        registr_token: registr_token.clone(),
    };

    Ok(HttpResponse::Created().json(registr_profile_response_dto)) // 201
}

/// confirm_registration
///
/// Confirmation of new user registration.
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/registration/registr_token1234
/// ```
///
/// Return the new user's profile. (`ProfileDto`) with status 201.
///
#[utoipa::path(
    responses(
        (status = 201, description = "New user profile.", body = ProfileDto,
            example = json!(ProfileDto::from(
            Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_LIGHT_DEF), None))
            )
        ),
        (status = 401, description = "The token is invalid or expired.", body = AppError,
            example = json!(AppError::unauthorized401(&format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken")))),
        (status = 404, description = "An entry for registering a new user was not found.", body = AppError,
            example = json!(AppError::not_found404(&format!("{}: user_registr_id: {}", MSG_REGISTR_NOT_FOUND, 123)))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("registr_token", description = "Registration token.")),
)]
#[put("/api/registration/{registr_token}")]
pub async fn confirm_registration(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let registr_token = request.match_info().query("registr_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “registr_token".
    let dual_token = decode_token(&registr_token, jwt_secret).map_err(|e| {
        let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401
    })?;

    // Get "user_registr ID" from "registr_token".
    let (user_registr_id, _) = dual_token;

    let user_registr_orm2 = user_registr_orm.clone();
    // Find a record with the specified ID in the “user_registr" table.
    let opt_user_registr = web::block(move || {
        let user_registr = user_registr_orm2.find_user_registr_by_id(user_registr_id).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        user_registr
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let user_registr_orm2 = user_registr_orm.clone();
    // Delete entries in the "user_registr" table, that are already expired.
    let _ = web::block(move || user_registr_orm2.delete_inactive_final_date(None)).await;

    // If no such entry exists, then exit with code 404.
    let user_registr = opt_user_registr.ok_or_else(|| {
        let message = format!("{}: user_registr_id: {}", MSG_REGISTR_NOT_FOUND, user_registr_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;

    // If such an entry exists, then add a new user.
    let create_profile = profile_models::CreateProfile::new(
        &user_registr.nickname,
        &user_registr.email,
        &user_registr.password,
        None,
    );

    let profile = web::block(move || {
        // Create a new entity (profile,user).
        let res_profile = profile_orm.create_profile_user(create_profile).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });

        res_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })??;

    let _ = web::block(move || {
        // Delete the processed record in the "user_registration" table.
        let _ = user_registr_orm.delete_user_registr(user_registr_id);
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        // An error during this operation has no effect.
    });

    let profile_dto = ProfileDto::from(profile);

    Ok(HttpResponse::Created().json(profile_dto)) // 201
}

/// recovery
/// 
/// Send a confirmation email to recover the user's password.
/// 
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/recovery \
/// -d '{"email": "user@email"}' \
/// -H 'Content-Type: application/json'
/// ```
/// Return new user registration parameters (`RecoveryProfileResponseDto`) with status 201.
/// 
#[utoipa::path(
    responses(
        (status = 201, description = "User password recovery options and recovery token.", body = RecoveryProfileResponseDto,
            example = json!(RecoveryProfileResponseDto {
                id: 27, email: "James_Miller@gmail.us".to_string(), recovery_token: TOKEN_RECOVERY.to_string() })
        ),
        (status = 404, description = "An entry to recover the user's password was not found.", body = AppError,
            example = json!(AppError::not_found404(&format!("{}: email: {}", MSG_USER_NOT_FOUND, "user@email")))),
        (status = 417, body = [AppError],
            description = "Validation error. `curl -i -X POST http://localhost:8080/api/recovery -d '{\"email\": \"us_email\" }'`",
            example = json!(AppError::validations((RecoveryProfileDto { email: "us_email".to_string() }).validate().err().unwrap()))),
        (status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
        (status = 510, description = "Error sending email.", body = AppError,
            example = json!(AppError::not_extended510(&format!("{}: {}", MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded.")))),
    ),
)]
#[post("/api/recovery")]
pub async fn recovery(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    json_body: web::Json<RecoveryProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let mut recovery_profile_dto: RecoveryProfileDto = json_body.into_inner();
    recovery_profile_dto.email = recovery_profile_dto.email.to_lowercase();
    let email = recovery_profile_dto.email.clone();

    // Find in the "user" table an entry by email.
    let opt_profile = web::block(move || {
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(None, Some(&email), false)
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

    // If such an entry does not exist, then exit with code 404.
    let profile = match opt_profile {
        Some(v) => v,
        None => {
            let message = format!("{}: email: {}", MSG_USER_NOT_FOUND, recovery_profile_dto.email.clone());
            log::error!("{}: {}", err::CD_NOT_FOUND, &message);
            return Err(AppError::not_found404(&message)); // 404
        }
    };
    let user_id = profile.user_id;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is a user with this ID, then move on to the next stage.

    // For this user, find an entry in the "user_recovery" table.
    let opt_user_recovery = web::block(move || {
        let existing_user_recovery = user_recovery_orm2
            .find_user_recovery_by_user_id(user_id)
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        existing_user_recovery
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    // Prepare data for writing to the "user_recovery" table.
    let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
    // Waiting time for password recovery confirmation (in seconds).
    let final_date_utc = Utc::now() + Duration::seconds(app_recovery_duration.into());

    let create_user_recovery = user_models::CreateUserRecovery {
        user_id: user_id,
        final_date: final_date_utc,
    };
    let user_recovery_id: i32;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is an entry for this user in the "user_recovery" table, then update it with a new token.
    if let Some(user_recovery) = opt_user_recovery {
        user_recovery_id = user_recovery.id;
        let _ = web::block(move || {
            let user_recovery = user_recovery_orm2
                .modify_user_recovery(user_recovery_id, create_user_recovery)
                .map_err(|e| {
                    log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e) // 507
                });
                user_recovery
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })??;
    } else {
        // If there is no entry for this user in the "user_recovery" table, then add a new entry.
        // Create a new entity (user_recovery).
        let user_recovery = web::block(move || {
            let user_recovery = user_recovery_orm2
                .create_user_recovery(create_user_recovery)
                .map_err(|e| {
                    log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e) // 507
                });
                user_recovery
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })??;

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
    .map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, e.to_string());
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Prepare a letter confirming this recovery.
    let domain = &config_app.app_domain;
    let subject = format!("Account recovery on {}", &config_app.app_name);
    let nickname = profile.nickname.clone();
    let receiver = profile.email.clone();
    let target = recovery_token.clone();
    let recovery_duration = app_recovery_duration.clone() / 60; // Convert from seconds to minutes.
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
        let message = format!("{}: {}", MSG_ERROR_SENDING_EMAIL, result.unwrap_err());
        log::error!("{}: {}", err::CD_NOT_EXTENDED, &message);
        return Err(AppError::not_extended510(&message)); // 510
    }

    let recovery_profile_response_dto = RecoveryProfileResponseDto {
        id: user_recovery_id,
        email: profile.email.clone(),
        recovery_token: recovery_token.clone(),
    };

    Ok(HttpResponse::Created().json(recovery_profile_response_dto)) // 201
}

/// confirm_recovery
///
/// Confirmation of user password recovery.
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/recovery/recovery_token1234 \
/// -d '{ "password": "new_password"}' \
/// -H 'Content-Type: application/json'
/// ```
///
/// Returns data about the user whose password was recovered (`ProfileDto`), with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Information about the user whose password was restored.", body = ProfileDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                        Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), None))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK), None))
            )))),
        ),
        (status = 401, description = "The token is invalid or expired.", body = AppError,
            example = json!(AppError::unauthorized401(&format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken")))),
        (status = 404, description = "Error: record not found.", body = AppError, examples(
            ("recovery" = (summary = "recovery_not_found",
                description = "An record to recover the user's password was not found.",
                value = json!(AppError::not_found404(&format!("{}: user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, 1234))))),
            ("user" = (summary = "user_not_found",
                description = "User not found.",
                value = json!(AppError::not_found404(&format!("{}: user_id: {}", MSG_USER_NOT_FOUND, 123)))))
        )),
        (status = 417, body = [AppError],
            description = "Validation error. `curl -i -X PUT http://localhost:8080/api/recovery/1234 -d '{ \"password\": \"pas\" }'`",
            example = json!(AppError::validations((RecoveryDataDto { password: "pas".to_string() }).validate().err().unwrap()) )),
        (status = 500, description = "Error while calculating the password hash.", body = AppError, 
            example = json!(AppError::internal_err500(&format!("{}: {}", err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("recovery_token", description = "Recovery token.")),
)]
#[put("/api/recovery/{recovery_token}")]
pub async fn confirm_recovery(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    json_body: web::Json<RecoveryDataDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let recovery_data_dto: RecoveryDataDto = json_body.into_inner();

    // Prepare a password hash.
    let password_hashed = hash_tools::encode_hash(&recovery_data_dto.password).map_err(|e| {
        let message = format!("{}: {}", err::MSG_ERROR_HASHING_PASSWORD, e.to_string());
        log::error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
        AppError::internal_err500(&message) // 500
    })?;

    let recovery_token = request.match_info().query("recovery_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “recovery_token".
    let dual_token = decode_token(&recovery_token, jwt_secret).map_err(|e| {
        let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401
    })?;

    // Get "user_recovery ID" from "recovery_token".
    let (user_recovery_id, _) = dual_token;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Find a record with the specified ID in the “user_recovery" table.
    let opt_user_recovery = web::block(move || {
        let user_recovery = user_recovery_orm2.get_user_recovery_by_id(user_recovery_id)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        user_recovery
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Delete entries in the "user_recovery" table, that are already expired.
    let _ = web::block(move || user_recovery_orm2.delete_inactive_final_date(None)).await;

    // If no such entry exists, then exit with code 404.
    let user_recovery = opt_user_recovery.ok_or_else(|| {
        let message = format!("{}: user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, user_recovery_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;
    let user_id = user_recovery.user_id;

    let profile_orm2 = profile_orm.clone();
    // If there is "user_recovery" with this ID, then move on to the next step.
    let opt_profile = web::block(move || {
        // Find profile by user id.
        let res_profile = profile_orm2.get_profile_user_by_id(user_id, false).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });

        res_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })??;

    // If no such entry exists, then exit with code 404.
    let profile = opt_profile.ok_or_else(|| {
        let message = format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;
    // Create a model to update the "password" field in the user profile.
    let modify_profile = profile_models::ModifyProfile{
        nickname: None, email: None, password: Some(password_hashed), role: None, avatar: None, descript: None, theme: None, locale: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(profile.user_id, modify_profile)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        opt_profile1
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    // If the user profile is updated successfully,
    // then delete the password recovery entry (table "user_recovery").
    if let Some(profile) = opt_profile {
        let user_recovery_orm2 = user_recovery_orm.clone();
        let _ = web::block(move || {
            // Delete entries in the “user_recovery" table.
            let user_recovery_res = user_recovery_orm2.delete_user_recovery(user_recovery_id);

            user_recovery_res
        })
        .await;    

        let profile_dto = ProfileDto::from(profile);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        let message = format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        Err(AppError::not_found404(&message)) // 404
    }
}

/// clear_for_expire
///
/// Clean up expired user registration and password recovery requests.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/clear_for_expired
/// ```
///
/// Returns the number of expired records deleted (`ClearForExpiredResponseDto`) with status 200.
///
/// The "admin" role is required.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The number of deleted user registration and expired password recovery records.",
            body = ClearForExpiredResponseDto, 
            example = json!(ClearForExpiredResponseDto { count_inactive_registr: 10, count_inactive_recover: 12 })
        ),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn clear_for_expired(
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Delete entries in the "user_registr" table, that are already expired.
    let count_inactive_registr_res = 
        web::block(move || user_registr_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
        ).await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })?;

    let count_inactive_registr = count_inactive_registr_res.unwrap_or(0);

    // Delete entries in the "user_recovery" table, that are already expired.
    let count_inactive_recover_res = 
        web::block(move || user_recovery_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
        ).await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })?;

    let count_inactive_recover = count_inactive_recover_res.unwrap_or(0);

    let clear_for_expired_response_dto = ClearForExpiredResponseDto {
        count_inactive_registr,
        count_inactive_recover,
    };
    
    Ok(HttpResponse::Ok().json(clear_for_expired_response_dto)) // 200
}


#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;

    use crate::errors::AppError;
    use crate::profiles::{
        profile_models::{self, Profile, ProfileTest},
        profile_err as p_err,
    };
    use crate::send_email::config_smtp;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::decode_token,
    };
    use crate::settings::{config_app, err};
    use crate::users::{
        user_models::{UserRecovery, UserRegistr, UserRole},
        user_recovery_orm::tests::UserRecoveryOrmApp,
        user_registr_orm::tests::UserRegistrOrmApp,
    };
    use crate::utils::token::BEARER;

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_profile() -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role)
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_user_registr() -> UserRegistr {
        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);

        let user_registr =
            UserRegistrOrmApp::new_user_registr(1, "Robert_Brown", "Robert_Brown@gmail.com", "passwdR2B2", final_date);
        user_registr
    }
    fn user_registr_with_id(user_registr: UserRegistr) -> UserRegistr {
        let user_reg_orm = UserRegistrOrmApp::create(&vec![user_registr]);
        user_reg_orm.user_registr_vec.get(0).unwrap().clone()
    }
    fn create_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
        UserRecoveryOrmApp::new_user_recovery(id, user_id, final_date)
    }
    fn create_user_recovery_with_id(user_recovery: UserRecovery) -> UserRecovery {
        let user_recovery_orm = UserRecoveryOrmApp::create(&vec![user_recovery]);
        user_recovery_orm.user_recovery_vec.get(0).unwrap().clone()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data(is_registr: bool, opt_recovery_duration: Option<i64>) -> (
        (config_app::ConfigApp, config_jwt::ConfigJwt), 
        (Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<UserRecovery>),
        String
    ) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile());
        let user_id = profile1.user_id;
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let user_registr_vec:Vec<UserRegistr> = if is_registr {
            vec![user_registr_with_id(create_user_registr())]
        } else { vec![] };

        let user_recovery_vec:Vec<UserRecovery> = if let Some(recovery_duration) = opt_recovery_duration {
            let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
            let user_recovery = UserRecoveryOrmApp::new_user_recovery(1, user_id, final_date_utc);
            UserRecoveryOrmApp::create(&vec![user_recovery]).user_recovery_vec
        } else { vec![] };

        let config_app = config_app::get_test_config();
        let cfg_c = (config_app, config_jwt);
        let data_c = (vec![profile1], vec![session1], user_registr_vec,  user_recovery_vec);

        (cfg_c, data_c, token)
    }
    fn configure_reg(
        cfg_c: (config_app::ConfigApp, config_jwt::ConfigJwt), // cortege of configurations
        data_c: (
            Vec<Profile>,
            Vec<Session>,
            Vec<UserRegistr>,
            Vec<UserRecovery>,
        ), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_app = web::Data::new(cfg_c.0);
            let data_config_jwt = web::Data::new(cfg_c.1);
            let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));

            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.2));
            let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(&data_c.3));

            config
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm));
        }
    }
    fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** registration **
    #[actix_web::test]
    async fn test_registration_no_data() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration").to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Content type error"));
    }
    #[actix_web::test]
    async fn test_registration_empty_json_object() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration").set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Json deserialize error: missing field"));
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_min(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_max(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: ProfileTest::nickname_wrong(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "".to_string(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_min(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_max(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: ProfileTest::email_wrong(),
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: "".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_min(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_max(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_registration_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: "Oliver_Taylor".to_string(),
                email: "Oliver_Taylor@gmail.com".to_string(),
                password: ProfileTest::password_wrong(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_users() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let nickname1 = data_c.0.get(0).unwrap().nickname.clone();
        let email1 = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: nickname1, email: format!("A{}", email1), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, err::MSG_NICKNAME_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_email_exists_in_users() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let nickname1 = data_c.0.get(0).unwrap().nickname.clone();
        let email1 = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: format!("A{}", nickname1), email: email1, password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, err::MSG_EMAIL_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_nickname_exists_in_registr() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let nickname1 = data_c.2.get(0).unwrap().nickname.clone();
        let email1 = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: nickname1, email: format!("A{}", email1), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, err::MSG_NICKNAME_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_if_email_exists_in_registr() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let nickname1 = data_c.2.get(0).unwrap().nickname.clone();
        let email1 = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: format!("A{}", nickname1), email: email1, password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert_eq!(app_err.message, err::MSG_EMAIL_ALREADY_USE);
    }
    #[actix_web::test]
    async fn test_registration_err_jsonwebtoken_encode() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let mut config_jwt = cfg_c.1;
        config_jwt.jwt_secret = "".to_string();
        let cfg_c = (cfg_c.0, config_jwt);
        let nickname = "Mary_Williams".to_string();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: nickname.clone(), email: format!("{}@gmail.com", nickname), password: "passwordD2T2".to_string(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNPROCESSABLE_ENTITY); // 422

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNPROCESSABLE_ENTITY);
        assert!(app_err.message.starts_with(&format!("{};", p_err::MSG_JSON_WEB_TOKEN_ENCODE)));
    }
    #[actix_web::test]
    async fn test_registration_new_user() {
        let user_registr1 = user_registr_with_id(create_user_registr());
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/registration")
            .set_json(RegistrProfileDto {
                nickname: user_registr1.nickname.clone(),
                email: user_registr1.email.clone(),
                password: user_registr1.password.clone(),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let registr_profile_resp: RegistrProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(user_registr1.nickname, registr_profile_resp.nickname);
        assert_eq!(user_registr1.email, registr_profile_resp.email);

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let (user_registr_id, _) = decode_token(&registr_profile_resp.registr_token, jwt_secret).unwrap();
        assert_eq!(user_registr1.id, user_registr_id);
    }

    // ** confirm_registration **
    #[actix_web::test]
    async fn test_confirm_registration_invalid_registr_token() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", "invalid_registr_token"))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_confirm_registration_final_date_has_expired() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = encode_token(user_reg1_id, num_token, jwt_secret, -reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "ExpiredSignature"));
    }
    #[actix_web::test]
    async fn test_confirm_registration_no_exists_in_user_regist() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let user_reg1_id = user_reg1.id;

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let user_reg_id = user_reg1_id + 1;
        let registr_token = encode_token(user_reg_id, num_token, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_registr_id: {}", MSG_REGISTR_NOT_FOUND, user_reg_id));
    }
    #[actix_web::test]
    async fn test_confirm_registration_exists_in_user_regist() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, None);
        let last_user_id = data_c.0.last().unwrap().user_id;
        let user_reg1 = data_c.2.get(0).unwrap().clone();
        let nickname = user_reg1.nickname.to_string();
        let email = user_reg1.email.to_string();

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let registr_token = encode_token(user_reg1.id, num_token, jwt_secret, reg_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_registration).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/registration/{}", registr_token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, last_user_id + 1);
        assert_eq!(profile_dto_res.nickname, nickname);
        assert_eq!(profile_dto_res.email, email);
        assert_eq!(profile_dto_res.role, UserRole::User);
    }

    // ** recovery **
    #[actix_web::test]
    async fn test_recovery_no_data() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_recovery_empty_json_object() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery").set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Json deserialize error: missing field"));
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_recovery_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: ProfileTest::email_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_with_email_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let email = format!("A{}", data_c.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, format!("{}: email: {}", MSG_USER_NOT_FOUND, &email.to_lowercase()));
    }
    #[actix_web::test]
    async fn test_recovery_if_user_recovery_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let user1_id = data_c.0.get(0).unwrap().user_id;
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let final_date_utc = Utc::now() + Duration::seconds(600);
        let user_recovery1 = create_user_recovery_with_id(create_user_recovery(0, user1_id, final_date_utc));
        let user_recovery1_id = user_recovery1.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery_id, user_recovery1_id);
    }
    #[actix_web::test]
    async fn test_recovery_if_user_recovery_already_exists() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let user_recovery1_id = user_recovery1.id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_recov_res: RecoveryProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) = decode_token(&recovery_token, jwt_secret).expect("decode_token error");
        assert_eq!(user_recovery1_id, user_recovery_id);
    }
    #[actix_web::test]
    async fn test_recovery_err_jsonwebtoken_encode() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user1_email = data_c.0.get(0).unwrap().email.clone();
        let mut config_jwt = cfg_c.1;
        config_jwt.jwt_secret = "".to_string();
        let cfg_c = (cfg_c.0, config_jwt);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/recovery")
            .set_json(RecoveryProfileDto { email: user1_email.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNPROCESSABLE_ENTITY); // 422

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNPROCESSABLE_ENTITY);
        assert!(app_err.message.starts_with(&format!("{};", p_err::MSG_JSON_WEB_TOKEN_ENCODE)));
    }

    // ** confirm_recovery **
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: ProfileTest::password_wrong() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_recovery_token() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "invalid_recovery_token"))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_final_date_has_expired() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.3.get(0).unwrap().clone();

        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, -recovery_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(
            app_err.message,
            format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "ExpiredSignature")
        );
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user_recovery() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let user_recovery_id = user_recovery1.id + 1;
        let num_token = data_c.1.get(0).unwrap().clone().num_token.unwrap();
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = encode_token(user_recovery_id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, user_recovery_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let user_id = data_c.0.get(0).unwrap().user_id + 1;

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
        let user_recovery1 = create_user_recovery_with_id(create_user_recovery(0, user_id, final_date_utc));
        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();

        let data_c = (data_c.0, data_c.1, data_c.2, vec![user_recovery1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_success() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let profile1_dto = ProfileDto::from(data_c.0.get(0).unwrap().clone());
        let user_recovery1 = data_c.3.get(0).unwrap().clone();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();

        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile1_dto_ser.id);
        assert_eq!(profile_dto_res.nickname, profile1_dto_ser.nickname);
        assert_eq!(profile_dto_res.email, profile1_dto_ser.email);
        assert_eq!(profile_dto_res.role, profile1_dto_ser.role);
        assert_eq!(profile_dto_res.avatar, profile1_dto_ser.avatar);
        assert_eq!(profile_dto_res.descript, profile1_dto_ser.descript);
        assert_eq!(profile_dto_res.theme, profile1_dto_ser.theme);
        assert_eq!(profile_dto_res.created_at, profile1_dto_ser.created_at);
    }

    // ** clear_for_expired **
    #[actix_web::test]
    async fn test_clear_for_expired_user_recovery() {
        let (cfg_c, data_c, token) = get_cfg_data(true, Some(600));

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.role = UserRole::Admin;

        let registr_duration: i64 = cfg_c.0.app_registr_duration.try_into().unwrap();
        let final_date_registr = Utc::now() - Duration::seconds(registr_duration);
        let mut user_registr1 = data_c.2.get(0).unwrap().clone();
        user_registr1.final_date = final_date_registr;

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_recovery = Utc::now() - Duration::seconds(recovery_duration);
        let mut user_recovery1 = data_c.3.get(0).unwrap().clone();
        user_recovery1.final_date = final_date_recovery;
        #[rustfmt::skip]
        let data_c = (vec![profile1], data_c.1, vec![user_registr1], vec![user_recovery1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(clear_for_expired).configure(configure_reg(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&"/api/clear_for_expired")
            .insert_header(header_auth(&token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto: ClearForExpiredResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto.count_inactive_registr, 1);
        assert_eq!(response_dto.count_inactive_recover, 1);
    }
}

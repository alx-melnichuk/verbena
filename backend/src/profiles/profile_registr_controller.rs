use actix_web::{get, http::StatusCode, post, put, web, HttpResponse};
use chrono::{Duration, Utc};
use log::error;
use utoipa;
use vrb_authentication::authentication::RequireAuth;
use vrb_common::{
    api_error::{code_to_str, ApiError},
    validators::{msg_validation, Validator},
};
use vrb_dbase::{db_enums::UserRole, user_auth::config_jwt};
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_tools::send_email::mailer::impls::MailerApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_tools::send_email::mailer::tests::MailerApp;
use vrb_tools::{config_app, err, hash_tools, send_email::mailer::Mailer, token_coding};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    profile_check,
    profile_models::{
        self, ClearForExpiredResponseDto, Profile, ProfileDto, RecoveryDataDto, RecoveryProfileDto, RecoveryProfileResponseDto,
        RegistrProfileDto, RegistrProfileResponseDto, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_recovery_orm::impls::UserRecoveryOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_recovery_orm::tests::UserRecoveryOrmApp;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::{user_models, user_recovery_orm::UserRecoveryOrm, user_registr_orm::UserRegistrOrm};

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
        (status = 409, description = "Error: nickname (email) is already in use.", body = ApiError, examples(
            ("Nickname" = (summary = "Nickname already used",
                description = "The nickname value has already been used.",
                value = json!(ApiError::new(409, err::MSG_NICKNAME_ALREADY_USE)))),
            ("Email" = (summary = "Email already used", 
                description = "The email value has already been used.",
                value = json!(ApiError::new(409, err::MSG_EMAIL_ALREADY_USE))))
        )),
        (status = 417, body = [ApiError], description = "Validation error. `curl -i
             -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"email\": \"us_email\", \"password\": \"pas\" }'`",
            example = json!(ApiError::validations(
                (RegistrProfileDto { nickname: "us".to_string(), email: "us_email".to_string(), password: "pas".to_string() })
                    .validate().err().unwrap()) )),
        (status = 422, description = "Token encoding error.", body = ApiError,
            example = json!(ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat"))),
        (status = 500, description = "Error while calculating the password hash.", body = ApiError, 
            example = json!(ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error sending email.", body = ApiError,
            example = json!(ApiError::create(510, MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded."))),
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
) -> actix_web::Result<HttpResponse, ApiError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let mut registr_profile_dto: RegistrProfileDto = json_body.into_inner();
    registr_profile_dto.nickname = registr_profile_dto.nickname.to_lowercase();
    registr_profile_dto.email = registr_profile_dto.email.to_lowercase();

    let password = registr_profile_dto.password.clone();
    let password_hashed = hash_tools::encode_hash(&password).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_HASHING_PASSWORD, &e);
        ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, &e) // 500
    })?;

    let nickname = registr_profile_dto.nickname.clone();
    let email = registr_profile_dto.email.clone();

    let profile_orm2 = profile_orm.get_ref().clone();
    let registr_orm2 = user_registr_orm.get_ref().clone();

    let res_search = profile_check::uniqueness_nickname_or_email(Some(nickname), Some(email), profile_orm2, registr_orm2)
        .await
        .map_err(|err| {
            #[rustfmt::skip]
            let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
            error!("{}:{}; {}", &err.code, &err.message, &prm1);
            err
        })?;

    // Since the specified "nickname" or "email" is not unique, return an error.
    if let Some((is_nickname, _)) = res_search {
        #[rustfmt::skip]
        let message = if is_nickname { err::MSG_NICKNAME_ALREADY_USE } else { err::MSG_EMAIL_ALREADY_USE };
        error!("{}-{}", code_to_str(StatusCode::CONFLICT), &message);
        return Err(ApiError::new(409, &message)); // 409
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
        #[rustfmt::skip]
        let user_registr = user_registr_orm.create_user_registr(create_profile_registr_dto)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        user_registr
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    let num_token = token_coding::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_registr.id, num_token) into a registr_token.
    #[rustfmt::skip]
    let registr_token = token_coding::encode_token(user_registr.id, num_token, jwt_secret, app_registr_duration)
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
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
        let e = result.unwrap_err();
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), MSG_ERROR_SENDING_EMAIL, &e);
        return Err(ApiError::create(510, MSG_ERROR_SENDING_EMAIL, &e)); // 510
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
        (status = 401, description = "The token is invalid or expired.", body = ApiError,
            example = json!(ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken"))),
        (status = 404, description = "An entry for registering a new user was not found.", body = ApiError,
            example = json!(ApiError::create(404, MSG_REGISTR_NOT_FOUND, "user_registr_id: 123"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("registr_token", description = "Registration token.")),
)]
#[put("/api/registration/{registr_token}")]
pub async fn confirm_registration(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let registr_token = request.match_info().query("registr_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “registr_token".
    let dual_token = token_coding::decode_token(&registr_token, jwt_secret).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, &e) // 401
    })?;

    // Get "user_registr ID" from "registr_token".
    let (user_registr_id, _) = dual_token;

    let user_registr_orm2 = user_registr_orm.clone();
    // Find a record with the specified ID in the “user_registr" table.
    let opt_user_registr = web::block(move || {
        let user_registr = user_registr_orm2.find_user_registr_by_id(user_registr_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        user_registr
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    let user_registr_orm2 = user_registr_orm.clone();
    // Delete entries in the "user_registr" table, that are already expired.
    let _ = web::block(move || user_registr_orm2.delete_inactive_final_date(None)).await;

    // If no such entry exists, then exit with code 404.
    let user_registr = opt_user_registr.ok_or_else(|| {
        let msg = format!("user_registr_id: {}", user_registr_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_REGISTR_NOT_FOUND, &msg);
        ApiError::create(404, MSG_REGISTR_NOT_FOUND, &msg) // 404
    })?;

    // If such an entry exists, then add a new user.
    let create_profile = profile_models::CreateProfile::new(&user_registr.nickname, &user_registr.email, &user_registr.password, None);

    let profile = web::block(move || {
        // Create a new entity (profile,user).
        let res_profile = profile_orm.create_profile_user(create_profile).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });

        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string())
    })??;

    let _ = web::block(move || {
        // Delete the processed record in the "user_registration" table.
        let _ = user_registr_orm.delete_user_registr(user_registr_id);
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
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
        (status = 404, description = "An entry to recover the user's password was not found.", body = ApiError,
            example = json!(ApiError::create(404, MSG_USER_NOT_FOUND, "email: user@email"))),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X POST http://localhost:8080/api/recovery -d '{\"email\": \"us_email\" }'`",
            example = json!(ApiError::validations((RecoveryProfileDto { email: "us_email".to_string() }).validate().err().unwrap()))),
        (status = 422, description = "Token encoding error.", body = ApiError,
            example = json!(ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error sending email.", body = ApiError,
            example = json!(ApiError::create(510, MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded."))),
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
) -> actix_web::Result<HttpResponse, ApiError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let mut recovery_profile_dto: RecoveryProfileDto = json_body.into_inner();
    recovery_profile_dto.email = recovery_profile_dto.email.to_lowercase();
    let email = recovery_profile_dto.email.clone();

    // Find in the "user" table an entry by email.
    let opt_profile = web::block(move || {
        let existing_profile = profile_orm.find_profile_by_nickname_or_email(None, Some(&email), false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        existing_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    // If such an entry does not exist, then exit with code 404.
    let profile = match opt_profile {
        Some(v) => v,
        None => {
            let msg = format!("email: {}", recovery_profile_dto.email.clone());
            error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
            return Err(ApiError::create(404, MSG_USER_NOT_FOUND, &msg)); // 404
        }
    };
    let user_id = profile.user_id;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is a user with this ID, then move on to the next stage.

    // For this user, find an entry in the "user_recovery" table.
    let opt_user_recovery = web::block(move || {
        let existing_user_recovery = user_recovery_orm2.find_user_recovery_by_user_id(user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        existing_user_recovery
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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
                    error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                    ApiError::create(507, err::MSG_DATABASE, &e) // 507
                });
            user_recovery
        })
        .await
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })??;
    } else {
        // If there is no entry for this user in the "user_recovery" table, then add a new entry.
        // Create a new entity (user_recovery).
        let user_recovery = web::block(move || {
            let user_recovery = user_recovery_orm2.create_user_recovery(create_user_recovery).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
            user_recovery
        })
        .await
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })??;

        user_recovery_id = user_recovery.id;
    }

    let num_token = token_coding::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_recovery_id, num_token) into a recovery_token.
    let recovery_token = token_coding::encode_token(user_recovery_id, num_token, jwt_secret, app_recovery_duration).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    // Prepare a letter confirming this recovery.
    let domain = &config_app.app_domain;
    let subject = format!("Account recovery on {}", &config_app.app_name);
    let nickname = profile.nickname.clone();
    let receiver = profile.email.clone();
    let target = recovery_token.clone();
    let recovery_duration = app_recovery_duration.clone() / 60; // Convert from seconds to minutes.

    // Send an email to this user.
    let result = mailer.send_password_recovery(&receiver, &domain, &subject, &nickname, &target, recovery_duration);

    if result.is_err() {
        let msg = result.unwrap_err();
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), MSG_ERROR_SENDING_EMAIL, &msg);
        return Err(ApiError::create(510, MSG_ERROR_SENDING_EMAIL, &msg)); // 510
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
        (status = 401, description = "The token is invalid or expired.", body = ApiError,
            example = json!(ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken"))),
        (status = 404, description = "Error: record not found.", body = ApiError, examples(
            ("recovery" = (summary = "recovery_not_found",
                description = "An record to recover the user's password was not found.",
                value = json!(ApiError::create(404, MSG_RECOVERY_NOT_FOUND, "user_recovery_id: 1234")))),
            ("user" = (summary = "user_not_found",
                description = "User not found.",
                value = json!(ApiError::create(404, MSG_USER_NOT_FOUND, "user_id: 123"))))
        )),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X PUT http://localhost:8080/api/recovery/1234 -d '{ \"password\": \"pas\" }'`",
            example = json!(ApiError::validations((RecoveryDataDto { password: "pas".to_string() }).validate().err().unwrap()) )),
        (status = 500, description = "Error while calculating the password hash.", body = ApiError, 
            example = json!(ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
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
) -> actix_web::Result<HttpResponse, ApiError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let recovery_data_dto: RecoveryDataDto = json_body.into_inner();

    // Prepare a password hash.
    let password_hashed = hash_tools::encode_hash(&recovery_data_dto.password).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_HASHING_PASSWORD, &e);
        ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, &e) // 500
    })?;

    let recovery_token = request.match_info().query("recovery_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “recovery_token".
    let dual_token = token_coding::decode_token(&recovery_token, jwt_secret).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, &e) // 401
    })?;

    // Get "user_recovery ID" from "recovery_token".
    let (user_recovery_id, _) = dual_token;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Find a record with the specified ID in the “user_recovery" table.
    let opt_user_recovery = web::block(move || {
        let user_recovery = user_recovery_orm2.get_user_recovery_by_id(user_recovery_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        user_recovery
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Delete entries in the "user_recovery" table, that are already expired.
    let _ = web::block(move || user_recovery_orm2.delete_inactive_final_date(None)).await;

    // If no such entry exists, then exit with code 404.
    let user_recovery = opt_user_recovery.ok_or_else(|| {
        let msg = format!("user_recovery_id: {}", user_recovery_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_RECOVERY_NOT_FOUND, &msg);
        ApiError::create(404, MSG_RECOVERY_NOT_FOUND, &msg) // 404
    })?;
    let user_id = user_recovery.user_id;

    let profile_orm2 = profile_orm.clone();
    // If there is "user_recovery" with this ID, then move on to the next step.
    let opt_profile = web::block(move || {
        // Find profile by user id.
        let res_profile = profile_orm2.get_profile_user_by_id(user_id, false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });

        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) //506
    })??;

    // If no such entry exists, then exit with code 404.
    let profile = opt_profile.ok_or_else(|| {
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
        ApiError::create(404, MSG_USER_NOT_FOUND, &msg) // 404
    })?;
    // Create a model to update the "password" field in the user profile.
    #[rustfmt::skip]
    let modify_profile = profile_models::ModifyProfile {
        nickname: None, email: None, password: Some(password_hashed), role: None,  avatar: None, descript: None, theme: None, locale: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(profile.user_id, modify_profile).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        opt_profile1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
        Err(ApiError::create(404, MSG_USER_NOT_FOUND, &msg)) // 404
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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn clear_for_expired(
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Delete entries in the "user_registr" table, that are already expired.
    let count_inactive_registr_res = 
        web::block(move || user_registr_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
        ).await
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;

    let count_inactive_registr = count_inactive_registr_res.unwrap_or(0);

    // Delete entries in the "user_recovery" table, that are already expired.
    let count_inactive_recover_res = 
        web::block(move || user_recovery_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        })
        ).await
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
        })?;

    let count_inactive_recover = count_inactive_recover_res.unwrap_or(0);

    let clear_for_expired_response_dto = ClearForExpiredResponseDto {
        count_inactive_registr,
        count_inactive_recover,
    };
    
    Ok(HttpResponse::Ok().json(clear_for_expired_response_dto)) // 200
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::{http, web};
    use vrb_common::api_error::ApiError;
    use vrb_tools::{
        config_app,
        send_email::{config_smtp, mailer::tests::MailerApp},
        token_data::BEARER,
    };

    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }

    pub fn cfg_config_app(config_app: config_app::ConfigApp) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_app = web::Data::new(config_app);
            config.app_data(web::Data::clone(&data_config_app));
        }
    }

    pub fn cfg_mailer(config_smtp: config_smtp::ConfigSmtp) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_mailer = web::Data::new(MailerApp::new(config_smtp));
            config.app_data(web::Data::clone(&data_mailer));
        }
    }

    pub fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
}

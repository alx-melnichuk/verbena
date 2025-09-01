use std::time::Instant as tm;

use actix_web::{get, http::StatusCode, post, put, web, HttpResponse};
use chrono::{Duration, Utc};
use log::{error, info, log_enabled, Level::Info};
use utoipa;
use vrb_common::{
    api_error::{code_to_str, ApiError},
    err,
    validators::{msg_validation, Validator},
};
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_tools::send_email::mailer::impls::MailerApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_tools::send_email::mailer::tests::MailerApp;
use vrb_tools::{
    config_app, hash_tools,
    send_email::{config_smtp, mailer::Mailer},
    token_coding,
};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::user_orm::impls::UserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::user_orm::tests::UserOrmApp;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::user_recovery_orm::impls::UserRecoveryOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::user_recovery_orm::tests::UserRecoveryOrmApp;

use crate::{
    authentication::RequireAuth,
    config_jwt,
    user_models::ModifyUser,
    user_orm::UserOrm,
    user_recovery_models::{
        ConfirmRecoveryUserResponseDto, CreateUserRecovery, RecoveryClearForExpiredResponseDto, RecoveryDataDto, RecoveryUserDto,
        RecoveryUserResponseDto,
    },
    user_recovery_orm::UserRecoveryOrm,
};

// 404 Not Found - Recovery record not found.
pub const MSG_RECOVERY_NOT_FOUND: &str = "recovery_not_found";
// 404 Not Found - User not found.
pub const MSG_USER_NOT_FOUND: &str = "user_not_found";

const TOKEN_RECOVERY: &str = concat!(
    "BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT_allv-RMwbJsI67Aufl3gCD7pyis6TI9VxbRkc_jx3yORspBCqVJ8yuNNRcZox0sr9XLWlbIYKLR7yv",
    "H16LMFEmE1fuP7fu4hdc4XRk7j9XTd2pOxry-jHiPMJ-ijWKhZrXORmac2KFLMf6dUO2qWh7LcmpD73WWfqoAqMPUnAUDreY_xwkZ3wofSUXHsloucUbk=",
);

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/recovery
            .service(recovery)
            // PUT /api/recovery/{recovery_token}
            .service(confirm_recovery)
            // GET /api/recovery/clear_for_expired
            .service(recovery_clear_for_expired);
    }
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
/// Return new user registration parameters (`RecoveryUserResponseDto`) with status 201.
///
#[utoipa::path(
    responses(
        (status = 201, description = "User password recovery options and recovery token.", body = RecoveryUserResponseDto,
            example = json!(RecoveryUserResponseDto {
                id: 27, email: "James_Miller@gmail.us".to_string(), recovery_token: TOKEN_RECOVERY.to_string() })
        ),
        (status = 404, description = "An entry to recover the user's password was not found.", body = ApiError,
            example = json!(ApiError::create(404, MSG_USER_NOT_FOUND, "email: user@email"))),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X POST http://localhost:8080/api/recovery -d '{\"email\": \"us_email\" }'`",
            example = json!(ApiError::validations((RecoveryUserDto { email: "us_email".to_string() }).validate().err().unwrap()))),
        (status = 422, description = "Token encoding error.", body = ApiError,
            example = json!(ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error sending email.", body = ApiError,
            example = json!(ApiError::create(510, err::MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded."))),
    ),
)]
#[post("/api/recovery")]
pub async fn recovery(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    config_smtp: web::Data<config_smtp::ConfigSmtp>,
    user_orm: web::Data<UserOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    json_body: web::Json<RecoveryUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let mut recovery_profile_dto: RecoveryUserDto = json_body.into_inner();
    recovery_profile_dto.email = recovery_profile_dto.email.to_lowercase();
    let email = recovery_profile_dto.email.clone();

    // Find in the "user" table an entry by email.
    let opt_user = web::block(move || {
        let existing_user = user_orm.find_user_by_nickname_or_email(None, Some(&email), false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        existing_user
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    // If such an entry does not exist, then exit with code 404.
    let user = match opt_user {
        Some(v) => v,
        None => {
            let msg = format!("email: {}", recovery_profile_dto.email.clone());
            error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
            return Err(ApiError::create(404, MSG_USER_NOT_FOUND, &msg)); // 404
        }
    };
    let user_id = user.id;
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

    let create_user_recovery = CreateUserRecovery {
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

    let config_smtp = config_smtp.get_ref().clone();
    let path_template = config_smtp.smtp_path_template;
    // Prepare a letter confirming this recovery.
    let domain = &config_app.app_domain;
    let subject = format!("Account recovery on {}", &config_app.app_name);
    let nickname = user.nickname.clone();
    let receiver = user.email.clone();
    let target = recovery_token.clone();
    let recovery_duration = app_recovery_duration.clone() / 60; // Convert from seconds to minutes.

    // Send an email to this user.
    #[rustfmt::skip]
    let result = mailer.send_password_recovery(
        &path_template, &receiver, &domain, &subject, &nickname, &target, recovery_duration);

    if result.is_err() {
        let msg = result.unwrap_err();
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), err::MSG_ERROR_SENDING_EMAIL, &msg);
        return Err(ApiError::create(510, err::MSG_ERROR_SENDING_EMAIL, &msg)); // 510
    }

    let recovery_profile_response_dto = RecoveryUserResponseDto {
        id: user_recovery_id,
        email: user.email.clone(),
        recovery_token: recovery_token.clone(),
    };
    if let Some(timer) = timer {
        info!("recovery() time: {}", format!("{:.2?}", timer.elapsed()));
    }
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
/// Returns data about the user whose password was recovered (`ConfirmRecoveryUserResponseDto`), with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Information about the user whose password was restored.", body = ConfirmRecoveryUserResponseDto,
            example = json!(ConfirmRecoveryUserResponseDto { id: 120, nickname: "james_miller".to_owned()
                , email: "james_miller@gmail.us".to_owned(), created_at: Utc::now()
                , updated_at: (Utc::now() - Duration::hours(2) - Duration::minutes(30)) })
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
    user_orm: web::Data<UserOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    json_body: web::Json<RecoveryDataDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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

    let user_orm2 = user_orm.clone();
    // If there is "user_recovery" with this ID, then move on to the next step.
    let opt_user = web::block(move || {
        // Find profile by user id.
        let res_user = user_orm2.get_user_by_id(user_id, false).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });

        res_user
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) //506
    })??;

    // If no such entry exists, then exit with code 404.
    let user = opt_user.ok_or_else(|| {
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
        ApiError::create(404, MSG_USER_NOT_FOUND, &msg) // 404
    })?;
    // Create a model to update the "password" field in the user profile.
    let modify_user = ModifyUser {
        nickname: None,
        email: None,
        password: Some(password_hashed),
        role: None,
    };
    // Update the password hash for the user profile.
    let opt_user = web::block(move || {
        let opt_user1 = user_orm.modify_user(user.id, modify_user).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        opt_user1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    // If the user is updated successfully,
    // then delete the password recovery entry (table "user_recovery").
    if let Some(user) = opt_user {
        let user_recovery_orm2 = user_recovery_orm.clone();
        let _ = web::block(move || {
            // Delete entries in the “user_recovery" table.
            let user_recovery_res = user_recovery_orm2.delete_user_recovery(user_recovery_id);

            user_recovery_res
        })
        .await;

        let response_dto = ConfirmRecoveryUserResponseDto {
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };
        if let Some(timer) = timer {
            info!("confirm_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
        }
        Ok(HttpResponse::Ok().json(response_dto)) // 200
    } else {
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_FOUND), MSG_USER_NOT_FOUND, &msg);
        if let Some(timer) = timer {
            info!("confirm_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
        }
        Err(ApiError::create(404, MSG_USER_NOT_FOUND, &msg)) // 404
    }
}

/// clear_for_expire
///
/// Clean up expired user registration and password recovery requests.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/recovery/clear_for_expired
/// ```
///
/// Returns the number (of expired) records deleted (`RecoveryClearForExpiredResponseDto`) with status 200.
///
/// The "admin" role is required.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The number of deleted outdated expired password recovery records.",
            body = RecoveryClearForExpiredResponseDto, 
            example = json!(RecoveryClearForExpiredResponseDto { count_inactive_recover: 2 })
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
#[get("/api/recovery/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn recovery_clear_for_expired(
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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

    let clear_for_expired_response_dto = RecoveryClearForExpiredResponseDto {
        count_inactive_recover,
    };
    if let Some(timer) = timer {
        info!("recovery_clear_for_expired() time: {}", format!("{:.2?}", timer.elapsed()));
    }
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
            let data_config_smtp = web::Data::new(config_smtp.clone());
            config.app_data(web::Data::clone(&data_config_smtp));

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

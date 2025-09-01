use std::{borrow::Cow, time::Instant as tm};

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
use crate::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::user_registr_orm::tests::UserRegistrOrmApp;
use crate::{
    authentication::RequireAuth,
    config_jwt,
    user_models::CreateUser,
    user_orm::UserOrm,
    user_registr_models::{
        ConfirmRegistrUserResponseDto, CreateUserRegistr, RegistrUserDto, RegistrUserResponseDto, RegistrationClearForExpiredResponseDto,
    },
    user_registr_orm::UserRegistrOrm,
};

// 404 Not Found - Registration record not found.
pub const MSG_REGISTR_NOT_FOUND: &str = "registration_not_found";

const TOKEN_REGISTR: &str = concat!(
    "BP3Y6aQTyguP2Q0Jzm9rQ1wdyZpODpz2H3QwCKT_allv-RMwbJsI67Aufl3gCD7pkSNU4zJWr2eROQ-6xGjDwibujOTTf6xZXV29k3E8ODzFqbkk2Pty4W",
    "puSI6YG-d6XDi4s4yczfWgfYM65kKjrD3FxGSa15zBXOR2yxrEXeziVyV2bbnDu1-Uex0_Pqg0zkHvu_k--L2D0xP4o_m5WeBV539-1YTNeExQ0A9_Pv8=",
);

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/registration
            .service(registration)
            // PUT /api/registration/{registr_token}
            .service(confirm_registration)
            // GET /api/registration/clear_for_expired
            .service(registration_clear_for_expired);
    }
}

// Check if the nickname and email parameters are specified.
fn is_nickname_email_params_not_specified(opt_nickname: Option<&str>, opt_email: Option<&str>) -> Result<(), ApiError> {
    let nickname = opt_nickname.unwrap_or("");
    let email = opt_email.unwrap_or("");
    #[rustfmt::skip]
    let opt_nickname = if nickname.len() > 0 { Some(nickname) } else { None };
    let opt_email = if email.len() > 0 { Some(email) } else { None };

    if opt_nickname.is_none() && opt_email.is_none() {
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        return Err(ApiError::new(406, err::MSG_PARAMS_NOT_SPECIFIED) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json));
    }
    Ok(())
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
/// Return new user registration parameters (`RegistrUserResponseDto`) with status 201.
///
#[utoipa::path(
    responses(
        (status = 201, description = "New user registration parameters and registration token.", body = RegistrUserResponseDto,
            example = json!(RegistrUserResponseDto { nickname:"Emma_Johnson".to_string(), email:"Emma_Johnson@gmail.us".to_string(),registr_token: TOKEN_REGISTR.to_string() })
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
                (RegistrUserDto { nickname: "us".to_string(), email: "us_email".to_string(), password: "pas".to_string() })
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
            example = json!(ApiError::create(510, err::MSG_ERROR_SENDING_EMAIL, "The mail server is overloaded."))),
    ),
)]
#[post("/api/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    config_smtp: web::Data<config_smtp::ConfigSmtp>,
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_body: web::Json<RegistrUserDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let mut registr_user_dto: RegistrUserDto = json_body.into_inner();
    registr_user_dto.nickname = registr_user_dto.nickname.to_lowercase();
    registr_user_dto.email = registr_user_dto.email.to_lowercase();

    let password = registr_user_dto.password.clone();
    let password_hashed = hash_tools::encode_hash(&password).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_HASHING_PASSWORD, &e);
        ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, &e) // 500
    })?;

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();
    let mut res_search: Option<(bool, bool)> = None;

    let opt_nickname: Option<String> = Some(nickname.clone());
    let opt_email: Option<String> = Some(email.clone());
    // Check if the nickname and email parameters are specified.
    is_nickname_email_params_not_specified(opt_nickname.as_deref(), opt_email.as_deref())?;

    if res_search.is_none() {
        let user_orm2 = user_orm.get_ref().clone();
        // Find in the "profile" table an entry by nickname or email.
        let opt_user = web::block(move || {
            let existing_user = user_orm2
                .find_user_by_nickname_or_email(opt_nickname.as_deref(), opt_email.as_deref(), false)
                .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
                .ok()?;
            existing_user
        })
        .await
        .map_err(|e| ApiError::create(506, err::MSG_BLOCKING, &e.to_string()))?; // 506

        // If such an entry exists in the "profiles" table, then exit.
        if let Some(user) = opt_user {
            res_search = Some((nickname == user.nickname, email == user.email));
        }
    }
    if res_search.is_none() {
        let opt_nickname: Option<String> = Some(nickname.clone());
        let opt_email: Option<String> = Some(email.clone());
        let user_registr_orm2 = user_registr_orm.get_ref().clone();
        // Find in the "user_registr" table an entry with an active date, by nickname or email.
        let opt_user_registr = web::block(move || {
            let existing_user_registr = user_registr_orm2
                .find_user_registr_by_nickname_or_email(opt_nickname.as_deref(), opt_email.as_deref())
                .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
                .ok()?;
            existing_user_registr
        })
        .await
        .map_err(|e| ApiError::create(506, err::MSG_BLOCKING, &e.to_string()))?; // 506

        // If such a record exists in the "registration" table, then exit.
        if let Some(user_registr) = opt_user_registr {
            res_search = Some((nickname == user_registr.nickname, email == user_registr.email));
        }
    }

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

    let create_user_registr = CreateUserRegistr {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        password: password_hashed,
        final_date: final_date_utc,
    };
    // Create a new entity (user).
    let user_registr = web::block(move || {
        #[rustfmt::skip]
        let user_registr = user_registr_orm.create_user_registr(create_user_registr)
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

    let config_smtp = config_smtp.get_ref().clone();
    let path_template = config_smtp.smtp_path_template;
    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let subject = format!("Account registration in {}", &config_app.app_name);
    let nickname = registr_user_dto.nickname.clone();
    let receiver = registr_user_dto.email.clone();
    let target = registr_token.clone();
    let registr_duration = app_registr_duration.clone() / 60; // Convert from seconds to minutes.
    #[rustfmt::skip]
    let result = mailer.send_verification_code(
        &path_template, &receiver, &domain, &subject, &nickname, &target, registr_duration);

    if result.is_err() {
        let e = result.unwrap_err();
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), err::MSG_ERROR_SENDING_EMAIL, &e);
        return Err(ApiError::create(510, err::MSG_ERROR_SENDING_EMAIL, &e)); // 510
    }

    let registr_profile_response_dto = RegistrUserResponseDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        registr_token: registr_token.clone(),
    };
    if let Some(timer) = timer {
        info!("registration() time: {}", format!("{:.2?}", timer.elapsed()));
    }
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
/// Return the new user's profile. (`ConfirmRegistrUserResponseDto`) with status 201.
///
#[utoipa::path(
    responses(
        (status = 201, description = "New user profile.", body = ConfirmRegistrUserResponseDto,
            example = json!(ConfirmRegistrUserResponseDto { id: 120, nickname: "james_miller".to_owned()
                , email: "james_miller@gmail.us".to_owned(), created_at: Utc::now() })
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
    user_orm: web::Data<UserOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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
    let create_user = CreateUser::new(&user_registr.nickname, &user_registr.email, &user_registr.password, None);

    let user = web::block(move || {
        // Create a new entity (user, profile).
        let res_profile = user_orm.create_user(create_user).map_err(|e| {
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

    let response_dto = ConfirmRegistrUserResponseDto {
        id: user.id,
        nickname: user.nickname,
        email: user.email,
        created_at: user.created_at,
    };
    if let Some(timer) = timer {
        info!("confirm_registration() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Created().json(response_dto)) // 201
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
/// Returns the number (of expired) records deleted (`RegistrationClearForExpiredResponseDto`) with status 200.
///
/// The "admin" role is required.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The number of deleted outdated user registration records.",
            body = RegistrationClearForExpiredResponseDto, 
            example = json!(RegistrationClearForExpiredResponseDto { count_inactive_registr: 4, })
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
#[get("/api/registration/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn registration_clear_for_expired(
    user_registr_orm: web::Data<UserRegistrOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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

    let clear_for_expired_response_dto = RegistrationClearForExpiredResponseDto {
        count_inactive_registr,
    };
    if let Some(timer) = timer {
        info!("registration_clear_for_expired() time: {}", format!("{:.2?}", timer.elapsed()));
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

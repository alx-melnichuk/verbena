use std::ops::Deref;

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, http::StatusCode, post, web, HttpResponse};
use log::{debug, error, log_enabled, Level::Debug};
use utoipa;
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_authent::user_orm::impls::UserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_authent::user_orm::tests::UserOrmApp;
use vrb_authent::{
    authentication::{Authenticated, RequireAuth},
    config_jwt,
    user_orm::UserOrm,
};
use vrb_common::{
    api_error::{code_to_str, ApiError},
    err,
    validators::{msg_validation, Validator},
};
use vrb_tools::{hash_tools, token_coding};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profile_orm::tests::ProfileOrmApp;
use crate::{
    profile_models::{self, LoginProfileDto, LoginProfileResponseDto, ProfileTokensDto, TokenDto},
    profile_orm::ProfileOrm,
};

pub const TOKEN_NAME: &str = "token";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/login
            .service(login)
            // POST /api/logout
            .service(logout)
            // POST /api/token
            .service(update_token);
    }
}

/// login
///
/// User authentication to enter an authorized state.
///
/// Open a session for the current user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/login \
/// -d '{"nickname": "user01", "password": "password"}' \
/// -H 'Content-Type: application/json'
/// ```
///
/// Returns the current user's profile (`ProfileDto`) and open session token (`ProfileTokensDto`) with status 200.
///
#[utoipa::path(
    request_body(content = LoginProfileDto,
        description = "Credentials to log in to your account `LoginProfileDto`",
        example = json!({"nickname": "james_miller","password": "Pswr1234="})
    ),
    responses(
        ( status = 200, description = "The current user's profile and the open session token.",
            body = LoginProfileResponseDto),
        (status = 401, description = "The nickname or password is incorrect.", body = ApiError, examples(
            ("Nickname" = (summary = "Nickname is incorrect", description = "The nickname is incorrect.",
                value = json!(ApiError::new(401, err::MSG_WRONG_NICKNAME_EMAIL)))),
            ("Password" = (summary = "Password is incorrect", description = "The password is incorrect.",
                value = json!(ApiError::new(401, err::MSG_PASSWORD_INCORRECT))))
        )),
        (status = 417, body = [ApiError], description =
            "Validation error. `curl -i -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"password\": \"pas\" }'`",
            example = json!(ApiError::validations(
                (LoginProfileDto { nickname: "us".to_string(), password: "pas".to_string() }).validate().err().unwrap()) )),
        ( status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, "user_id: 1"))),
        (status = 409, description = "Error when comparing password hashes.", body = ApiError,
            example = json!(ApiError::create(409, err::MSG_INVALID_HASH, "Parameter is empty."))),
        ( status = 422, description = "Token encoding error.", body = ApiError,
            example = json!(ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::new(506, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::new(507, "Error while querying the database."))),
    ),
)]
#[post("/api/login")]
pub async fn login(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    json_body: web::Json<LoginProfileDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let login_profile_dto: LoginProfileDto = json_body.into_inner();
    let nickname = login_profile_dto.nickname.clone();
    let email = login_profile_dto.nickname.clone();
    let password = login_profile_dto.password.clone();
    let profile_orm2 = profile_orm.get_ref().clone();

    let opt_profile_pwd = web::block(move || {
        // Find user's profile by nickname or email.
        let existing_profile = profile_orm2
            .find_profile_by_nickname_or_email(Some(&nickname), Some(&email), true)
            .map_err(|e| {
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

    let profile_pwd = opt_profile_pwd.ok_or_else(|| {
        error!("{}-{}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_WRONG_NICKNAME_EMAIL);
        ApiError::new(401, err::MSG_WRONG_NICKNAME_EMAIL) // 401(f)
    })?;

    let profile_password = profile_pwd.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &profile_password).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::CONFLICT), err::MSG_INVALID_HASH, &e);
        ApiError::create(409, err::MSG_INVALID_HASH, &e) // 409
    })?;

    if !password_matches {
        error!("{}-{}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_PASSWORD_INCORRECT);
        return Err(ApiError::new(401, err::MSG_PASSWORD_INCORRECT)); // 401(g)
    }

    let num_token = token_coding::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Packing two parameters (user_id, num_token) into access_token.
    let access_token = token_coding::encode_token(profile_pwd.user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    // Packing two parameters (user_id, num_token) into refresh_token.
    let refresh_token = token_coding::encode_token(profile_pwd.user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = user_orm.modify_session(profile_pwd.user_id, Some(num_token)).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let msg = format!("user_id: {}", profile_pwd.user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        return Err(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg)); // 406
    }

    let profile_tokens_dto = profile_models::ProfileTokensDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let login_profile_response_dto = profile_models::LoginProfileResponseDto {
        profile_dto: profile_models::ProfileDto::from(profile_pwd),
        profile_tokens_dto,
    };

    let cookie = Cookie::build(TOKEN_NAME, access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    if let Some(timer0) = opt_timer0 {
        debug!("timer0: {}, \"login\" successful", format!("{:.2?}", timer0.elapsed()));
    }

    Ok(HttpResponse::Ok().cookie(cookie).json(login_profile_response_dto)) // 200
}

/// logout
///
/// Exit from the authorized state.
///
/// Close the session for the current user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/logout
/// ```
///
/// Return the response with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Session is closed."),
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(401, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
        (status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, "user_id: 1"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::new(506, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::new(507, "Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[post("/api/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(authenticated: Authenticated, user_orm: web::Data<UserOrmApp>) -> actix_web::Result<HttpResponse, ApiError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };
    // Get user ID.
    let user = authenticated.deref().clone();

    // Clear "num_token" value.
    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = user_orm.modify_session(user.id, None).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let msg = format!("user_id: {}", user.id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        return Err(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg)); // 406
    }

    // If a cookie has expired, the browser will delete the existing cookie.
    let cookie = Cookie::build(TOKEN_NAME, "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    if let Some(timer0) = opt_timer0 {
        debug!("timer0: {}, \"logout\" successful", format!("{:.2?}", timer0.elapsed()));
    }
    Ok(HttpResponse::Ok().cookie(cookie).body(()))
}

/// update_token
///
/// Update the value of the authorization token.
///
/// When a token has expired, it can be refreshed using "refresh_token".
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/token \
/// -d '{"token": "refresh_token"}' \
/// -H 'Content-Type: application/json'
/// ```
///
/// Return the new session token (`ProfileTokensDto`) with a status of 200.
///
#[utoipa::path(
    request_body(content = TokenDto,
        description = "The value of the \"refreshToken\" field that was received during login. `TokenDto`",
        example = json!({"token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyLjQ1MiIsImlhdCI6MTY5NjYwODExOCwiZXhwIjoxNzMyODk2MTE4fQ.NMxMEzQa2IOc_yCI0drJE89lZmGSruKghGNX80Czliw"})
    ),
    responses(
        (status = 200, description = "The new session token.", body = ProfileTokensDto),
        (status = 401, description = "Authorization required.", body = ApiError, examples(
            ("Token" = (summary = "Token is invalid or expired",
                description = "The token is invalid or expired.",
                value = json!(ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken")))),
            ("Token_number" = (summary = "Token number is incorrect", 
                description = "The specified token number is incorrect.",
                value = json!(ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_NUM, "user_id: 1"))))
        )),
        (status = 406, description = "Error session not found.", body = ApiError,
            example = json!(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, "user_id: 1"))),
        (status = 422, description = "Token encoding error.", body = ApiError,
            example = json!(ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::new(506, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::new(507, "Error while querying the database."))),
    ),
)]
#[post("/api/token")]
pub async fn update_token(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    json_token_user_dto: web::Json<TokenDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };

    // Get token from json.
    let token_user_dto: TokenDto = json_token_user_dto.into_inner();
    let token = token_user_dto.token;
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Get user ID.
    let (user_id, num_token) = token_coding::decode_token(&token, jwt_secret).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        ApiError::create(401, err::MSG_INVALID_OR_EXPIRED_TOKEN, &e) // 401
    })?;

    let user_orm2 = user_orm.get_ref().clone();

    let opt_session = web::block(move || {
        // Find a session for a given user.
        let existing_session = user_orm2.get_session_by_id(user_id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

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

    let num_token = token_coding::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = token_coding::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = token_coding::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    let opt_session = web::block(move || {
        // Find a session for a given user.
        #[rustfmt::skip]
        let existing_session = user_orm.modify_session(user_id, Some(num_token))
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        // There is no session for this user.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg); // 406
        return Err(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg));
    }

    let profile_tokens_dto = profile_models::ProfileTokensDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let cookie = Cookie::build(TOKEN_NAME, access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    if let Some(timer0) = opt_timer0 {
        debug!("timer0: {}, \"update_token\" successful", format!("{:.2?}", timer0.elapsed()));
    }

    Ok(HttpResponse::Ok().cookie(cookie).json(profile_tokens_dto)) // 200
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::http;
    use vrb_common::api_error::ApiError;
    use vrb_tools::token_data::BEARER;

    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
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

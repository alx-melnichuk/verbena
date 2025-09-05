use std::{borrow::Cow, ops::Deref, time::Instant as tm};

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, get, http::StatusCode, post, web, HttpResponse};
use log::{error, info, log_enabled, Level::Info};
use serde_json::json;
use utoipa;
use vrb_common::{
    api_error::{code_to_str, ApiError},
    err,
    validators::{msg_validation, Validator},
};
use vrb_dbase::enm_user_role::UserRole;
use vrb_tools::{hash_tools, token_coding, token_data::TOKEN_NAME};

use crate::{
    authentication::{Authenticated, RequireAuth}, config_jwt, user_authent_models::{
        LoginDto, LoginResponseDto, LoginUserProfileDto, UserTokenDto, UserTokenResponseDto, UserUniquenessDto, UserUniquenessResponseDto,
    }, user_models::User, user_orm::UserOrm, user_registr_orm::UserRegistrOrm
};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::{user_orm::impls::UserOrmApp, user_registr_orm::impls::UserRegistrOrmApp};
#[cfg(all(test, feature = "mockdata"))]
use crate::{user_orm::tests::UserOrmApp, user_registr_orm::tests::UserRegistrOrmApp};

const PASSWORD1: &str = "$argon2id$v=19$m=19456,t=2,p=1$sUU7bgDw7XH4z8SzvgXjkA$izpWfsHPJeXEhD90cRxxR/no7gyRz/DiANxe5Ckt53I";
const TOKEN1: &str = "6lqN0k3-SB_OXGzOJYUr2GwYwAEmlJWFMpOwiYrT04_WQMRQs3PAlb7WHFExilHzFrbNSTsdGzmBzFMwFD2rVXgiQtoK4fON634zV9rjMswSd7FW7eHh3PmoVxUVtID1j6TWck_wJy0TdO2rcnLZIfu2jbMzk6myQCl_5u05Ii9YvtXOI8-a0fhMRveIcM8udUGatXT5HRnGAzjDQuhDZ-94DonA0rvn2DK3D9h-baU=";
const TOKEN2: &str = "6lqN0k3-SB_OXGzOJYUr2GwYwAEmlJWFMpOwiYrT04_WQMRQs3PAlb7WHFExilHzFrbNSTsdGzmBzFMwFD2rVXgiQtoK4fON634zV9rjMsycrLJ_eCAP5d_zV7JldChywcL8qi90BT67-GoisEs_KWGhtNs9oiue3cB346cD91M3KfKmEyQ9NxroZrj9YURVr5rKJbuB5mnNJK7yc_zzHXvkQq5qmaCtp3jv93C8aaM=";
const TOKEN3: &str = "6lqN0k3-SB_OXGzOJYUr2GwYwAEmlJWFMpOwiYrT04_WQMRQs3PAlb7WHFExilHzgDSlSV4w1nQNpFT5PnamCv-tKrU2MGSdsQIRwPvTCIgvQqsScZb5j_zt2FQSG_C7kWfYtj1NcvEfC9Ze7Psl27Mua_cE909J-v8FutvVk3l5fLT3WxQL5yh0dZ2KpZ7YXDM17UYROGhzfHO1cC7rB6qF4zArCyCSmywTZ4ssUlI=";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /api/users_uniqueness
            .service(users_uniqueness)
            // POST /api/login
            .service(login)
            // POST /api/logout
            .service(logout)
            // POST /api/token
            .service(update_token);
    }
}

// ** Section: users_uniqueness **

/// users_uniqueness
///
/// Checking the uniqueness of the user's "nickname" or "email".
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users_uniqueness?nickname=demo1
/// ```
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users_uniqueness?email=demo1@gmail.us
/// ```
///
/// Returns the result of the user data uniqueness check (`UserUniquenessResponseDto`) with status 200.
/// If the value is already in use, then `{"uniqueness":false}`.
/// If the value is not yet used, then `{"uniqueness":true}`.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The result of checking whether nickname (email) is already in use.", 
            body = UserUniquenessResponseDto,
            examples(
            ("already_use" = (summary = "already in use",  description = "If the nickname (email) is already in use.",
                value = json!(UserUniquenessResponseDto::new(false)))),
            ("not_use" = (summary = "not yet in use", description = "If the nickname (email) is not yet used.",
                value = json!(UserUniquenessResponseDto::new(true))))
        )),
        (status = 406, description = "None of the parameters are specified.", body = ApiError,
            example = json!(ApiError::new(406, err::MSG_PARAMS_NOT_SPECIFIED)
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "nickname": "null", "email": "null" })))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
)]
#[get("/api/users_uniqueness")]
pub async fn users_uniqueness(
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    query_params: web::Query<UserUniquenessDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Get search parameters.
    let uniqueness_user_dto: UserUniquenessDto = query_params.clone().into_inner();

    let nickname = uniqueness_user_dto.nickname.unwrap_or("".to_owned());
    let email = uniqueness_user_dto.email.unwrap_or("".to_owned());
    // Check if the nickname and email parameters are specified.
    if nickname.len() == 0 && email.len() == 0 {
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        return Err(ApiError::new(406, err::MSG_PARAMS_NOT_SPECIFIED) // 406
            .add_param(Cow::Borrowed("invalidParams"), &json));
    }

    let user_orm2 = user_orm.get_ref().clone();
    let user_registr_orm2 = user_registr_orm.get_ref().clone();

    let opt_search = web::block(move || {
        let mut res_search: Option<(bool, bool)> = None;

        if res_search.is_none() {
            // Search for "nickname" or "email" in the "users" table.
            let opt_user = user_orm2
                .find_user_by_nickname_or_email(Some(&nickname), Some(&email), false)
                .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
                .ok()?;
            // If such an entry exists in the "users" table, then exit.
            if let Some(user) = opt_user {
                res_search = Some((nickname == user.nickname, email == user.email));
            }
        }
        if res_search.is_none() {
            let opt_user_registr = user_registr_orm2
                .find_user_registr_by_nickname_or_email(Some(&nickname), Some(&email))
                .map_err(|e| ApiError::create(507, err::MSG_DATABASE, &e)) // 507
                .ok()?;
            // If such an entry exists in the "user_registrs" table, then exit.
            if let Some(user_registr) = opt_user_registr {
                res_search = Some((nickname == user_registr.nickname, email == user_registr.email));
            }
        }
        res_search
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let uniqueness = opt_search.is_none();

    let response_dto = UserUniquenessResponseDto::new(uniqueness);

    if let Some(timer) = timer {
        info!("users_uniqueness() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Ok().json(response_dto)) // 200
}

fn get_login_user_profile() -> LoginUserProfileDto {
    let user = User::new(1100, "james_miller", "james_miller@email.us", "", UserRole::User);
    let mut result = LoginUserProfileDto::from(user.clone());
    result.avatar = None;
    result.descript = Some("descript".to_owned());
    result.theme = Some("light".to_owned());
    result.locale = Some("default".to_owned());
    result.updated_at = user.updated_at;
    result
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
/// Returns (`LoginResponseDto`) the current user data (`LoginUserProfileDto`) and the open session token (`UserTokenResponseDto`)
/// with status 200.
///
#[utoipa::path(
    request_body(content = LoginDto,
        description = "Credentials to log in to your account `LoginDto`",
        example = json!(LoginDto { nickname: "james_miller".to_owned(), password: PASSWORD1.to_owned() })
    ),
    responses(
        ( status = 200, description = "The current user's profile and the open session token.",
            body = LoginResponseDto,
            example = json!(LoginResponseDto {
                user_profile_dto: get_login_user_profile(),
                token_user_response_dto: UserTokenResponseDto { access_token: TOKEN2.to_owned(), refresh_token: TOKEN3.to_owned() },
            })
        ),
        (status = 401, description = "The nickname or password is incorrect.", body = ApiError, examples(
            ("Nickname" = (summary = "Nickname is incorrect", description = "The nickname is incorrect.",
                value = json!(ApiError::new(401, err::MSG_WRONG_NICKNAME_EMAIL)))),
            ("Password" = (summary = "Password is incorrect", description = "The password is incorrect.",
                value = json!(ApiError::new(401, err::MSG_PASSWORD_INCORRECT))))
        )),
        (status = 417, body = [ApiError], description =
            "Validation error. `curl -i -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"password\": \"pas\" }'`",
            example = json!(ApiError::validations(
                (LoginDto { nickname: "us".to_string(), password: "pas".to_string() }).validate().err().unwrap()) )),
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
    json_body: web::Json<LoginDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors)); // 417
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors)));
    }

    let login_dto: LoginDto = json_body.into_inner();
    let nickname = login_dto.nickname.clone();
    let email = login_dto.nickname.clone();
    let password = login_dto.password.clone();
    let user_orm2 = user_orm.get_ref().clone();

    let opt_user_pwd = web::block(move || {
        // Find a user profile by nickname or email address. Return user properties and password hash.
        let existing_user = user_orm2
            .find_user_by_nickname_or_email(Some(&nickname), Some(&email), true)
            .map_err(|e| {
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

    let user_pwd = opt_user_pwd.ok_or_else(|| {
        error!("{}-{}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_WRONG_NICKNAME_EMAIL);
        ApiError::new(401, err::MSG_WRONG_NICKNAME_EMAIL) // 401(f)
    })?;

    let user_password = user_pwd.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &user_password).map_err(|e| {
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
    let access_token = token_coding::encode_token(user_pwd.id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    // Packing two parameters (user_id, num_token) into refresh_token.
    let refresh_token = token_coding::encode_token(user_pwd.id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::UNPROCESSABLE_ENTITY), err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        ApiError::create(422, err::MSG_JSON_WEB_TOKEN_ENCODE, &e) // 422
    })?;

    let res_session_profile = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = user_orm.modify_session(user_pwd.id, Some(num_token)).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });

        let res_profile = user_orm.get_profile_by_id(user_pwd.id).map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });

        (res_session, res_profile)
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    let opt_session = res_session_profile.0?;
    if opt_session.is_none() {
        let msg = format!("user_id: {}", user_pwd.id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        return Err(ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg)); // 406
    }

    let opt_profile = res_session_profile.1?;
    if opt_profile.is_none() {
        let msg = format!("user_id: {}", user_pwd.id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_PROFILE_NOT_FOUND, &msg);
        return Err(ApiError::create(406, err::MSG_PROFILE_NOT_FOUND, &msg)); // 406
    }
    let profile = opt_profile.unwrap();

    let mut login_user_profile_dto = LoginUserProfileDto::from(user_pwd);
    login_user_profile_dto.avatar = profile.avatar;
    login_user_profile_dto.descript = profile.descript;
    login_user_profile_dto.theme = profile.theme;
    login_user_profile_dto.locale = profile.locale;
    login_user_profile_dto.updated_at = profile.updated_at;

    let token_user_response_dto = UserTokenResponseDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let login_response_dto = LoginResponseDto {
        user_profile_dto: login_user_profile_dto,
        token_user_response_dto,
    };

    let cookie = Cookie::build(TOKEN_NAME, access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    if let Some(timer) = timer {
        info!("login() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Ok().cookie(cookie).json(login_response_dto)) // 200
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
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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

    if let Some(timer) = timer {
        info!("logout() time: {}", format!("{:.2?}", timer.elapsed()));
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
/// Return the new session token (`UserTokenResponseDto`) with a status of 200.
///
#[utoipa::path(
    request_body(content = UserTokenDto,
        description = "The value of the \"refreshToken\" field that was received during login. `TokenUserDto`",
        example = json!(UserTokenDto { token: TOKEN1.to_owned() })
    ),
    responses(
        (status = 200, description = "The new session token.", body = UserTokenResponseDto,
            example = json!(UserTokenResponseDto { access_token: TOKEN2.to_owned(), refresh_token: TOKEN3.to_owned() })
        ),
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
    json_token_user_dto: web::Json<UserTokenDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Get token from json.
    let token_user_dto: UserTokenDto = json_token_user_dto.into_inner();
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

    let token_user_response_dto = UserTokenResponseDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let cookie = Cookie::build(TOKEN_NAME, access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    if let Some(timer) = timer {
        info!("update_token() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    Ok(HttpResponse::Ok().cookie(cookie).json(token_user_response_dto)) // 200
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

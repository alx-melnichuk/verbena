use std::ops::Deref;

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse};
use log::{debug, error, log_enabled, Level::Debug};
use utoipa;
use vrb_tools::hash_tools;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    profile_err as p_err,
    profile_models::{self, LoginProfileDto, LoginProfileResponseDto, ProfileTokensDto, TokenDto},
    profile_orm::ProfileOrm,
};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_orm::SessionOrm, tokens};
use crate::settings::err;
use crate::validators::{msg_validation, Validator};

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
    responses(
        ( status = 200, description = "The current user's profile and the open session token.",
            body = LoginProfileResponseDto),
        (status = 401, description = "The nickname or password is incorrect.", body = AppError, examples(
            ("Nickname" = (summary = "Nickname is incorrect",
                description = "The nickname is incorrect.",
                value = json!(AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL)))),
            ("Password" = (summary = "Password is incorrect", 
                description = "The password is incorrect.",
                value = json!(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT))))
        )),
        (status = 417, body = [AppError], description =
            "Validation error. `curl -i -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"password\": \"pas\" }'`",
            example = json!(AppError::validations(
                (LoginProfileDto { nickname: "us".to_string(), password: "pas".to_string() }).validate().err().unwrap()) )),
        ( status = 406, description = "Error session not found.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, 1)))),
        (status = 409, description = "Error when comparing password hashes.", body = AppError,
            example = json!(AppError::conflict409(&format!("{}; {}", err::MSG_INVALID_HASH, "Parameter is empty.")))),
        ( status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
)]
#[post("/api/login")]
pub async fn login(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    profile_orm: web::Data<ProfileOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<LoginProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors)); // 417
        return Ok(AppError::to_response(&AppError::validations(validation_errors)));
    }

    let login_profile_dto: LoginProfileDto = json_body.into_inner();
    let nickname = login_profile_dto.nickname.clone();
    let email = login_profile_dto.nickname.clone();
    let password = login_profile_dto.password.clone();

    let opt_profile_pwd = web::block(move || {
        // Find user's profile by nickname or email.
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(Some(&nickname), Some(&email), true)
            .map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        existing_profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let profile_pwd = opt_profile_pwd.ok_or_else(|| {
        error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
    })?;

    let profile_password = profile_pwd.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &profile_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_HASH, &e);
        error!("{}: {}", err::CD_CONFLICT, &message);
        AppError::conflict409(&message) // 409
    })?;

    if !password_matches {
        error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_PASSWORD_INCORRECT);
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Packing two parameters (user_id, num_token) into access_token.
    let access_token = tokens::encode_token(profile_pwd.user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Packing two parameters (user_id, num_token) into refresh_token.
    let refresh_token = tokens::encode_token(profile_pwd.user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message)
    })?;

    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = session_orm.modify_session(profile_pwd.user_id, Some(num_token)).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let message = format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile_pwd.user_id);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        return Err(AppError::not_acceptable406(&message)); // 406
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
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 406, description = "Error session not found.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, 1)))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[post("/api/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(authenticated: Authenticated, session_orm: web::Data<SessionOrmApp>) -> actix_web::Result<HttpResponse, AppError> {
    // Get user ID.
    let profile_user = authenticated.deref().clone();

    // Clear "num_token" value.
    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = session_orm.modify_session(profile_user.user_id, None).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let message = format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile_user.user_id);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        return Err(AppError::not_acceptable406(&message)); // 406
    }

    // If a cookie has expired, the browser will delete the existing cookie.
    let cookie = Cookie::build(TOKEN_NAME, "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
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
    responses(
        (status = 200, description = "The new session token.", body = ProfileTokensDto),
        (status = 401, description = "Authorization required.", body = AppError, examples(
            ("Token" = (summary = "Token is invalid or expired",
                description = "The token is invalid or expired.",
                value = json!(AppError::unauthorized401(&format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken"))))),
            ("Token_number" = (summary = "Token number is incorrect", 
                description = "The specified token number is incorrect.",
                value = json!(AppError::unauthorized401(&format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, 1)))))
        )),
        (status = 406, description = "Error session not found.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, 1)))),
        (status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
)]
#[post("/api/token")]
pub async fn update_token(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    session_orm: web::Data<SessionOrmApp>,
    json_token_user_dto: web::Json<TokenDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    #[rustfmt::skip]
    let opt_timer0 = if log_enabled!(Debug) { Some(std::time::Instant::now()) } else { None };

    // Get token from json.
    let token_user_dto: TokenDto = json_token_user_dto.into_inner();
    let token = token_user_dto.token;
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Get user ID.
    let (user_id, num_token) = tokens::decode_token(&token, jwt_secret).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401
    })?;

    let session_orm1 = session_orm.clone();

    let opt_session = web::block(move || {
        // Find a session for a given user.
        let existing_session = session_orm1.get_session_by_id(user_id).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

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
        error!("{}: {}", err::CD_UNAUTHORIZED, &message); // 401
        return Err(AppError::unauthorized401(&message));
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        let message = format!("{}; {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    let opt_session = web::block(move || {
        // Find a session for a given user.
        let existing_session = session_orm.modify_session(user_id, Some(num_token)).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        // There is no session for this user.
        let message = format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, user_id);
        error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message); // 406
        return Err(AppError::not_acceptable406(&message));
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
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use profile_models::{Profile, ProfileDto, ProfileTest};
    use serde_json::json;
    use vrb_tools::{hash_tools, token::BEARER};
    
    use crate::sessions::{config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token};
    use crate::users::user_models::UserRole;

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_profile() -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);
        profile
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_profile_pwd(nickname: &str, email: &str, password: &str) -> Profile {
        let mut profile = ProfileOrmApp::new_profile(1, nickname, email, UserRole::User);
        profile.password = hash_tools::encode_hash(password).unwrap(); // hashed
        profile
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }

    fn get_cfg_data() -> (config_jwt::ConfigJwt, (Vec<Profile>, Vec<Session>), String) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile());
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(profile1.user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(profile1.user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let data_c = (vec![profile1], vec![session1]);

        (config_jwt, data_c, token)
    }
    fn configure_profile(
        config_jwt: config_jwt::ConfigJwt,    // configuration
        data_c: (Vec<Profile>, Vec<Session>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm));
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

    // ** login **
    #[actix_web::test]
    async fn test_login_no_data() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        let req = test::TestRequest::post().uri("/api/login").to_request();
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
    async fn test_login_empty_json_object() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        let req = test::TestRequest::post().uri("/api/login").set_json(json!({})).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "".to_string(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_nickname_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_min(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_nickname_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_max(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_nickname_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_wrong(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_min(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_max(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_wrong(), password: "passwordD1T1".to_string()
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
    async fn test_login_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: "".to_string()
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
    async fn test_login_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_min()
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
    async fn test_login_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_max()
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
    async fn test_login_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_wrong()
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
    async fn test_login_if_nickname_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = data_c.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: format!("a{}", nickname), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (A)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_email_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let email = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: format!("a{}", email), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (A)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_password_invalid_hash() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Oliver_Taylor".to_string();
        let password = "hash_password_R2B2";
        let mut profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &"1".to_string());
        profile1.password = password.to_string();
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: nickname.to_string(), password: password.to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert!(app_err.message.starts_with(err::MSG_INVALID_HASH));
    }
    #[actix_web::test]
    async fn test_login_if_password_incorrect() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: nickname.to_string(), password: format!("{}b", password)
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (B)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_login_err_jsonwebtoken_encode() {
        let (mut cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        cfg_c.jwt_secret = "".to_string();
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNPROCESSABLE_ENTITY); // 422

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNPROCESSABLE_ENTITY);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; InvalidKeyFormat", p_err::MSG_JSON_WEB_TOKEN_ENCODE));
    }
    #[actix_web::test]
    async fn test_login_if_session_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let data_c = (vec![profile1], vec![]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile1_id));
    }
    #[actix_web::test]
    async fn test_login_valid_credentials() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let profile1 = data_c.0.get(0).unwrap().clone();
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let profile_vec = ProfileOrmApp::create(&vec![profile1.clone()]).profile_vec;
        let profile1_dto = ProfileDto::from(profile_vec.get(0).unwrap().clone());

        let data_c = (vec![profile1], data_c.1);
        let jwt_access = cfg_c.jwt_access;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let token_cookie_opt = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie_opt.is_some());

        let token = token_cookie_opt.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() > 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(jwt_access, 0));
        assert_eq!(true, token.http_only().unwrap());

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let login_resp: profile_models::LoginProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = login_resp.profile_tokens_dto.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = login_resp.profile_tokens_dto.refresh_token;
        assert!(refresh_token.len() > 0);

        let profile_dto_res = login_resp.profile_dto;
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }

    // ** logout **
    #[actix_web::test]
    async fn test_logout_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/logout")
            .insert_header(header_auth(&token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let token_cookie_opt = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie_opt.is_some());
        let token = token_cookie_opt.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() == 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(0, 0));
        assert_eq!(true, token.http_only().unwrap());

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(body_str, "");
    }

    // ** update_token **
    #[actix_web::test]
    async fn test_update_token_no_data() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
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
    async fn test_update_token_empty_json_object() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_update_token_invalid_dto_token_empty() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(TokenDto { token: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidSubject"));
    }
    #[actix_web::test]
    async fn test_update_token_invalid_dto_token_invalid() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(TokenDto { token: "invalid_token".to_string() })
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
    async fn test_update_token_unacceptable_token_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let profile_id_bad = profile1_id + 1;
        let num_token = data_c.1.get(0).unwrap().num_token.unwrap();
        let token_bad = encode_token(profile_id_bad, num_token, &jwt_secret, cfg_c.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(TokenDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_SESSION_NOT_FOUND, profile_id_bad));
    }
    #[actix_web::test]
    async fn test_update_token_unacceptable_token_num() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let num_token = data_c.1.get(0).unwrap().num_token.unwrap();
        let token_bad = encode_token(profile1_id, num_token + 1, &jwt_secret, cfg_c.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(TokenDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}; user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, profile1_id));
    }
    #[actix_web::test]
    async fn test_update_token_valid_dto_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let jwt_access = cfg_c.jwt_access;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let num_token = data_c.1.get(0).unwrap().num_token.unwrap();
        let token_refresh = encode_token(profile1_id, num_token, &jwt_secret, cfg_c.jwt_refresh).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(TokenDto { token: token_refresh })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let opt_token_cookie = resp.response().cookies().find(|cookie| cookie.name() == TOKEN_NAME);
        assert!(opt_token_cookie.is_some());

        let token = opt_token_cookie.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() > 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(jwt_access, 0));
        assert_eq!(true, token.http_only().unwrap());

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_token_resp: profile_models::ProfileTokensDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let access_token: String = profile_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = profile_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);

        assert_eq!(token_value, access_token);
    }
}

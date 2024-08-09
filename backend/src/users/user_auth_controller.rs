use std::ops::Deref;

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse};
use utoipa;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_orm::SessionOrm, tokens};
use crate::settings::err;
use crate::users::{
    user_err as u_err,
    user_models::{self, TokenUserDto},
};

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/logout
            .service(logout)
            // POST /api/token
            .service(update_token);
    }
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
        (status = 406, description = "Error closing session.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, 1)))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[post("/api/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(
    authenticated: Authenticated,
    session_orm: web::Data<SessionOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get user ID.
    let profile_user = authenticated.deref().clone();

    // Clear "num_token" value.
    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = session_orm.modify_session(profile_user.user_id, None).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let message = format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, profile_user.user_id);
        log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        return Err(AppError::not_acceptable406(&message)); // 406
    }

    // If a cookie has expired, the browser will delete the existing cookie.
    let cookie = Cookie::build("token", "")
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
/// Return the new session token (`UserTokensDto`) with a status of 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The new session token.", body = UserTokensDto),
        (status = 401, description = "Authorization required.", body = AppError, examples(
            ("Token" = (summary = "Token is invalid or expired",
                description = "The token is invalid or expired.",
                value = json!(AppError::unauthorized401(&format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidToken"))))),
            ("Token_number" = (summary = "Token number is incorrect", 
                description = "The specified token number is incorrect.",
                value = json!(AppError::unauthorized401(&format!("{}: user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, 1)))))
        )),
        (status = 406, description = "Error closing session.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, 1)))),
        (status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}: {}", u_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
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
    json_token_user_dto: web::Json<TokenUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get token from json.
    let token_user_dto: TokenUserDto = json_token_user_dto.into_inner();
    let token = token_user_dto.token;
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Get user ID.
    let (user_id, num_token) = tokens::decode_token(&token, jwt_secret).map_err(|e| {
        let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401
    })?;

    let session_orm1 = session_orm.clone();

    let opt_session = web::block(move || {
        // Find a session for a given user.
        let existing_session = session_orm.get_session_by_id(user_id).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let session = opt_session.ok_or_else(|| {
        // There is no session for this user.
        let message = format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, user_id);
        log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        AppError::not_acceptable406(&message) // 406
    })?;

    // Each session contains an additional numeric value.
    let session_num_token = session.num_token.unwrap_or(0);
    // Compare an additional numeric value from the session and from the token.
    if session_num_token != num_token {
        // If they do not match, then this is an error.
        let message = format!("{}: user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user_id);
        log::error!("{}: {}", err::CD_UNAUTHORIZED, &message); // 401
        return Err(AppError::unauthorized401(&message));
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        let message = format!("{}: {}", u_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        let message = format!("{}: {}", u_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    let opt_session = web::block(move || {
        // Find a session for a given user.
        let existing_session = session_orm1.modify_session(user_id, Some(num_token)).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        existing_session
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        // There is no session for this user.
        let message = format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, user_id);
        log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message); // 406
        return Err(AppError::not_acceptable406(&message));
    }

    let user_tokens_dto = user_models::UserTokensDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let cookie = Cookie::build("token", access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();
    Ok(HttpResponse::Ok().cookie(cookie).json(user_tokens_dto))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use serde_json::json;

    use crate::profiles::{profile_models::Profile, profile_orm::tests::ProfileOrmApp};
    use crate::sessions::{config_jwt, session_models::Session, tokens::encode_token};
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };
    use crate::{extractors::authentication::BEARER, hash_tools};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user(nickname: &str, email: &str, password: &str) -> User {
        let password = hash_tools::encode_hash(password).unwrap(); // hashed
        let mut user = UserOrmApp::new_user(1, nickname, email, &password);
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_profile(user: User) -> Profile {
        Profile::new(
            user.id,
            &user.nickname,
            &user.email,
            user.role.clone(),
            None,
            None,
            None,
        )
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data() -> (config_jwt::ConfigJwt, (Vec<User>, Vec<Profile>, Vec<Session>), String) {
        let user1: User = user_with_id(create_user(
            "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1"));
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // Create profile values.
        let profile1 = create_profile(user1.clone());

        let data_c = (vec![user1], vec![profile1], vec![session1]);
        
        (config_jwt, data_c, token)
    }
    fn configure_user(
        config_jwt: config_jwt::ConfigJwt,               // configuration
        data_c: (Vec<User>, Vec<Profile>, Vec<Session>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.1));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm));
        }
    }

    // ** logout **
    #[actix_web::test]
    async fn test_logout_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(logout).configure(configure_user(cfg_c, data_c))).await;
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
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
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
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
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
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(user_models::TokenUserDto { token: "".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "InvalidSubject"));
    }
    #[actix_web::test]
    async fn test_update_token_invalid_dto_token_invalid() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(user_models::TokenUserDto { token: "invalid_token".to_string() })
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
        let user1_id = data_c.0.get(0).unwrap().id;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let user_id_bad = user1_id + 1;
        let num_token = 1234;
        let token_bad = encode_token(user_id_bad, num_token, &jwt_secret, cfg_c.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(user_models::TokenUserDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, user_id_bad));
    }
    #[actix_web::test]
    async fn test_update_token_unacceptable_token_num() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1_id = data_c.0.get(0).unwrap().id;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let num_token = 1234;
        let token_bad = encode_token(user1_id, num_token + 1, &jwt_secret, cfg_c.jwt_access).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(user_models::TokenUserDto { token: token_bad })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user1_id));
    }
    #[actix_web::test]
    async fn test_update_token_valid_dto_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1_id = data_c.0.get(0).unwrap().id;
        let jwt_access = cfg_c.jwt_access;
        let jwt_secret = cfg_c.jwt_secret.as_bytes();
        let num_token = 1234;
        let token_refresh = encode_token(user1_id, num_token, &jwt_secret, cfg_c.jwt_refresh).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(update_token).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/token")
            .insert_header(header_auth(&token))
            .set_json(user_models::TokenUserDto { token: token_refresh })
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
        let user_token_resp: user_models::UserTokensDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let access_token: String = user_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = user_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);
    }
}

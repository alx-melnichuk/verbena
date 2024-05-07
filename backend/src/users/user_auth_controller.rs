use std::{borrow, ops::Deref, time::Instant};

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse};

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_orm::SessionOrm, tokens};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::{
    config_usr,
    user_models::{self, LoginUserDto, TokenUserDto},
    user_orm::UserOrm,
};
use crate::validators::{msg_validation, Validator};

pub const MSG_WRONG_NICKNAME_EMAIL: &str = "nickname_or_email_incorrect";
pub const MSG_PASSWORD_INCORRECT: &str = "password_incorrect";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        // POST /api/login
        config
            .service(login)
            // POST /api/logout
            .service(logout)
            // POST /api/token
            .service(update_token);
    }
}

fn err_database(err: String) -> AppError {
    log::error!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_jsonwebtoken_encode(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_INTER_SRV_ERROR, err::MSG_JSON_WEB_TOKEN_ENCODE, err);
    AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_JSON_WEB_TOKEN_ENCODE)
        .set_status(500)
        .add_param(borrow::Cow::Borrowed("error"), &err)
}
fn err_jsonwebtoken_decode(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", err::CD_FORBIDDEN, err::MSG_INVALID_OR_EXPIRED_TOKEN, err);
    AppError::new(err::CD_FORBIDDEN, err::MSG_INVALID_OR_EXPIRED_TOKEN).set_status(403)
}
fn err_session(user_id: i32) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - user_id - {}", err::CD_INTER_SRV_ERROR, err::MSG_SESSION_NOT_EXIST, user_id);
    AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_SESSION_NOT_EXIST)
        .set_status(500)
        .add_param(borrow::Cow::Borrowed("user_id"), &user_id)
}

/// login
///
/// User authentication.
///
/// Open a session for the specified user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/login -d '{"nickname": "user", "password": "pass"}'
/// ```
///
/// Return found `User` with status 200 or 204 not found if `User` is not found from shared in-memory storage.
#[utoipa::path(
    responses(
        (status = 200, description = "User found from storage", body = LoginUserResponseDto),
        (status = 400, description = "Input data validation error.", body = AppError),
        (status = 401, description = "The nickname or email address is incorrect.", body = AppError),
        // (status = 401, description = "The password value is incorrect.", body = AppError),
        // (status = 500, description = "Error while accessing the database.", body = AppError),
        (status = 500, description = "The hash value is invalid.", body = AppError),
        // (status = 500, description = "There was an error encoding the JSON web token.", body = AppError),
        // (status = 500, description = "Error while updating user session.", body = AppError),
        // (status = 204, description = "User not found by id",
        //  body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1")))
        // )
    ),
    // params(
    //     ("id", description = "Unique storage id of User")
    // ),
    // security(
    //     ("bearer_auth" = [])
    // )
)]
// POST /api/login
#[post("/api/login")]
pub async fn login(
    config_usr: web::Data<config_usr::ConfigUsr>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<LoginUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        #[rustfmt::skip]
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors)));
    }

    let login_user_dto: LoginUserDto = json_body.into_inner();
    let nickname = login_user_dto.nickname.clone();
    let email = login_user_dto.nickname.clone();
    let password = login_user_dto.password.clone();

    let user = web::block(move || {
        // find user by nickname or email
        let existing_user = user_orm
            .find_user_by_nickname_or_email(Some(&nickname), Some(&email))
            .map_err(|e| err_database(e.to_string()));
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let user: user_models::User = user.ok_or_else(|| {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, MSG_WRONG_NICKNAME_EMAIL);
        AppError::new(err::CD_UNAUTHORIZED, MSG_WRONG_NICKNAME_EMAIL).set_status(401)
    })?;

    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &user_password).map_err(|e| {
        #[rustfmt::skip]
        log::error!("{}: {} {:?}", err::CD_INTER_SRV_ERROR, err::MSG_INVALID_HASH, e.to_string());
        AppError::new(err::CD_INTER_SRV_ERROR, err::MSG_INVALID_HASH).set_status(500)
    })?;

    if !password_matches {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, MSG_PASSWORD_INCORRECT);
        return Err(AppError::new(err::CD_UNAUTHORIZED, MSG_PASSWORD_INCORRECT).set_status(401));
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Packing two parameters (user.id, num_token) into access_token.
    let access_token = tokens::encode_token(user.id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;

    // Packing two parameters (user.id, num_token) into refresh_token.
    let refresh_token = tokens::encode_token(user.id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;

    let session_opt = session_orm
        .modify_session(user.id, Some(num_token))
        .map_err(|e| err_database(e.to_string()))?;
    if session_opt.is_none() {
        return Err(err_session(user.id));
    }

    let user_tokens_dto = user_models::UserTokensDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let login_user_response_dto = user_models::LoginUserResponseDto {
        user_dto: user_models::UserDto::from(user),
        user_tokens_dto,
    };

    let cookie = Cookie::build("token", access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    if config_usr.usr_show_lead_time {
        log::info!("login() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().cookie(cookie).json(login_user_response_dto))
}

// POST /api/logout
#[rustfmt::skip]
#[post("/api/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(
    config_usr: web::Data<config_usr::ConfigUsr>,
    authenticated: Authenticated,
    session_orm: web::Data<SessionOrmApp>
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get user ID.
    let user = authenticated.deref().clone();

    // Clear "num_token" value.
    let session_opt = session_orm.modify_session(user.id, None)
        .map_err(|e| err_database(e.to_string()))?;

    if session_opt.is_none() {
        return Err(err_session(user.id));
    }
    // If a cookie has expired, the browser will delete the existing cookie.
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    if config_usr.usr_show_lead_time {
        log::info!("logout() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().cookie(cookie).body(()))
}

// POST /api/token
#[rustfmt::skip]
#[post("/api/token")]
pub async fn update_token(
    config_usr: web::Data<config_usr::ConfigUsr>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    session_orm: web::Data<SessionOrmApp>,
    json_token_user_dto: web::Json<TokenUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get token2 from json.
    let token_user_dto: TokenUserDto = json_token_user_dto.into_inner();
    let token = token_user_dto.token;
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Get user ID.
    let (user_id, num_token) = tokens::decode_token(&token, jwt_secret)
        .map_err(|err| err_jsonwebtoken_decode(err))?;

    let session_orm1 = session_orm.clone();

    let session_opt = web::block(move || {
        let existing_session = session_orm.find_session_by_id(user_id).map_err(|e| {
            log::error!("{}: {}", err::CD_DATABASE, e.to_string());
            AppError::new(err::CD_DATABASE, &e.to_string()).set_status(500)
        });
        existing_session
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let session = session_opt.ok_or_else(|| err_session(user_id))?;

    let session_num_token = session.num_token.unwrap_or(0);
    if session_num_token != num_token {
        log::error!("{}: {}", err::CD_FORBIDDEN, err::MSG_UNACCEPTABLE_TOKEN_NUM);
        #[rustfmt::skip]
        return Err(AppError::new(err::CD_FORBIDDEN, err::MSG_UNACCEPTABLE_TOKEN_NUM).set_status(403));
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;


    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = tokens::encode_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| err_jsonwebtoken_encode(err.to_string()))?;

    let session_opt = session_orm1
        .modify_session(user_id, Some(num_token))
        .map_err(|e| err_database(e.to_string()))?;
    if session_opt.is_none() {
        return Err(err_session(user_id));
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
    if config_usr.usr_show_lead_time {
        log::info!("new_token() lead time: {:.2?}", now.elapsed());
    }
    Ok(HttpResponse::Ok().cookie(cookie).json(user_tokens_dto))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use serde_json::json;

    use crate::extractors::authentication::BEARER;
    use crate::sessions::{config_jwt, session_models::Session, tokens::encode_token};
    use crate::users::{
        config_usr,
        user_models::{User, UserModelsTest, UserRole},
        user_orm::tests::UserOrmApp,
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }
    fn cfg_usr() -> config_usr::ConfigUsr {
        config_usr::get_test_config()
    }
    fn cfg_jwt() -> config_jwt::ConfigJwt {
        config_jwt::get_test_config()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    async fn call_service1(
        cfg_c: (config_usr::ConfigUsr, config_jwt::ConfigJwt),
        data_c: (Vec<User>, Vec<Session>),
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_usr = web::Data::new(cfg_c.0);
        let data_config_jwt = web::Data::new(cfg_c.1);

        let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_usr))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(factory),
        )
        .await;

        test::call_service(&app, request.to_request()).await
    }

    // ** login **
    #[test]
    async fn test_login_no_data() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login");

        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_login_empty_json_object() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(json!({}));

        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_login_invalid_dto_nickname_empty() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "".to_string(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REQUIRED);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_min() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::nickname_min(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_max() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::nickname_max(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_wrong() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::nickname_wrong(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REGEX);
    }
    #[test]
    async fn test_login_invalid_dto_email_min() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::email_min(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_email_max() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::email_max(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_email_wrong() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: UserModelsTest::email_wrong(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_EMAIL_TYPE);
    }
    #[test]
    async fn test_login_invalid_dto_password_empty() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith".to_string(),
            password: "".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REQUIRED);
    }
    #[test]
    async fn test_login_invalid_dto_password_min() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith".to_string(),
            password: UserModelsTest::password_min(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_password_max() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith".to_string(),
            password: UserModelsTest::password_max(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_password_wrong() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith".to_string(),
            password: UserModelsTest::password_wrong(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REGEX);
    }
    #[test]
    async fn test_login_if_nickname_not_exist() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith".to_string(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME_EMAIL);
    }
    #[test]
    async fn test_login_if_email_not_exist() {
        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: "James_Smith@gmail.com".to_string(),
            password: "passwordD1T1".to_string(),
        });
        let data_c = (vec![], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME_EMAIL);
    }
    #[test]
    async fn test_login_if_password_invalid_hash() {
        let user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: user1_nickname.to_string(),
            password: user1_password.to_string(),
        });
        let data_c = (vec![user1], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_INVALID_HASH);
    }
    #[test]
    async fn test_login_if_password_incorrect() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: user1_nickname.to_string(),
            password: format!("{}a", user1_password),
        });
        let data_c = (vec![user1], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_PASSWORD_INCORRECT);
    }
    #[test]
    async fn test_login_err_jsonwebtoken_encode() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: user1_nickname.to_string(),
            password: user1_password,
        });
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let data_c = (vec![user1], vec![]);
        let resp = call_service1((cfg_usr(), config_jwt), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_JSON_WEB_TOKEN_ENCODE);
    }
    #[test]
    async fn test_login_if_session_not_exist() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: user1_nickname.to_string(),
            password: user1_password,
        });
        let data_c = (vec![user1], vec![]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, login, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_SESSION_NOT_EXIST);
    }
    #[test]
    async fn test_login_valid_credentials() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();
        let user1_dto = user_models::UserDto::from(user1.clone());

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        // POST /api/login
        let request = test::TestRequest::post().uri("/api/login").set_json(LoginUserDto {
            nickname: user1_nickname.to_string(),
            password: user1_password,
        });
        let config_jwt = config_jwt::get_test_config();
        let jwt_access = config_jwt.jwt_access;
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), config_jwt), data_c, login, request).await;
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

        let body = test::read_body(resp).await;
        let login_resp: user_models::LoginUserResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = login_resp.user_tokens_dto.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = login_resp.user_tokens_dto.refresh_token;
        assert!(refresh_token.len() > 0);

        let user_dto_res = login_resp.user_dto;
        let json_user1_dto = serde_json::json!(user1_dto).to_string();
        let user1_dto_ser: user_models::UserDto =
            serde_json::from_slice(json_user1_dto.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(user_dto_res, user1_dto_ser);
    }

    // ** logout **
    #[test]
    async fn test_logout_valid_token() {
        let user1: User = user_with_id(create_user());

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // POST /api/logout
        let request = test::TestRequest::post().uri("/api/logout");
        let request = request.insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), config_jwt), data_c, logout, request).await;
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
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(body_str, "");
    }

    // ** new_token **
    #[test]
    async fn test_update_token_no_data() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token");
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_update_token_empty_json_object() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token").set_json(json!({}));

        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_update_token_invalid_dto_token_empty() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        // POST /api/token
        let request = test::TestRequest::post()
            .uri("/api/token")
            .set_json(user_models::TokenUserDto { token: "".to_string() });

        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_update_token_invalid_dto_token_invalid() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token").set_json(user_models::TokenUserDto {
            token: "invalid_token".to_string(),
        });
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_INVALID_OR_EXPIRED_TOKEN);
    }
    #[test]
    async fn test_update_token_unacceptable_token_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let user_id_bad = user1.id + 1;
        let token_bad = encode_token(user_id_bad, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token").set_json(user_models::TokenUserDto {
            token: token_bad.to_string(),
        });
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), config_jwt), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        dbg!(&body);
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, err::MSG_SESSION_NOT_EXIST);
        assert_eq!(
            app_err.params.get("user_id"),
            serde_json::json!({ "user_id": user_id_bad }).get("user_id")
        );
    }
    #[test]
    async fn test_update_token_unacceptable_token_num() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token_bad = encode_token(user1.id, num_token + 1, &jwt_secret, config_jwt.jwt_access).unwrap();
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token").set_json(user_models::TokenUserDto {
            token: token_bad.to_string(),
        });
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), config_jwt), data_c, update_token, request).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_UNACCEPTABLE_TOKEN_NUM);
    }
    #[test]
    async fn test_update_token_valid_dto_token() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_access = config_jwt.jwt_access;
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token_bad = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // POST /api/token
        let request = test::TestRequest::post().uri("/api/token").set_json(user_models::TokenUserDto {
            token: token_bad.to_string(),
        });
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1((cfg_usr(), cfg_jwt()), data_c, update_token, request).await;
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

        let body = test::read_body(resp).await;
        let user_token_resp: user_models::UserTokensDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = user_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = user_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);
    }
}

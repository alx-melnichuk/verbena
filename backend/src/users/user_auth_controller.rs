use std::{borrow::Cow, ops::Deref};

use actix_web::{
    cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse,
};

use crate::errors::{AppError, CD_VALIDATION};
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{
    config_jwt,
    session_orm::SessionOrm,
    tokens::{decode_dual_token, encode_dual_token, generate_num_token},
};
use crate::settings::err::{
    self, CD_BLOCKING, CD_DATABASE, CD_INTER_SRV_ERROR, CD_JSON_WEB_TOKEN, CD_UNAUTHORIZED,
    MSG_INVALID_HASH, MSG_JSON_WEB_ENCODE_TOKEN, MSG_SESSION_NOT_EXIST,
};
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::{user_models, user_orm::UserOrm};
use crate::validators::{msg_validation, Validator};

pub const MSG_WRONG_NICKNAME_EMAIL: &str = "nickname_or_email_incorrect";
pub const MSG_PASSWORD_INCORRECT: &str = "password_incorrect";

pub fn configure(cfg: &mut web::ServiceConfig) {
    // POST api/login
    cfg.service(login)
        // POST api/logout
        .service(logout)
        // POST api/token
        .service(new_token);
}

fn err_database(err: String) -> AppError {
    log::error!("{}: {}", CD_DATABASE, err);
    AppError::new(CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", CD_BLOCKING, err);
    AppError::new(CD_BLOCKING, &err).set_status(500)
}
fn err_json_web_encode_token(err: String) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - {}", CD_INTER_SRV_ERROR, MSG_JSON_WEB_ENCODE_TOKEN, err);
    AppError::new(CD_INTER_SRV_ERROR, MSG_JSON_WEB_ENCODE_TOKEN)
        .set_status(500)
        .add_param(Cow::Borrowed("error"), &err)
}
fn err_session(user_id: i32) -> AppError {
    #[rustfmt::skip]
    log::error!("{}: {} - user_id - {}", CD_INTER_SRV_ERROR, MSG_SESSION_NOT_EXIST, user_id);
    AppError::new(CD_INTER_SRV_ERROR, MSG_SESSION_NOT_EXIST)
        .set_status(500)
        .add_param(Cow::Borrowed("user_id"), &user_id)
}
// POST api/login
#[post("/login")]
pub async fn login(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<user_models::LoginUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let login_user_dto: user_models::LoginUserDto = json_body.0.clone();
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
        log::error!("{}: {}", CD_UNAUTHORIZED, MSG_WRONG_NICKNAME_EMAIL);
        AppError::new(CD_UNAUTHORIZED, MSG_WRONG_NICKNAME_EMAIL).set_status(401)
    })?;

    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &user_password).map_err(|e| {
        #[rustfmt::skip]
        log::error!("{}: {} {:?}", CD_INTER_SRV_ERROR, MSG_INVALID_HASH, e.to_string());
        AppError::new(CD_INTER_SRV_ERROR, MSG_INVALID_HASH).set_status(500)
    })?;

    if !password_matches {
        log::error!("{}: {}", CD_UNAUTHORIZED, MSG_PASSWORD_INCORRECT);
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_PASSWORD_INCORRECT).set_status(401));
    }

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = encode_dual_token(user.id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| err_json_web_encode_token(err))?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = encode_dual_token(user.id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| err_json_web_encode_token(err))?;

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

    let max_age = ActixWebDuration::new(config_jwt.jwt_access, 0);
    #[rustfmt::skip]
    let cookie = Cookie::build("token", access_token.to_owned()).path("/").max_age(max_age).http_only(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).json(login_user_response_dto))
}

// POST api/logout
#[rustfmt::skip]
#[post("/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(session_orm: web::Data<SessionOrmApp>, authenticated: Authenticated) -> actix_web::Result<HttpResponse, AppError> {
    // Get user ID.
    let user = authenticated.deref().clone();

    // Clear "num_token" value.
    let session_opt = session_orm.modify_session(user.id, None)
        .map_err(|e| err_database(e.to_string()))?;

    if session_opt.is_none() {
        return Err(err_session(user.id));
    }

    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(0, 0))
        .http_only(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).body(()))
}

// POST api/token
#[post("/token")]
pub async fn new_token(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    session_orm: web::Data<SessionOrmApp>,
    json_token_user_dto: web::Json<user_models::TokenUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get token2 from json.
    let token_user_dto: user_models::TokenUserDto = json_token_user_dto.0.clone();
    let token = token_user_dto.token;
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    let (user_id, num_token) = decode_dual_token(&token, jwt_secret)?;

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

    let session = session_opt.ok_or_else(|| {
        #[rustfmt::skip]
        log::error!("{}: {} ({} not found)", err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN, user_id.clone());
        AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN).set_status(403)
    })?;

    let session_num_token = session.num_token.unwrap_or(0);
    if session_num_token != num_token {
        #[rustfmt::skip]
        log::error!("{}: {}", err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN);
        return Err(
            AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN).set_status(403),
        );
    }

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = encode_dual_token(user_id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| {
            log::error!("{}: {}", CD_JSON_WEB_TOKEN, err);
            AppError::new(CD_JSON_WEB_TOKEN, &err).set_status(500)
        })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = encode_dual_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| {
        log::error!("{}: {}", CD_JSON_WEB_TOKEN, err);
        AppError::new(CD_JSON_WEB_TOKEN, &err).set_status(500)
    })?;

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

    let max_age = ActixWebDuration::new(config_jwt.jwt_access, 0);
    #[rustfmt::skip]
    let cookie = Cookie::build("token", access_token.to_owned()).path("/").max_age(max_age).http_only(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).json(user_tokens_dto))
}
/*fn create_tokens(
     sub: &str,
     config_jwt: config_jwt::ConfigJwt,
) -> Result<(String, String), AppError> {
let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

let access_token =
    tokens::encode_token(sub, &jwt_secret, config_jwt.jwt_access).map_err(|e| {
        log::error!("{}: {}", CD_JSON_WEB_TOKEN, e.to_string());
        AppError::new(CD_JSON_WEB_TOKEN, &e.to_string()).set_status(500)
    })?;

let refresh_token =
    tokens::encode_token(&sub, &jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        log::error!("{}: {}", CD_JSON_WEB_TOKEN, e.to_string());
        AppError::new(CD_JSON_WEB_TOKEN, &e.to_string()).set_status(500)
    })?;


let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

let access_token =
    tools_token::collect_token(user_id: i32, num_token: i32, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        log::error!("{}: {}", CD_JSON_WEB_TOKEN, e.to_string());
        AppError::new(CD_JSON_WEB_TOKEN, &e.to_string()).set_status(500)
    })?;

let refresh_token =
    tools_token::collect_token(user_id: i32, num_token: i32, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        log::error!("{}: {}", CD_JSON_WEB_TOKEN, e.to_string());
        AppError::new(CD_JSON_WEB_TOKEN, &e.to_string()).set_status(500)
    })?;

 Ok((access_token, refresh_token))
}*/

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use serde_json::json;
    // use serde_json::{json, to_string};

    // use crate::errors::AppError;
    // use crate::sessions::{config_jwt, tokens::encode_token};
    use crate::sessions::{
        config_jwt::{self, ConfigJwt},
        session_models::Session,
    };
    use crate::users::{
        user_models::{LoginUserDto, User, UserModelsTest, UserRole},
        user_orm::tests::UserOrmApp,
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    // const MSG_FAILED_DESER2: &str = "Failed to deserialize JSON string";

    fn create_user() -> User {
        let mut user =
            UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }

    async fn call_service1(
        config_jwt: ConfigJwt,
        vec: (Vec<User>, Vec<Session>),
        token: &str,
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(config_jwt);

        let data_user_orm = web::Data::new(UserOrmApp::create(vec.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(vec.1));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(factory),
        )
        .await;
        let test_request = if token.len() > 0 {
            request.insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
        } else {
            request
        };
        let req = test_request.to_request();

        test::call_service(&app, req).await
    }

    // ** login **
    #[test]
    async fn test_login_no_data() {
        let token = "";

        let request = test::TestRequest::post().uri("/login"); // POST /login

        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_login_empty_json_object() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(json!({}));

        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_login_invalid_dto_nickname_empty() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REQUIRED);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::nickname_min(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::nickname_max(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_nickname_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::nickname_wrong(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_NICKNAME_REGEX);
    }
    #[test]
    async fn test_login_invalid_dto_email_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::email_min(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_email_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::email_max(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_email_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: UserModelsTest::email_wrong(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_EMAIL_EMAIL_TYPE);
    }
    #[test]
    async fn test_login_invalid_dto_password_empty() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith".to_string(),
                password: "".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REQUIRED);
    }
    #[test]
    async fn test_login_invalid_dto_password_min() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith".to_string(),
                password: UserModelsTest::password_min(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MIN_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_password_max() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith".to_string(),
                password: UserModelsTest::password_max(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_MAX_LENGTH);
    }
    #[test]
    async fn test_login_invalid_dto_password_wrong() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith".to_string(),
                password: UserModelsTest::password_wrong(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, CD_VALIDATION);
        assert_eq!(app_err.message, user_models::MSG_PASSWORD_REGEX);
    }
    #[test]
    async fn test_login_if_nickname_not_exist() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME_EMAIL);
    }
    #[test]
    async fn test_login_if_email_not_exist() {
        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: "James_Smith@gmail.com".to_string(),
                password: "passwordD1T1".to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME_EMAIL);
    }
    #[test]
    async fn test_login_if_password_invalid_hash() {
        let user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: user1_nickname.to_string(),
                password: user1_password.to_string(),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, MSG_INVALID_HASH);
    }
    #[test]
    async fn test_login_if_password_incorrect() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: user1_nickname.to_string(),
                password: format!("{}a", user1_password),
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_PASSWORD_INCORRECT);
    }
    #[test]
    async fn test_login_err_json_web_encode_token() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: user1_nickname.to_string(),
                password: user1_password,
            });
        let mut config_jwt = config_jwt::get_test_config();
        config_jwt.jwt_secret = "".to_string();
        let vec = (vec![user1], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, MSG_JSON_WEB_ENCODE_TOKEN);
    }
    #[test]
    async fn test_login_if_session_not_exist() {
        let mut user1: User = user_with_id(create_user());
        let user1_nickname = user1.nickname.to_string();
        let user1_password = user1.password.to_string();

        let password = user1.password.to_string();
        let password_hashed = hash_tools::encode_hash(&password).unwrap();
        user1.password = password_hashed;

        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: user1_nickname.to_string(),
                password: user1_password,
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR); // 500

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_INTER_SRV_ERROR);
        assert_eq!(app_err.message, MSG_SESSION_NOT_EXIST);
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

        let token = "";

        let request = test::TestRequest::post()
            .uri("/login") // POST /login
            .set_json(LoginUserDto {
                nickname: user1_nickname.to_string(),
                password: user1_password,
            });
        let config_jwt = config_jwt::get_test_config();
        let vec = (vec![user1], vec![session1]);
        let factory = login;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let token_cookie = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie.is_some());

        let body = test::read_body(resp).await;
        let login_resp: user_models::LoginUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let user_dto_res = login_resp.user_dto;

        let json_user1_dto = serde_json::json!(user1_dto).to_string();
        let user1_dto_ser: user_models::UserDto =
            serde_json::from_slice(json_user1_dto.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(user_dto_res, user1_dto_ser);
        let access_token: String = login_resp.user_tokens_dto.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = login_resp.user_tokens_dto.refresh_token;
        assert!(refresh_token.len() > 0);
    }

    // ** logout **
    #[test]
    async fn test_logout_valid_token() {
        let user1: User = user_with_id(create_user());

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::post().uri("/logout"); // POST /logout

        let vec = (vec![user1], vec![session1]);
        let factory = logout;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
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

    //#[test]
    /*async fn test_new_token_invalid_token() {
        let user1: user_models::User = create_user();

        let config_jwt = config_jwt::get_test_config();
        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", "invalid_token"),
            ))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto {
                token: "token".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }*/

    //#[test]
    /*async fn test_new_token_expired_token() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let expired_token = tokens::create_token(&user_id, jwt_secret, -60).unwrap();
        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", expired_token),
            ))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto {
                token: "token".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }*/

    //#[test]
    /*async fn test_new_token_data_empty_token() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();
        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto {
                token: "".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401
        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_MISSING_TOKEN);
        assert_eq!(app_err.message, err::MSG_MISSING_TOKEN);
    }

    #[test]
    async fn test_new_token_data_invalid_token() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();
        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto {
                token: "invalid_token".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403
        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }

    #[test]
    async fn test_new_token_data_token_invalid_user_id() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();
        let user_id_bad: String = format!("{}a", user1.id);

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = tokens::create_token(&user_id, jwt_secret, 60).unwrap();
        let data_token = tokens::create_token(&user_id_bad, jwt_secret, 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto { token: data_token })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403
        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }

    #[test]
    async fn test_new_token_data_token_misssing_user() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();
        let user_id_bad: String = (user1.id + 1).to_string();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = tokens::create_token(&user_id, jwt_secret, 60).unwrap();
        let data_token = tokens::create_token(&user_id_bad, jwt_secret, 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto { token: data_token })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403
        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNALLOWABLE_TOKEN);
        assert_eq!(app_err.message, err::MSG_UNALLOWABLE_TOKEN);
    }
    */
    /*#[test]
    async fn test_new_token_data_token_valid_user() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(&user_id, jwt_secret, -6).unwrap();
        let data_token = encode_token(&user_id, jwt_secret, 12).unwrap();

        let num_token: Option<i32> = None;
        let session: Session = SessionOrmApp::new_session(user1.id, num_token);

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_session_orm = web::Data::new(SessionOrmApp::create(vec![session]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_session_orm))
                .service(new_token),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto { token: data_token })
            .to_request();

        let resp = test::call_service(&app, req).await;
        // assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        eprintln!("\n###### body: {:?}\n", &body);
        let user_token_resp: user_models::UserTokensDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = user_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = user_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);
    }
    */
    /*    #[test]
       async fn test_login_valid_credentials() {
           let nickname = "Oliver_Taylor".to_string();
           let email = format!("{}@gmail.com", nickname).to_string();
           let password = "passwdT1R1".to_string();
           let mut user1 =
               UserOrmApp::new_user(USER_ID_1, &nickname.clone(), &email.clone(), &password.clone());
           user1.role = user_models::UserRole::User;
           let user1b_dto = user_models::UserDto::from(user1.clone());

           let config_jwt = config_jwt::get_test_config();
           let data_config_jwt = web::Data::new(config_jwt.clone());
           let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
           let app = test::init_service(
               App::new()
                   .app_data(web::Data::clone(&data_config_jwt))
                   .app_data(web::Data::clone(&data_user_orm))
                   .service(login),
           )
           .await;
           let req = test::TestRequest::post()
               .uri("/login") //POST /login
               .set_json(user_models::LoginUserDto {
                   nickname: nickname,
                   password: password,
               })
               .to_request();
           let resp = test::call_service(&app, req).await;
           assert_eq!(resp.status(), http::StatusCode::OK);

           let body = test::read_body(resp).await;
           let login_resp: user_models::LoginUserResponseDto =
               serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

           let user_dto_res = login_resp.user_dto;
           let json_user1b_dto = serde_json::json!(user1b_dto).to_string();
           let user1b_dto_ser: user_models::UserDto =
               serde_json::from_slice(json_user1b_dto.as_bytes()).expect(MSG_FAILED_DESER);
           assert_eq!(user_dto_res, user1b_dto_ser);

           let access_token: String = login_resp.user_tokens_dto.access_token;
           assert!(!access_token.is_empty());
           let refresh_token: String = login_resp.user_tokens_dto.refresh_token;
           assert!(refresh_token.len() > 0);
       }
    */
}

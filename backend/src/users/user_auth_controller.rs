use std::ops::Deref;

use actix_web::{
    cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse,
};
use validator::Validate;

use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_models, session_orm::SessionOrm, tokens};
use crate::users::user_models;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::user_orm::UserOrm;
use crate::utils::err;

pub const CD_USER_EXISTS: &str = "NicknameOrEmailExist";
pub const MSG_USER_EXISTS: &str = "A user with the same nickname or email already exists.";

// #- pub const CD_WRONG_NICKNAME: &str = "WrongNickname";
// #- pub const MSG_WRONG_NICKNAME: &str = "The specified nickname is incorrect!";

pub const CD_WRONG_NICKNAME_EMAIL: &str = "WrongEmailNickname";
pub const MSG_WRONG_NICKNAME_EMAIL: &str = "The specified nickname or email is incorrect!";

pub const CD_WRONG_PASSWORD: &str = "WrongPassword";
pub const MSG_WRONG_PASSWORD: &str = "The password specified is incorrect!";

pub const CD_INVALID_HASH: &str = "InvalidHash";
pub const MSG_INVALID_HASH: &str = "Invalid hash format in the database.";

pub const CD_JSONWEBTOKEN: &str = "jsonwebtoken"; // #-

pub const CD_SESSION_ERROR: &str = "SessionError";
pub const MSG_SESSION_ERROR: &str = "Session error";

pub const CD_USER_NO_SESSION: &str = "UserNoSession";
pub const MSG_USER_NO_SESSION: &str = "There is no session for user";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/registration0
    cfg.service(registration0)
        // POST api/login
        .service(login)
        // POST api/logout
        .service(logout)
        // POST api/token
        .service(new_token);
}

fn err_database(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}

// POST api/registration
#[post("/registration0")]
pub async fn registration0(
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_user_dto: web::Json<user_models::CreateUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        eprintln!("##json_user_dto.validate()"); // #-
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let create_user_dto: user_models::CreateUserDto = json_user_dto.0.clone();

    let nickname = create_user_dto.nickname.clone();
    let email = create_user_dto.email.clone();

    let result_user = web::block(move || {
        // Find for a user by nickname or email.
        let existing_user = user_orm
            .find_user_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()))?;

        if existing_user.is_some() {
            eprintln!("##existing_user.is_some()"); // #-
            return Err(AppError::new(CD_USER_EXISTS, MSG_USER_EXISTS).set_status(409));
        }

        // Create a new entity (user).
        let user = user_orm
            .create_user(&create_user_dto)
            .map_err(|e| err_database(e.to_string()))?;

        // Create a new entity (session).
        #[rustfmt::skip]
        let session = session_models::Session { user_id: user.clone().id, num_token: None };
        session_orm.create_session(&session).map_err(|e| err_database(e.to_string()))?;

        Ok(user)
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;
    eprintln!("##result_user: {:#?}", result_user); // #-
    Ok(HttpResponse::Created().body(()))
}

// PUT('registration2/:confirmationId')
// userRegistration(confirmationId)

// POST api/login
#[post("/login")]
pub async fn login(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_user_dto: web::Json<user_models::LoginUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let login_user_dto: user_models::LoginUserDto = json_user_dto.0.clone();
    let nickname = login_user_dto.nickname.clone();
    let email = login_user_dto.nickname.clone();
    let password = login_user_dto.password.clone();

    let user = web::block(move || {
        // find user by nickname or email
        let existing_user = user_orm
            .find_user_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()));
        eprintln!("$$existing_user: {:?}", existing_user); // #-
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let user: user_models::User = user.ok_or_else(|| {
        log::debug!("{}: {}", CD_WRONG_NICKNAME_EMAIL, MSG_WRONG_NICKNAME_EMAIL);
        AppError::new(CD_WRONG_NICKNAME_EMAIL, MSG_WRONG_NICKNAME_EMAIL).set_status(403)
    })?;

    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &user_password).map_err(|e| {
        log::debug!("{CD_INVALID_HASH}: {MSG_INVALID_HASH} {:?}", e.to_string());
        AppError::new(CD_INVALID_HASH, MSG_INVALID_HASH).set_status(500)
    })?;

    if !password_matches {
        log::debug!("{CD_WRONG_PASSWORD}: {MSG_WRONG_PASSWORD}");
        return Err(AppError::new(CD_WRONG_PASSWORD, MSG_WRONG_PASSWORD).set_status(403));
    }

    // if (!user.registered) { ForbiddenException('Your registration not confirmed!'); }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = tokens::pack_token(user.id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| {
            log::debug!("{CD_JSONWEBTOKEN}: {}", err);
            AppError::new(CD_JSONWEBTOKEN, &err).set_status(500)
        })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = tokens::pack_token(user.id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| {
            log::debug!("{CD_JSONWEBTOKEN}: {}", err);
            AppError::new(CD_JSONWEBTOKEN, &err).set_status(500)
        })?;

    let session_opt = session_orm.modify_session(user.id, Some(num_token)).map_err(|e| {
        #[rustfmt::skip]
        log::debug!("{}: {} {}", CD_SESSION_ERROR, MSG_SESSION_ERROR, e.to_string());
        AppError::new(CD_SESSION_ERROR, MSG_SESSION_ERROR).set_status(500)
    })?;
    if session_opt.is_none() {
        #[rustfmt::skip]
        log::debug!("{}: {}{}", CD_USER_NO_SESSION, MSG_USER_NO_SESSION, user.id.to_string());
        return Err(AppError::new(CD_USER_NO_SESSION, MSG_USER_NO_SESSION).set_status(500));
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

// fn create_tokens(
//     sub: &str,
//     config_jwt: config_jwt::ConfigJwt,
// ) -> Result<(String, String), AppError> {
/*let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

let access_token =
    tokens::encode_token(sub, &jwt_secret, config_jwt.jwt_access).map_err(|e| {
        log::debug!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;

let refresh_token =
    tokens::encode_token(&sub, &jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        log::debug!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;
    */
/*
let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

let access_token =
    tools_token::collect_token(user_id: i32, num_token: i32, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        log::debug!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;

let refresh_token =
    tools_token::collect_token(user_id: i32, num_token: i32, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        log::debug!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;
*/
// Ok((access_token, refresh_token))
// }

// POST api/logout
#[rustfmt::skip]
#[post("/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout(session_orm: web::Data<SessionOrmApp>, authenticated: Authenticated) -> actix_web::Result<HttpResponse, AppError> {
    // Get user ID.
    let user = authenticated.deref().clone();

    // Clear "num_token" value.
    let session_opt = session_orm.modify_session(user.id, None).map_err(|e| {
        #[rustfmt::skip]
        log::debug!("{}: {} {}", CD_SESSION_ERROR, MSG_SESSION_ERROR, e.to_string());
        AppError::new(CD_SESSION_ERROR, MSG_SESSION_ERROR).set_status(500)
    })?;
    if session_opt.is_none() {
        #[rustfmt::skip]
        log::debug!("{}: {}{}", CD_USER_NO_SESSION, MSG_USER_NO_SESSION, user.id.to_string());
        return Err(AppError::new(CD_USER_NO_SESSION, MSG_USER_NO_SESSION).set_status(500));
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

    let (user_id, num_token) = tokens::unpack_token(&token, jwt_secret)?;

    let session_orm1 = session_orm.clone();

    let session_opt = web::block(move || {
        let existing_session = session_orm.find_session_by_id(user_id).map_err(|e| {
            log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
            AppError::new(err::CD_DATABASE, &e.to_string()).set_status(500)
        });
        existing_session
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let session = session_opt.ok_or_else(|| {
        eprintln!("$^session with {} not found", user_id.clone()); // #-
        #[rustfmt::skip]
        log::debug!("{}: session with {} not found", err::CD_UNACCEPTABLE_TOKEN, user_id.clone());
        AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN).set_status(403)
    })?;

    let session_num_token = session.num_token.unwrap_or(0);
    if session_num_token != num_token {
        eprintln!("$^session_num_token != num_token"); // #-
        #[rustfmt::skip]
        log::debug!("{}: {}", err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN);
        return Err(
            AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN).set_status(403),
        );
    }
    eprintln!("$^user_id: {}", user_id.clone());

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user.id, num_token) into a access_token.
    let access_token = tokens::pack_token(user_id, num_token, jwt_secret, config_jwt.jwt_access)
        .map_err(|err| {
            log::debug!("{CD_JSONWEBTOKEN}: {}", err);
            AppError::new(CD_JSONWEBTOKEN, &err).set_status(500)
        })?;

    // Pack two parameters (user.id, num_token) into a access_token.
    let refresh_token = tokens::pack_token(user_id, num_token, jwt_secret, config_jwt.jwt_refresh)
        .map_err(|err| {
            log::debug!("{CD_JSONWEBTOKEN}: {}", err);
            AppError::new(CD_JSONWEBTOKEN, &err).set_status(500)
        })?;

    let session_opt = session_orm1.modify_session(user_id, Some(num_token)).map_err(|e| {
        #[rustfmt::skip]
        log::debug!("{}: {} {}", CD_SESSION_ERROR, MSG_SESSION_ERROR, e.to_string());
        AppError::new(CD_SESSION_ERROR, MSG_SESSION_ERROR).set_status(500)
    })?;
    if session_opt.is_none() {
        #[rustfmt::skip]
        log::debug!("{}: {}{}", CD_USER_NO_SESSION, MSG_USER_NO_SESSION, user_id);
        return Err(AppError::new(CD_USER_NO_SESSION, MSG_USER_NO_SESSION).set_status(500));
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

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{http, test, web, App};
    // use serde_json::{json, to_string};

    // use crate::errors::AppError;
    use crate::sessions::config_jwt;
    use crate::users::{user_models, user_orm::tests::UserOrmApp};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    // const MSG_FAILED_DESER2: &str = "Failed to deserialize JSON string";

    fn create_user() -> user_models::User {
        let mut user = UserOrmApp::new_user(
            1001,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
        );
        user.role = user_models::UserRole::User;
        user
    }

    //#[test] // !
    /*async fn test_login_invalid_dto_nickname_min() {
        let user1: user_models::User = create_user();
        let nickname: String = (0..(user_models::NICKNAME_MIN - 1)).map(|_| 'a').collect();

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
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MIN);
        assert_eq!(app_err.message, msg_err);
    }*/

    //#[test] // !
    /*async fn test_login_invalid_dto_nickname_max() {
        let user1: user_models::User = create_user();
        let nickname: String = (0..(user_models::NICKNAME_MAX + 1)).map(|_| 'a').collect();

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
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MAX);
        assert_eq!(app_err.message, msg_err);
    }*/

    //#[test] // !
    /*async fn test_login_invalid_dto_wrong_nickname() {
        let user1: user_models::User = create_user();
        let nickname: String = "Oliver_Taylor#".to_string();

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
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_REGEX);
        assert_eq!(app_err.message, msg_err);
    }*/
    /*
    #[test]
    async fn test_login_non_existent_nickname() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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
                nickname: format!("{}a", nickname).to_string(),
                password: password,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME);
    }*/

    //#[test] // !
    /*async fn test_login_invalid_dto_email_min() {
        let user1: user_models::User = create_user();
        let suffix = "@us".to_string();
        let email_min: usize = user_models::EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        let email2 = format!("{}{}", email, suffix);

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
                nickname: email2,
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_EMAIL_MIN);
        assert_eq!(app_err.message, msg_err);
    }*/

    //#[test] // !
    /*async fn test_login_invalid_dto_email_max() {
        let user1: user_models::User = create_user();
        let email_max: usize = user_models::EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        let email2 = format!("{}@{}{}", prefix, suffix, domain);

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
                nickname: email2,
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_EMAIL_MAX);
        assert_eq!(app_err.message, msg_err);
    }*/

    // #[test] // !
    /*async fn test_login_invalid_dto_wrong_email() {
        let user1: user_models::User = create_user();
        let suffix = "@".to_string();
        let email_min: usize = user_models::EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        let email2 = format!("{}{}", email, suffix);

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
                nickname: email2,
                password: "passwordD1T1".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_EMAIL);
        assert_eq!(app_err.message, msg_err);
    }*/

    //#[test]
    /*async fn test_login_non_existent_email() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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
                nickname: format!("a{}", nickname).to_string(),
                password: password,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME);
    }*/

    // #[test] // !
    /*async fn test_login_invalid_dto_password_min() {
        let user1: user_models::User = create_user();
        let password: String = (0..(user_models::PASSWORD_MIN - 1)).map(|_| 'a').collect();

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
                nickname: "James_Smith".to_string(),
                password: password,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MIN);
        assert_eq!(app_err.message, msg_err);
    }*/

    // #[test] // !
    /*async fn test_login_invalid_dto_password_max() {
        let user1: user_models::User = create_user();
        let password: String = (0..(user_models::PASSWORD_MAX + 1)).map(|_| 'a').collect();

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
                nickname: "James_Smith".to_string(),
                password: password,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MAX);
        assert_eq!(app_err.message, msg_err);
    }*/

    // #[test]
    /*async fn test_login_wrong_hashed_password() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;
        user1.password += "bad";

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
                nickname: nickname.to_string(),
                password: password.to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401
        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_INVALID_HASH_FORMAT);
    }*/

    // #[test] // no use
    /*async fn test_login_wrong_password() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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
                nickname: nickname.to_string(),
                password: format!("{}a", password),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_PASSWORD);
    }*/

    // #[test] // !
    /*async fn test_login_no_data() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }*/

    // #[test] // !
    /*async fn test_login_empty_json_object() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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
            .set_json(json!({}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }*/

    // #[test]
    /*async fn test_login_valid_credentials() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
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

    #[test]
    async fn test_login_valid_credentials_receive_cookie() {
        let nickname = "Oliver_Taylor".to_string();
        let email = format!("{}@gmail.com", nickname).to_string();
        let password = "passwdT1R1".to_string();
        let mut user1 =
            UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
        user1.role = user_models::UserRole::User;

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

        let token_cookie = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie.is_some());
    }

    #[test]
    async fn test_logout_valid_token() {
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
                .service(logout),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/logout") //POST /login
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let token_cookie = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie.is_some());

        let token = token_cookie.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() == 0);

        let max_age = token.max_age();
        assert!(max_age.is_some());

        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(0, 0));

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(body_str, "");
    }

    #[test]
    async fn test_logout_misssing_token() {
        let user1: user_models::User = create_user();
        let config_jwt = config_jwt::get_test_config();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(logout),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/logout") //POST /login
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED);

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_DESER2);
        assert_eq!(app_err.code, err::CD_MISSING_TOKEN);
        assert_eq!(app_err.message, err::MSG_MISSING_TOKEN);
    }

    #[test]
    async fn test_logout_invalid_token() {
        let user1: user_models::User = create_user();
        let config_jwt = config_jwt::get_test_config();
        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(logout),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", "invalid_token"),
            ))
            .uri("/logout") //POST /logout
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_DESER2);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }

    #[test]
    async fn test_logout_expired_token() {
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
                .service(logout),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", expired_token),
            ))
            .uri("/logout") //POST /login
            .to_request();
        let result = test::try_call_service(&app, req).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");
        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect(MSG_FAILED_DESER2);
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }
    */
    /*#[test]
    async fn test_new_token_misssing_token() {
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
            .uri("/token") //POST /token
            .set_json(user_models::TokenUserDto {
                token: "token".to_string(),
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
    async fn test_new_token_invalid_token() {
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
    }

    #[test]
    async fn test_new_token_expired_token() {
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
    }

    #[test]
    async fn test_new_token_data_empty_token() {
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
    #[test]
    async fn test_new_token_data_token_valid_user() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = tokens::encode_token(&user_id, jwt_secret, -60).unwrap();
        let data_token = tokens::encode_token(&user_id, jwt_secret, 120).unwrap();

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_token_resp: user_models::UserTokensDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = user_token_resp.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = user_token_resp.refresh_token;
        assert!(refresh_token.len() > 0);
    }

    /*    #[test]
       async fn test_login_valid_credentials() {
           let nickname = "Oliver_Taylor".to_string();
           let email = format!("{}@gmail.com", nickname).to_string();
           let password = "passwdT1R1".to_string();
           let mut user1 =
               UserOrmApp::new_user(1001, &nickname.clone(), &email.clone(), &password.clone());
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

use actix_web::{
    cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse, Responder,
};
use serde_json::json;
use validator::Validate;

use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::extractors::authentication::RequireAuth;
use crate::sessions::{config_jwt::ConfigJwt, hash_tools, tokens};
use crate::users::user_models;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::user_orm::{UserOrm, CD_BLOCKING, CD_DATA_BASE};

pub const CD_HASHING: &str = "Hashing";
pub const CD_UNAUTHORIZED: &str = "UnAuthorized";
pub const CD_USER_EXISTS: &str = "NicknameOrEmailExist";
pub const MSG_USER_EXISTS: &str = "A user with the same nickname or email already exists.";
// #- pub const MSG_NO_USER_FOR_TOKEN: &str = "There is no user for this token";
pub const MSG_WRONG_NICKNAME: &str = "Wrong email / nickname!";
pub const MSG_WRONG_PASSWORD: &str = "Wrong password!";
pub const MSG_INVALID_HASH_FORMAT: &str = "Invalid hash format!";

pub const CD_JSONWEBTOKEN: &str = "jsonwebtoken";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/signup
    cfg.service(signup)
        // POST api/login
        .service(login)
        // POST api/logout
        .service(logout);
}

// POST api/signup
#[post("/signup")]
pub async fn signup(
    user_orm: web::Data<UserOrmApp>,
    json_user_dto: web::Json<user_models::CreateUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let mut create_user_dto: user_models::CreateUserDto = json_user_dto.0.clone();
    // let mut create_user2 = create_user_dto.clone();

    let nickname = create_user_dto.nickname.clone();
    let email = create_user_dto.email.clone();
    let password = create_user_dto.password.clone();

    let password_hashed = hash_tools::hash(&password).map_err(|e| {
        log::debug!("{}: {}", CD_HASHING, e.to_string());
        AppError::new(CD_HASHING, &e.to_string())
    })?;
    create_user_dto.password = password_hashed;

    let result_user = web::block(move || {
        // Find for a user by nickname or email.
        let existing_user =
            user_orm.find_user_by_nickname_or_email(&nickname, &email).map_err(|e| {
                log::debug!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            })?;

        if existing_user.is_some() {
            return Err(AppError::new(CD_USER_EXISTS, MSG_USER_EXISTS).set_status(409));
        }

        // Create a new entity (user).
        let res_user = user_orm.create_user(&create_user_dto).map_err(|e| {
            log::debug!("{}: {}", CD_DATA_BASE, e.to_string());
            AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
        });
        res_user
    })
    .await
    .map_err(|e| {
        log::debug!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    Ok(HttpResponse::Created().json(user_models::UserDto::from(result_user)))
}

// POST api/login
#[post("/login")]
pub async fn login(
    config_jwt: web::Data<ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    json_user_dto: web::Json<user_models::LoginUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        eprintln!("$$validate() error: {:?}", errors);
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let login_user_dto: user_models::LoginUserDto = json_user_dto.0.clone();
    eprintln!("$$login_user_dto: {:?}", login_user_dto); // #-
    let nickname = login_user_dto.nickname.clone();
    let email = login_user_dto.nickname.clone();
    let password = login_user_dto.password.clone();

    let cnf_jwt = config_jwt.clone();

    let user = web::block(move || {
        // find user by nickname or email
        let existing_user =
            user_orm.find_user_by_nickname_or_email(&nickname, &email).map_err(|e| {
                log::debug!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            });

        existing_user
    })
    .await
    .map_err(|e| {
        log::debug!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    let user: user_models::User = user.ok_or_else(|| {
        log::debug!("{}: {}", CD_UNAUTHORIZED, MSG_WRONG_NICKNAME); // ++
        AppError::new(CD_UNAUTHORIZED, MSG_WRONG_NICKNAME).set_status(401)
    })?;

    eprintln!("$$user exists");
    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare(&password, &user_password).map_err(|e| {
        let msg = format!("InvalidHashFormat {:?}", e.to_string());
        log::debug!("{}: {} {}", CD_UNAUTHORIZED, MSG_INVALID_HASH_FORMAT, msg);
        AppError::new(CD_UNAUTHORIZED, MSG_INVALID_HASH_FORMAT).set_status(401) // ++
    })?;

    if !password_matches {
        eprintln!("$$user !password_matches $$");
        log::debug!("{}: {}", CD_UNAUTHORIZED, MSG_WRONG_PASSWORD); //++
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_WRONG_PASSWORD).set_status(401));
    }

    // if (!user.registered) { ForbiddenException('Your registration not confirmed!'); }

    let token = tokens::create_token(
        &user.id.to_string(),
        &cnf_jwt.jwt_secret.as_bytes(),
        cnf_jwt.jwt_maxage,
    )
    .map_err(|e| {
        log::debug!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(60 * &cnf_jwt.jwt_maxage, 0))
        .http_only(true)
        .finish();

    let refresh_token = "refresh_token".to_string();
    let user_dto: user_models::UserDto = user_models::UserDto::from(user);
    let user_tokens_dto: user_models::UserTokensDto = user_models::UserTokensDto {
        access_token: token.to_owned(),
        refresh_token,
    };

    let response = user_models::ResponseLoginUserDto {
        user_dto,
        user_tokens_dto,
    };
    // let response = json!({ "status": "success".to_string(), "token": token.to_string(), });

    Ok(HttpResponse::Ok().cookie(cookie).json(response))
}

// POST api/logout
#[rustfmt::skip]
#[post("/logout", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn logout() -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok().cookie(cookie).json(json!({"status": "success"}))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{http, test, web, App};
    // use chrono::Utc;

    use crate::errors::AppError;
    use crate::sessions::config_jwt;
    use crate::users::{user_models, user_orm::tests::UserOrmApp};

    use super::*;

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

    const MSG_FAILED_DESER: &str = "Failed to deserialize AppError response from JSON.";

    #[test]
    async fn test_login_invalid_dto_nickname_min() {
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
    }

    #[test]
    async fn test_login_invalid_dto_nickname_max() {
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
    }

    #[test]
    async fn test_login_invalid_dto_wrong_nickname() {
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
    }

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
    }

    #[test]
    async fn test_login_invalid_dto_email_min() {
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
    }

    #[test]
    async fn test_login_invalid_dto_email_max() {
        let user1: user_models::User = create_user();

        // let nickname: String = (0..(user_models::NICKNAME_MAX + 1)).map(|_| 'a').collect();
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
    }

    #[test]
    async fn test_login_invalid_dto_wrong_email() {
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
    }

    #[test]
    async fn test_login_non_existent_email() {
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
    }

    #[test]
    async fn test_login_invalid_dto_password_min() {
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
    }

    #[test]
    async fn test_login_invalid_dto_password_max() {
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
    }

    #[test]
    async fn test_login_wrong_hashed_password() {
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
    }

    #[test]
    async fn test_login_wrong_password() {
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
    }

    #[test]
    async fn test_login_no_data() {
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
    }

    #[test]
    async fn test_login_empty_json_object() {
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
    }

    // #[test]
    /*async fn test_login_wrong_email() {
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
                nickname: format!("a{}", email).to_string(),
                password: password.to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        // assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        let body = test::read_body(resp).await;
        eprintln!("\n###### body: {:?}\n", body);
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_UNAUTHORIZED);
        assert_eq!(app_err.message, MSG_WRONG_CREDENTIALS_A);
    }*/
    /*ValidationErrors(
    { "nickname": Field(
        [
            ValidationError {
                code: "regex",
                message: Some("wrong value /^[a-zA-Z0-9_]+$/"),
                params: {"value": String("aOliver_Taylor@gmail.com")}
            }
        ])
    })*/
    /*
        #[test]
        async fn test_login_wrong_password() {
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
                    password: format!("{}a", password).to_string(),
                })
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

            let body = test::read_body(resp).await;
            // eprintln!("\n###### body: {:?}\n", body);
            let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

            assert_eq!(app_err.code, CD_UNAUTHORIZED);
            assert_eq!(app_err.message, MSG_WRONG_CREDENTIALS_C);
        }

    */
}

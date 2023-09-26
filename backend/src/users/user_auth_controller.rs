use actix_web::{
    cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse, Responder,
};
use serde_json::json;

use crate::errors::AppError;
use crate::extractors::authentication::RequireAuth;
use crate::sessions::{hash_tools, tokens};
use crate::users::{user_models, user_orm::UserOrm};

use super::user_models::UserRole;

pub const CD_HASHING: &str = "Hashing";
pub const CD_BLOCKING: &str = "Blocking";
pub const CD_DATA_BASE: &str = "DataBase";
pub const CD_UNAUTHORIZED: &str = "UnAuthorized";
pub const CD_USER_EXISTS: &str = "NicknameOrEmailExist";
pub const MSG_USER_EXISTS: &str = "A user with the same nickname or email already exists.";
pub const MSG_WRONG_CREDENTIALS: &str = "Email or password is wrong";
pub const CD_JSONWEBTOKEN: &str = "jsonwebtoken";

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            //signup
            .service(register)
            // login
            .service(login),
    )
    .service(
        web::scope("/auth")
            // logout
            .service(logout)
            .wrap(RequireAuth::allowed_roles(vec![
                UserRole::User,
                UserRole::Moderator,
                UserRole::Admin,
            ])),
    );
}

// POST api/auth/signup
#[post("/signup")]
pub async fn register(
    user_orm: web::Data<UserOrmApp>,
    body: web::Json<user_models::CreateUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let register_user: user_models::CreateUserDto = body.0.clone();
    let mut register_user2 = register_user.clone();

    // let err_res1: Vec<AppError> = user_models::CreateUserDto::validation(&register_user);
    // if err_res1.len() > 0 {
    //     return Ok(AppError::get_http_response(&err_res1));
    // }
    let nickname = register_user.nickname.clone();
    let email = register_user.email.clone();

    let password_hashed = hash_tools::hash(&register_user.password).map_err(|e| {
        log::warn!("{}: {}", CD_HASHING, e.to_string());
        AppError::new(CD_HASHING, &e.to_string())
    })?;
    register_user2.password = password_hashed;

    let result_user = web::block(move || {
        // Find for a user by nickname or email.
        let existing_user =
            user_orm.find_user_by_nickname_or_email(&nickname, &email).map_err(|e| {
                log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            })?;

        if existing_user.is_some() {
            return Err(AppError::new(CD_USER_EXISTS, MSG_USER_EXISTS).set_status(409));
        }

        // Create a new entity (user).
        let res_user = user_orm.create_user(&register_user2).map_err(|e| {
            log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
            AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
        });
        res_user
    })
    .await
    .map_err(|e| {
        log::warn!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    Ok(HttpResponse::Created().json(user_models::UserDto::from(result_user)))
}

// POST api/auth/login
#[post("/login")]
pub async fn login(
    config_jwt: web::Data<ConfigJwt>,
    user_orm: web::Data<UserOrmApp>,
    body: web::Json<user_models::LoginUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    let login_user_dto: user_models::LoginUserDto = body.0.clone();

    let nickname = login_user_dto.nickname.clone();
    let email = login_user_dto.nickname.clone();
    let password = login_user_dto.password.clone();

    let cnf_jwt = config_jwt.clone();

    let user = web::block(move || {
        // find user by nickname or email
        let existing_user =
            user_orm.find_user_by_nickname_or_email(&nickname, &email).map_err(|e| {
                log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            });
        existing_user
    })
    .await
    .map_err(|e| {
        log::warn!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    if user.is_none() {
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_WRONG_CREDENTIALS).set_status(401));
    }

    let user = user.unwrap();

    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare(&password, &user_password).map_err(|e| {
        log::warn!("{}: {}", "InvalidHashFormat", e.to_string());
        AppError::new("InvalidHashFormat", &e.to_string()).set_status(401)
    })?;

    if !password_matches {
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_WRONG_CREDENTIALS).set_status(401));
    }

    let token = tokens::create_token(
        &user.id.to_string(),
        &cnf_jwt.jwt_secret.as_bytes(),
        cnf_jwt.jwt_maxage,
    )
    .map_err(|e| {
        log::warn!("{}: {}", CD_JSONWEBTOKEN, e.to_string());
        AppError::new(CD_JSONWEBTOKEN, &e.to_string()).set_status(500)
    })?;

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(60 * &cnf_jwt.jwt_maxage, 0))
        .http_only(true)
        .finish();

    let response = json!({
        "status": "success".to_string(),
        "token": token.to_string(),
    });
    Ok(HttpResponse::Ok().cookie(cookie).json(response))
}

// POST api/auth/logout
#[post("/logout")]
pub async fn logout() -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok().cookie(cookie).json(json!({"status": "success"}))
}

use actix_web::{
    cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse, Responder,
};
use serde_json::json;
use validator::Validate;

use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::extractors::authentication::RequireAuth;
use crate::sessions::{config_jwt::ConfigJwt, hash_tools, tokens};
use crate::users::user_models::{self /*UserRole*/};
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::user_orm::{UserOrm, CD_BLOCKING, CD_DATA_BASE};

pub const CD_HASHING: &str = "Hashing";
pub const CD_UNAUTHORIZED: &str = "UnAuthorized";
pub const CD_USER_EXISTS: &str = "NicknameOrEmailExist";
pub const MSG_USER_EXISTS: &str = "A user with the same nickname or email already exists.";
pub const MSG_NO_USER_FOR_TOKEN: &str = "There is no user for this token";
pub const MSG_WRONG_CREDENTIALS: &str = "Email or password is wrong";

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
        log::debug!("{}: {}", CD_HASHING, e.to_string());
        AppError::new(CD_HASHING, &e.to_string())
    })?;
    register_user2.password = password_hashed;

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
        let res_user = user_orm.create_user(&register_user2).map_err(|e| {
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
    log::debug!("login()"); // #-
                            // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let login_user_dto: user_models::LoginUserDto = json_user_dto.0.clone();
    log::debug!("login_user_dto: {:?}", login_user_dto); // #-
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

        if existing_user.clone().unwrap().is_none() {
            log::debug!("##existing_user.is_none()");
        } else {
            log::debug!("##existing_user.is_some()");
        }

        existing_user
    })
    .await
    .map_err(|e| {
        log::debug!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    if user.is_none() {
        log::debug!("{}: {}", CD_UNAUTHORIZED, MSG_NO_USER_FOR_TOKEN);
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_NO_USER_FOR_TOKEN).set_status(401));
    }

    let user = user.unwrap();

    let user_password = user.password.to_string();
    let password_matches = hash_tools::compare(&password, &user_password).map_err(|e| {
        log::debug!("{}: {}", "InvalidHashFormat", e.to_string());
        AppError::new("InvalidHashFormat", &e.to_string()).set_status(401)
    })?;

    if !password_matches {
        log::debug!("{}: {}", CD_UNAUTHORIZED, MSG_WRONG_CREDENTIALS);
        return Err(AppError::new(CD_UNAUTHORIZED, MSG_WRONG_CREDENTIALS).set_status(401));
    }

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

    let response = json!({
        "status": "success".to_string(),
        "token": token.to_string(),
    });
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

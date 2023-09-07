use std::collections::HashMap;

use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use log;

use crate::dbase::db;
use crate::users::users_consts::ERR_INCORRECT_VALUE;
use crate::users::users_models::{CreateUserDTO, LoginInfoDTO, LoginUserDTO};
use crate::users::{
    users_consts::{ERR_CASTING_TO_TYPE, ERR_NOT_FOUND_BY_ID},
    users_models::UserDTO,
    users_service,
};
use crate::utils::{errors::AppError, strings::msg};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(signup).service(login))
        .service(get_user_by_id)
        .service(get_user_by_nickname)
        .service(put_user)
        .service(patch_user)
        .service(delete_user);
}

#[get("/user/{id}")]
pub async fn get_user_by_id(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();
    log::debug!("@@ id_str={}", id_str.to_string()); // #

    let id = id_str
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let result_user = web::block(move || {
        let mut conn = pool.get()?;
        // Find user by id.
        users_service::find_user_by_id(&mut conn, id)
    })
    .await??;
    let id_ = id.to_string();

    match result_user {
        Some(user_dto) => Ok(HttpResponse::Ok().json(user_dto)),
        None => Err(AppError::NotFound(msg(ERR_NOT_FOUND_BY_ID, &[&id_]))),
    }
}

#[get("/user")]
pub async fn get_user_by_nickname(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let params = web::Query::<HashMap<String, String>>::from_query(request.query_string()).unwrap();
    let empty_line = "".to_string();
    let value = params.get("nickname").unwrap_or(&empty_line);
    let nickname = value.to_string();

    log::debug!("@@ nickname={}", nickname.to_string()); // #
    eprintln!("@@ nickname={}", nickname.to_string());
    if nickname.len() == 0 {
        return Err(AppError::BadRequest(msg(
            ERR_INCORRECT_VALUE,
            &["nickname"],
        )));
    }

    let user_list = web::block(move || {
        let mut conn = pool.get()?;
        // Find user by nickname.
        users_service::find_users_by_nickname(&mut conn, &nickname)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_list))
}

//#[post("/user")]
/*pub async fn post_user(
    pool: web::Data<db::DbPool>,
    json_user_dto: web::Json<UserDTO>,
) -> actix_web::Result<HttpResponse, AppError> {
    let mut new_user: UserDTO = json_user_dto.0.clone();
    UserDTO::clear_optional(&mut new_user);

    let err_res1: Vec<AppError> = UserDTO::validation_for_add(&new_user);
    if err_res1.len() > 0 {
        return Ok(AppError::get_http_response(&err_res1));
    }

    let user_dto = web::block(move || {
        let mut conn = pool.get()?;

        // Create a new entity (user).
        users_service::create_user(&mut conn, new_user)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_dto))
}*/

#[put("/user/{id}")]
pub async fn put_user(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
    json_user_dto: web::Json<UserDTO>,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let mut new_user: UserDTO = json_user_dto.0.clone();
    UserDTO::clear_optional(&mut new_user);

    let create_user_dto = CreateUserDTO::from(new_user.clone());
    let err_res1: Vec<AppError> = CreateUserDTO::validation(&create_user_dto);
    if err_res1.len() > 0 {
        return Ok(AppError::get_http_response(&err_res1));
    }

    let user_dto = web::block(move || {
        let mut conn = pool.get()?;
        // Modify the entity (user) with new data.
        let data = users_service::modify_user(&mut conn, id, new_user);
        data
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_dto))
}

#[patch("/user/{id}")]
pub async fn patch_user(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
    json_user_dto: web::Json<UserDTO>,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let mut new_user: UserDTO = json_user_dto.0.clone();
    UserDTO::clear_optional(&mut new_user);

    let err_res1: Vec<AppError> = UserDTO::validation_for_edit(&new_user);
    if err_res1.len() > 0 {
        return Ok(AppError::get_http_response(&err_res1));
    }

    let user_dto = web::block(move || {
        let mut conn = pool.get()?;

        // Modify the entity (user) with new data.
        users_service::modify_user(&mut conn, id, new_user)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_dto))
}

#[delete("/user/{id}")]
pub async fn delete_user(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let count = web::block(move || {
        let mut conn = pool.get()?;
        // Delete the entity (user) with the specified ID.
        users_service::delete_user(&mut conn, id)
    })
    .await??;

    if 0 == count {
        let id_ = id.to_string();
        Err(AppError::NotFound(msg(ERR_NOT_FOUND_BY_ID, &[&id_])))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

// POST api/auth/signup
#[post("/signup")]
pub async fn signup(
    pool: web::Data<db::DbPool>,
    json_create_user_dto: web::Json<CreateUserDTO>,
) -> actix_web::Result<HttpResponse, AppError> {
    let create_user_dto: CreateUserDTO = json_create_user_dto.0.clone();

    let err_res1: Vec<AppError> = CreateUserDTO::validation(&create_user_dto);
    if err_res1.len() > 0 {
        return Ok(AppError::get_http_response(&err_res1));
    }

    let user_dto = web::block(move || {
        let mut conn = pool.get()?;

        let nickname = create_user_dto.nickname.clone();
        let email = create_user_dto.email.clone();
        // find user by nickname or email
        let search_results =
            users_service::find_user_by_nickname_or_email(&mut conn, &nickname, &email)?;
        if search_results.is_some() {
            return Err(AppError::BadRequest(
                "A user with the same name or email is already registered.".to_string(),
            ));
        }

        // Create a new entity (user).
        users_service::create_user(&mut conn, &create_user_dto)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_dto))
}

// POST api/auth/login
#[post("/login")]
pub async fn login(
    pool: web::Data<db::DbPool>,
    json_login_user_dto: web::Json<LoginUserDTO>,
) -> actix_web::Result<HttpResponse, AppError> {
    let mut login_user_dto: LoginUserDTO = json_login_user_dto.0.clone();

    let nickname = login_user_dto.nickname.clone();
    let email = login_user_dto.nickname.clone();
    // let password = login_user_dto.password.clone();

    let user_dto = web::block(move || {
        let mut conn = pool.get()?;

        // find user by nickname or email
        let user_to_verify =
            users_service::find_user_by_nickname_or_email(&mut conn, &nickname, &email)?;

        if user_to_verify.is_none() {
            return Err(AppError::BadRequest(
                "The username or password is incorrect.".to_string(),
            ));
        }

        // Some(LoginInfoDTO {
        //     username: "user_to_verify.username".to_string(),
        //     login_session: String::new(),
        // });
        user_to_verify
    })
    .await??;
    /*
    match account_service::login(login_dto.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(ResponseBody::new(
            constants::MESSAGE_LOGIN_SUCCESS,
            token_res,
        ))),
        Err(err) => Err(err),
    }
    */
    Ok(HttpResponse::Ok().json("user_dto"))
}

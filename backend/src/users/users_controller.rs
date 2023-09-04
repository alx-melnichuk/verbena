use std::collections::HashMap;

use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use log;

use crate::dbase::db;
use crate::users::users_consts::ERR_INCORRECT_VALUE;
use crate::users::{
    users_consts::{ERR_CASTING_TO_TYPE, ERR_NOT_FOUND_BY_ID},
    users_models::UserDTO,
    users_service,
};
use crate::utils::{errors::AppError as AError, strings::msg};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_by_id)
        .service(get_user_by_nickname)
        .service(post_user)
        .service(put_user)
        .service(patch_user)
        .service(delete_user);
}

#[get("/user/{id}")]
pub async fn get_user_by_id(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AError> {
    let id_str = request.match_info().query("id").to_string();
    log::debug!("@@ id_str={}", id_str.to_string()); // #

    let id = id_str
        .parse::<i32>()
        .map_err(|_| AError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let result_user = web::block(move || {
        let mut conn = pool.get()?;
        // Find user by id.
        users_service::find_user_by_id(&mut conn, id)
    })
    .await??;
    let id_ = id.to_string();

    match result_user {
        Some(user_dto) => Ok(HttpResponse::Ok().json(user_dto)),
        None => Err(AError::NotFound(msg(ERR_NOT_FOUND_BY_ID, &[&id_]))),
    }
}

#[get("/user")]
pub async fn get_user_by_nickname(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AError> {
    let params = web::Query::<HashMap<String, String>>::from_query(request.query_string()).unwrap();
    let empty_line = "".to_string();
    let value = params.get("nickname").unwrap_or(&empty_line);
    let nickname = value.to_string();

    log::debug!("@@ nickname={}", nickname.to_string()); // #
    eprintln!("@@ nickname={}", nickname.to_string());
    if nickname.len() == 0 {
        return Err(AError::BadRequest(msg(ERR_INCORRECT_VALUE, &["nickname"])));
    }

    let user_list = web::block(move || {
        let mut conn = pool.get()?;
        // Find user by nickname.
        users_service::find_user_by_nickname(&mut conn, &nickname)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_list))
}

#[post("/user")]
pub async fn post_user(
    pool: web::Data<db::DbPool>,
    json_user_dto: web::Json<UserDTO>,
) -> actix_web::Result<HttpResponse, AError> {
    let user_dto = web::block(move || {
        let mut conn = pool.get()?;
        let new_user: UserDTO = json_user_dto.0;
        // Create a new entity (user).
        users_service::create_user(&mut conn, new_user)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user_dto))
}

#[put("/user/{id}")]
pub async fn put_user(
    pool: web::Data<db::DbPool>,
    request: actix_web::HttpRequest,
    json_user_dto: web::Json<UserDTO>,
) -> actix_web::Result<HttpResponse, AError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let mut new_user: UserDTO = json_user_dto.0.clone();
    UserDTO::clear_optional(&mut new_user);

    if UserDTO::is_empty(&new_user) {
        let msg = "The data model does not contain information to update.";
        return Err(AError::BadRequest(msg.to_string()));
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
) -> actix_web::Result<HttpResponse, AError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let mut new_user: UserDTO = json_user_dto.0.clone();
    UserDTO::clear_optional(&mut new_user);

    if UserDTO::is_empty(&new_user) {
        let msg = "The data model does not contain information to update.";
        return Err(AError::BadRequest(msg.to_string()));
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
) -> actix_web::Result<HttpResponse, AError> {
    let id_str = request.match_info().query("id").to_string();
    let id = id_str
        .parse::<i32>()
        .map_err(|_| AError::BadRequest(msg(ERR_CASTING_TO_TYPE, &[&id_str, "i32"])))?;

    let count = web::block(move || {
        let mut conn = pool.get()?;
        // Delete the entity (user) with the specified ID.
        users_service::delete_user(&mut conn, id)
    })
    .await??;

    if 0 == count {
        let id_ = id.to_string();
        Err(AError::NotFound(msg(ERR_NOT_FOUND_BY_ID, &[&id_])))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

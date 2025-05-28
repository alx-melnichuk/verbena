use std::{ops::Deref, time::Instant as tm};

use actix_web::{delete, post, web, HttpResponse};
use log::{error, info, log_enabled, Level::Info};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::blocked_user_orm::impls::BlockedUserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::blocked_user_orm::tests::BlockedUserOrmApp;
use crate::chats::{
    blocked_user_models::{
        BlockedUserDto, CreateBlockedUser, CreateBlockedUserDto, DeleteBlockedUser, DeleteBlockedUserDto,
    },
    blocked_user_orm::BlockedUserOrm,
};
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
use crate::validators::{msg_validation, Validator};

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // POST /api/blocked_users
            .service(post_blocked_user)
            // DELETE /api/blocked_users
            .service(delete_blocked_user);
    }
}

#[rustfmt::skip]
#[post("/api/blocked_users", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_blocked_user(
    authenticated: Authenticated,
    blocked_user_orm: web::Data<BlockedUserOrmApp>,
    json_body: web::Json<CreateBlockedUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }
    
    let create_blocked_user_dto: CreateBlockedUserDto = json_body.into_inner();
    let blocked_id = create_blocked_user_dto.blocked_id;
    let blocked_nickname = create_blocked_user_dto.blocked_nickname.clone();

    let create_blocked_user = CreateBlockedUser::new(user_id, blocked_id, blocked_nickname);

    let blocked_user_orm2 = blocked_user_orm.get_ref().clone();
    let res_blocked_user = web::block(move || {
        // Add a new entity (blocked_user).
        let res_blocked_user1 = blocked_user_orm2.create_blocked_user(create_blocked_user).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_blocked_user1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_blocked_user_dto = res_blocked_user?.map(|v| BlockedUserDto::from(v));

    if let Some(timer) = timer {
        info!("post_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(blocked_user_dto) = opt_blocked_user_dto {
        Ok(HttpResponse::Created().json(blocked_user_dto)) // 201
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[rustfmt::skip]
#[delete("/api/blocked_users", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_blocked_user(
    authenticated: Authenticated,
    blocked_user_orm: web::Data<BlockedUserOrmApp>,
    json_body: web::Json<DeleteBlockedUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
    // Get current user details.
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }
    
    let delete_blocked_user_dto: DeleteBlockedUserDto = json_body.into_inner();
    let blocked_id = delete_blocked_user_dto.blocked_id;
    let blocked_nickname = delete_blocked_user_dto.blocked_nickname.clone();

    let delete_blocked_user = DeleteBlockedUser::new(user_id, blocked_id, blocked_nickname);

    let blocked_user_orm2 = blocked_user_orm.get_ref().clone();
    let res_blocked_user = web::block(move || {
        // Add a new entity (blocked_user).
        let res_blocked_user1 = blocked_user_orm2.delete_blocked_user(delete_blocked_user).map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_blocked_user1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_blocked_user_dto = res_blocked_user?.map(|v| BlockedUserDto::from(v));

    if let Some(timer) = timer {
        info!("delete_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
    }
    if let Some(blocked_user_dto) = opt_blocked_user_dto {
        Ok(HttpResponse::Ok().json(blocked_user_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

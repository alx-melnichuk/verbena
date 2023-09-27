use actix_web::{delete, get, put, web, HttpResponse};
use log;
use std::ops::Deref;
use validator::Validate;

use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::users::user_models;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::user_orm::{UserOrm, CD_BLOCKING, CD_DATA_BASE};
use crate::utils::parse_err::{msg_parse_err, CD_PARSE_INT_ERROR, MSG_PARSE_INT_ERROR};

pub const CD_NOT_FOUND: &str = "NotFound";
pub const MSG_NOT_FOUND: &str = "The user with the specified ID was not found.";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/users/{id}
    cfg.service(get_user_by_id)
        // GET api/users/nickname/{nickname}
        .service(get_user_by_nickname)
        // GET api/user_me
        // .service(web::scope("")
        .service(get_user_me)
        // .wrap(RequireAuth::allowed_roles(
        //         // List of allowed roles.
        //         vec![UserRole::User, UserRole::Moderator, UserRole::Admin],
        //     )),
        // )
        // PUT api/users/{id}
        // .service(            web::scope("")
        .service(put_user)
        // .wrap(RequireAuth::allowed_roles(
        //     // List of allowed roles.
        //     vec![UserRole::User, UserRole::Moderator, UserRole::Admin],
        // )),
        // )
        // DELETE api/users/{id}
        // .service( web::scope("")
        .service(delete_user)
        // .wrap(RequireAuth::allowed_roles(
        //         // List of allowed roles.
        //         vec![UserRole::Admin],
        //     )),
        // )
        ;
}

// GET api/users/{id}
#[get("/users/{id}")]
pub async fn get_user_by_id(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = id_str.parse::<i32>().map_err(|e| {
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, &id_str, &e.to_string());
        log::warn!("{}: {}", CD_PARSE_INT_ERROR, msg);
        AppError::new(CD_PARSE_INT_ERROR, &msg.to_string()).set_status(400)
    })?;

    let result_user = web::block(move || {
        // Find user by id.
        let existing_user = user_orm
            .find_user_by_id(id)
            .map_err(|e| {
                log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            })
            .ok()?;

        existing_user
    })
    .await
    .map_err(|e| {
        log::warn!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })?;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}

// GET api/users/nickname/{nickname}
#[get("/users/nickname/{nickname}")]
pub async fn get_user_by_nickname(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let nickname = request.match_info().query("nickname").to_string();

    let result_user = web::block(move || {
        // Find user by nickname. Result <Vec<user_models::User>>.
        let res_user = user_orm
            .find_user_by_nickname(&nickname)
            .map_err(|e| {
                log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
                AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
            })
            .ok()?;

        res_user
    })
    .await
    .map_err(|e| {
        log::warn!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })?;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}

// GET api/user_me
#[get("/user_me")]
pub async fn get_user_me(
    authenticated: Authenticated,
) -> actix_web::Result<HttpResponse, AppError> {
    let user = authenticated.deref();
    let user_dto = user_models::UserDto::from(user.clone());

    Ok(HttpResponse::Ok().json(user_dto))
}

// PUT api/users/{id}
#[put("/users/{id}")]
pub async fn put_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
    json_user_dto: web::Json<user_models::ModifyUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = id_str.parse::<i32>().map_err(|e| {
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, &id_str, &e.to_string());
        log::warn!("{}: {}", CD_PARSE_INT_ERROR, msg);
        AppError::new(CD_PARSE_INT_ERROR, &msg.to_string()).set_status(400)
    })?;

    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::warn!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let modify_user: user_models::ModifyUserDto = json_user_dto.0.clone();

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user = user_orm.modify_user(id, modify_user).map_err(|e| {
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

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}
/*
// PATCH api/user/{id}
#[patch("/user/{id}")]
pub async fn patch_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
    json_user_dto: web::Json<user_models::UserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = id_str.parse::<i32>().map_err(|e| {
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, &id_str, &e.to_string());
        log::warn!("{}: {}", CD_PARSE_INT_ERROR, msg);
        AppError::new(CD_PARSE_INT_ERROR, &msg.to_string()).set_status(400)
    })?;

    let mut new_user: user_models::UserDto = json_user_dto.0.clone();

    if new_user.verify_for_edit() {
        log::warn!("{}: {}", CN_INCORRECT_PARAM, MSG_INCORRECT_PARAM);
        return Err(AppError::new(CN_INCORRECT_PARAM, MSG_INCORRECT_PARAM).set_status(400));
    }

    // let user_dto = new_user.clone();
    // let err_res1: Vec<AppError> = user_models::UserDto::validation?(&user_dto);
    // if err_res1.len() > 0 {
    //     return Ok(AppError::get_http_response(&err_res1));
    // }

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user = user_orm.modify_user(id, new_user).map_err(|e| {
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

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}
*/
// DELETE api/users/{id}
#[delete("/users/{id}")]
pub async fn delete_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = id_str.parse::<i32>().map_err(|e| {
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, &id_str, &e.to_string());
        log::warn!("{}: {}", CD_PARSE_INT_ERROR, msg);
        AppError::new(CD_PARSE_INT_ERROR, &msg.to_string()).set_status(400)
    })?;

    let result_count = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_count = user_orm.delete_user(id).map_err(|e| {
            log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
            AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500)
        });

        res_count
    })
    .await
    .map_err(|e| {
        log::warn!("{}: {}", CD_BLOCKING, e.to_string());
        AppError::new(CD_BLOCKING, &e.to_string()).set_status(500)
    })??;

    if 0 == result_count {
        Err(AppError::new(CD_NOT_FOUND, MSG_NOT_FOUND).set_status(404))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{http, test, web, App};
    use chrono::Utc;

    use super::*;
    use crate::errors::AppError;
    use crate::users::{
        user_models::{ModifyUserDto, UserDto},
        user_orm::tests::UserOrmApp,
    };

    const MSG_FAILED_DESER: &str = "Failed to deserialize AppError response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    #[test]
    async fn get_user_by_id_invalid_id() {
        let user_orm = web::Data::new(UserOrmApp::new());

        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(get_user_by_id),
        )
        .await;
        let user_id = "1a";
        let req = test::TestRequest::get()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user_id)) // GET /users/1a
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;

        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, user_id, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn get_user_by_id_valid_id() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();
        let user_orm = web::Data::new(UserOrmApp::create(users));

        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(get_user_by_id),
        )
        .await;
        let req = test::TestRequest::get()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // GET /users/1
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let buff = serde_json::json!(UserDto::from(user1.clone())).to_string();
        let user1_dto_ser: UserDto =
            serde_json::from_slice(buff.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res, user1_dto_ser);
    }

    #[test]
    async fn get_user_by_id_non_existent_id() {
        let user_orm = web::Data::new(UserOrmApp::new());

        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(get_user_by_id),
        )
        .await;
        let req = test::TestRequest::get()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/users/9999") // GET /users/9999
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn get_user_by_nickname_non_existent_nickname() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();
        let nickname = user1.nickname.to_uppercase().to_string();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(get_user_by_nickname),
        )
        .await;
        let req = test::TestRequest::get()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/nickname/{}_bad", nickname)) // GET /users/nickname/JAMES_SMITH
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn get_user_by_nickname_existent_nickname() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();
        let nickname = user1.nickname.to_uppercase().to_string();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(get_user_by_nickname),
        )
        .await;
        let req = test::TestRequest::get()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/nickname/{}", nickname)) // GET /users/nickname/JAMES_SMITH
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let buff = serde_json::json!(UserDto::from(user1.clone())).to_string();
        let user1_dto_ser: UserDto =
            serde_json::from_slice(buff.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res, user1_dto_ser);
    }

    #[test]
    async fn put_user_invalid_id() {
        let users = UserOrmApp::create_users();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let user_id = "1a";
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user_id)) // PUT user/1a
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, user_id, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn put_user_non_existent_id() {
        let users = UserOrmApp::create_users();
        let user1a: user_models::User = users.get(0).unwrap().clone();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}999", user1a.id)) // PUT users/1999
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn put_user_valid_id() {
        let new_nickname = "Oliver_Taylor".to_string(); // # TODO
        let new_email = "Oliver_Taylor@gmail.com".to_string();
        let new_password = "passwordD1T1".to_string();
        let new_role = user_models::UserRole::Admin;

        let users = UserOrmApp::create_users();
        let user1a: user_models::User = users.get(0).unwrap().clone();

        let mut user1b: user_models::User = UserOrmApp::new_user(
            user1a.id,
            new_nickname.clone(),
            new_email.clone(),
            new_password.clone(),
        );
        user1b.role = new_role.clone();
        user1b.created_at = user1a.created_at.clone();
        user1b.updated_at = Utc::now();

        let user1b_dto = UserDto::from(user1b);

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1a.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some(new_nickname.clone()),
                email: Some(new_email.clone()),
                password: Some(new_password.clone()),
                role: Some(new_role.clone()),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_user1b_dto = serde_json::json!(user1b_dto).to_string();
        let user1b_dto_ser: UserDto =
            serde_json::from_slice(json_user1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1b_dto_ser.id);
        #[rustfmt::skip]
        assert_eq!(user_dto_res.nickname, user1b_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1b_dto_ser.email);
        assert_eq!(user_dto_res.password, user1b_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1b_dto_ser.role);
        #[rustfmt::skip]
        assert_eq!(user_dto_res.created_at, user1b_dto_ser.created_at);
        #[rustfmt::skip]
        assert_ne!(user_dto_res.updated_at, user_dto_res.created_at);
    }

    #[test]
    async fn put_user_invalid_dto_nickname_min() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT user/1
            .set_json(ModifyUserDto {
                nickname: Some("Ol".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        assert_eq!(app_err.message, "nickname: must be more than 3 characters");
    }

    #[test]
    async fn put_user_invalid_dto_nickname_max() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();
        let nickname: String = (0..65).map(|_| 'a').collect();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some(nickname),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        #[rustfmt::skip]
        assert_eq!(app_err.message, "nickname: must be less than 64 characters");
    }

    #[test]
    async fn put_user_invalid_dto_email_min() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("o@us".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        assert_eq!(app_err.message, "email: must be more than 5 characters");
    }

    #[test]
    async fn put_user_invalid_dto_email_max() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();

        let suffix = "@gmail.com".to_string();
        let email: String = (0..(256 - suffix.len())).map(|_| 'a').collect();
        let email2 = format!("{}{}", email, suffix);

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some(email2.to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        #[rustfmt::skip]
        assert_eq!(app_err.message, "email: must be less than 255 characters");
    }

    #[test]
    async fn put_user_invalid_dto_password_min() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passw".to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        assert_eq!(app_err.message, "password: must be more than 6 characters");
    }

    #[test]
    async fn put_user_invalid_dto_password_max() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();
        let password: String = (0..65).map(|_| 'a').collect();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app =
            test::init_service(App::new().app_data(web::Data::clone(&user_orm)).service(put_user))
                .await;
        let req = test::TestRequest::put()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // PUT users/1
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some(password.to_string()),
                role: Some(user_models::UserRole::User),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, ERR_CN_VALIDATION);
        #[rustfmt::skip]
        assert_eq!(app_err.message, "password: must be less than 64 characters");
    }

    #[test]
    async fn delete_user_invalid_id() {
        let user_orm = web::Data::new(UserOrmApp::new());
        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(delete_user),
        )
        .await;
        let user_id = "1a";
        let req = test::TestRequest::delete()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user_id)) // DELETE /user/1a
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, user_id, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn delete_user_non_existent_id() {
        let user_orm = web::Data::new(UserOrmApp::new());
        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(delete_user),
        )
        .await;
        let req = test::TestRequest::delete()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/users/9999") // DELETE  /users/9999
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_NOT_FOUND);
        assert_eq!(app_err.message, MSG_NOT_FOUND);
    }

    #[test]
    async fn delete_user_existent_id() {
        let users = UserOrmApp::create_users();
        let user1: user_models::User = users.get(0).unwrap().clone();

        let user_orm = web::Data::new(UserOrmApp::create(users));
        let app = test::init_service(
            App::new().app_data(web::Data::clone(&user_orm)).service(delete_user),
        )
        .await;
        let req = test::TestRequest::delete()
            // .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .uri(&format!("/users/{}", user1.id)) // DELETE  /users/1
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 404
    }
}

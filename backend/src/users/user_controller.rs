use std::ops::Deref;

use actix_web::{delete, get, put, web, HttpResponse};
use log;
use serde_json::json;
use utoipa;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::{
    user_models::{self, ModifyUserDto, PasswordUserDto},
    user_orm::UserOrm,
};
use crate::utils::parser;
use crate::validators::{msg_validation, Validator};

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config // GET /api/users/email/{email}
            .service(get_user_by_email)
            // GET /api/users/nickname/{nickname}
            .service(get_user_by_nickname)
            // GET /api/users/{id}
            .service(get_user_by_id)
            // PUT /api/users/{id}
            .service(put_user)
            // DELETE /api/users/{id}
            .service(delete_user)
            // GET /api/users_current
            .service(get_user_current)
            // PUT /api/users_current
            .service(put_user_current)
            // DELETE /api/users_current
            .service(delete_user_current);
    }
}

/// get_user_by_email
///
/// Search for a user by his email.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users/email/demo1@gmail.us
/// ```
///
/// Return found user information with status 200 or 204 (no content) if the user is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "A user with the specified nickname was found.", body = JSON,
            example = json!({ "email": "demo1@gmail.us" })),
        (status = 204, description = "The user with the specified nickname was not found."),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("email", description = "User email value.")),
)]
#[get("/api/users/email/{email}")]
pub async fn get_user_by_email(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let email = request.match_info().query("email").to_string();

    let result_user = web::block(move || {
        // Find user by nickname. Result <Vec<user_models::User>>.
        let res_user = user_orm
            .find_user_by_nickname_or_email(None, Some(&email))
            .map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            })
            .ok()?;

        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(json!({ "email": user.email }))) // 200
    } else {
        Ok(HttpResponse::NoContent().json("")) // 204
    }
}

/// get_user_by_nickname
///
/// Search for a user by his nickname.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users/nickname/demo1
/// ```
///
/// Return found user information with status 200 or 204 (no content) if the user is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "A user with the specified nickname was found.", body = JSON,
            example = json!({ "nickname": "demo1" })),
        (status = 204, description = "The user with the specified nickname was not found."),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("nickname", description = "User nickname value.")),
)]
#[get("/api/users/nickname/{nickname}")]
pub async fn get_user_by_nickname(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let nickname = request.match_info().query("nickname").to_string();

    let result_user = web::block(move || {
        // Find user by nickname.
        let res_user = user_orm
            .find_user_by_nickname_or_email(Some(&nickname), None)
            .map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            })
            .ok()?;

        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(json!({ "nickname": user.nickname }))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// get_user_by_id
///
/// Search for a user by his ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users/1
/// ```
///
/// Return the found specified user (`UserDto`) with status 200 or 204 (no content) if the user is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "A user with the specified ID was found.", body = UserDto),
        (status = 204, description = "The user with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 415, description = "Error parsing input parameter.", body = AppError, 
            example = json!(AppError::unsupported_type415(&"parsing_type_not_supported: `id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/users/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())" )]
pub async fn get_user_by_id(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_UNSUPPORTED_TYPE, &message);
        AppError::unsupported_type415(&message)
    })?;

    let result_user = web::block(move || {
        // Find user by id.
        let existing_user =
            user_orm.find_user_by_id(id).map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            }).ok()?;

        existing_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    if let Some(user) = result_user {
        let user_dto = user_models::UserDto::from(user);
        Ok(HttpResponse::Ok().json(user_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// put_user
///
/// Update the data of the specified user (`UserDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/users/1  -d {"password": "new_password"}
/// ```
///
/// Return the found specified user (`UserDto`) with status 200 or 204 (no content) if the user is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Data about the specified user.", body = UserDto),
        (status = 204, description = "The specified user was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 415, description = "Error parsing input parameter.", body = AppError, 
            example = json!(AppError::unsupported_type415(&"parsing_type_not_supported: `id` - invalid digit found in string (2a)"))),
        (status = 417, description = "Validation error. `{ \"password\": \"pas\" }`", body = [AppError],
            example = json!(AppError::validations((PasswordUserDto { password: Some("pas".to_string()) }).validate().err().unwrap()) )),
        (status = 506, description = "Blocking error.", body = AppError,
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError,
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[put("/api/users/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn put_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<PasswordUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_UNSUPPORTED_TYPE, &message);
        AppError::unsupported_type415(&message)
    })?;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let password_user: PasswordUserDto = json_body.into_inner();
    let modify_user: ModifyUserDto = ModifyUserDto { nickname: None, email: None, password: password_user.password, role: None };

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user =
            user_orm.modify_user(id, modify_user).map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })??;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_user
///
/// Delete the specified user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/users/1
/// ```
///
/// Return the deleted user (`UserDto`) with status 200 or 204 (no content) if the user is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified user was deleted successfully.", body = UserDto),
        (status = 204, description = "The specified user was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 415, description = "Error parsing input parameter.", body = AppError, 
            example = json!(AppError::unsupported_type415(&"parsing_type_not_supported: `id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/users/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn delete_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_UNSUPPORTED_TYPE, &message);
        AppError::unsupported_type415(&message)
    })?;

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user = user_orm.delete_user(id)
        .map_err(|e| {
            log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })??;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// get_user_current
///
/// Get information about the current user (`UserDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/users_current
/// ```
///
/// Return the current user (`UserDto`) with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Data about the current user.", body = UserDto),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[get("/api/users_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_user_current(
    authenticated: Authenticated,
) -> actix_web::Result<HttpResponse, AppError> {
    let user = authenticated.deref();
    let user_dto = user_models::UserDto::from(user.clone());

    Ok(HttpResponse::Ok().json(user_dto)) // 200
}

/// put_user_current
///
/// Update the data of the current user (`UserDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/users_current  -d {"password": "new_password"}
/// ```
///
/// Return the current user (`UserDto`) with status 200 or 204 (no content) if the user is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Data about the current user.", body = UserDto),
        (status = 204, description = "The current user was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 417, description = "Validation error. `{ \"password\": \"pas\" }`", body = [AppError],
            example = json!(AppError::validations((PasswordUserDto { password: Some("pas".to_string()) }).validate().err().unwrap()) )),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[put("/api/users_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_user_current(
    authenticated: Authenticated,
    user_orm: web::Data<UserOrmApp>,
    json_body: web::Json<PasswordUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let user = authenticated.deref();
    let id = user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let password_user: PasswordUserDto = json_body.into_inner();
    let modify_user: ModifyUserDto = ModifyUserDto { nickname: None, email: None, password: password_user.password, role: None };

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user =
            user_orm.modify_user(id, modify_user).map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })??;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_user_current
///
/// Delete the current user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/users_current
/// ```
///
/// Return the current user (`UserDto`) with status 200 or 204 (no content) if the current user is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The current user was deleted successfully.", body = UserDto),
        (status = 204, description = "The current user was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/users_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_user_current(
    authenticated: Authenticated,
    user_orm: web::Data<UserOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let user = authenticated.deref();
    let id = user.id;

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user = user_orm.delete_user(id)
        .map_err(|e| {
            log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });

        res_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })??;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
        http::{
            self,
            header::{HeaderValue, CONTENT_TYPE},
            StatusCode,
        },
        test, web, App,
    };
    use chrono::Utc;

    use crate::errors::AppError;
    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::users::user_models::{User, UserDto, UserModelsTest, UserRole};

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
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    fn configure_user(
        config_jwt: config_jwt::ConfigJwt, // configuration
        data_c: (Vec<User>, Vec<Session>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm));
        }
    }
    #[rustfmt::skip]
    fn get_cfg_data() -> (config_jwt::ConfigJwt, (Vec<User>, Vec<Session>), String) {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let data_c = (vec![user1], vec![session1]);
        (config_jwt, data_c, token)
    }
    fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** get_user_by_email **
    #[actix_web::test]
    async fn test_get_user_by_email_non_existent_email() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let email = format!("a{}", data_c.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_email).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/email/{}", email))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_get_user_by_email_existent_email() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let email = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_email).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/email/{}", email))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let email_res = std::str::from_utf8(&body).unwrap();
        let str = format!("{}\"email\":\"{}\"{}", "{", email, "}");
        assert_eq!(email_res, str);
    }

    // ** get_user_by_nickname **
    #[actix_web::test]
    async fn test_get_user_by_nickname_non_existent_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let nickname = format!("a{}", data_c.0.get(0).unwrap().nickname.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_nickname).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/nickname/{}", nickname))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_get_user_by_nickname_existent_nickname() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let nickname = data_c.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_nickname).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/nickname/{}", nickname))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let nickname_res = std::str::from_utf8(&body).unwrap();
        let str = format!("{}\"nickname\":\"{}\"{}", "{", nickname, "}");
        assert_eq!(nickname_res, str);
    }

    // ** get_user_by_id **
    #[actix_web::test]
    async fn test_get_user_by_id_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user_id_bad = format!("{}a", user1.id);
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_id).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/{}", user_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        #[rustfmt::skip]
        let msg = format!("{}: `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, user_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_user_by_id_valid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user1 = data_c.0.get(0).unwrap().clone();
        user1.role = UserRole::Admin;
        let user_orm = UserOrmApp::create(&vec![
            user1,
            UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdL2S2"),
        ]);
        let user_vec = user_orm.user_vec.clone();
        let user2 = user_vec.get(1).unwrap().clone();
        let user2_dto = UserDto::from(user2.clone());
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_id).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/{}", &user2.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(user2_dto).to_string();
        let user2b_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(user_dto_res, user2b_dto_ser);
        assert_eq!(user_dto_res.password, "");
    }
    #[actix_web::test]
    async fn test_get_user_by_id_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user1 = data_c.0.get(0).unwrap().clone();
        user1.role = UserRole::Admin;
        let user_orm = UserOrmApp::create(&vec![
            user1,
            UserOrmApp::new_user(2, "Logan_Lewis", "Logan_Lewis@gmail.com", "passwdL2S2"),
        ]);
        let user_vec = user_orm.user_vec.clone();
        let user2 = user_vec.get(1).unwrap().clone();
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_by_id).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/users/{}", (user2.id + 1)))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    // ** put_user **
    #[actix_web::test]
    async fn test_put_user_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user_id_bad = format!("{}a", user1.id);
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user_id_bad))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto {
                password: Some("passwdQ0W0".to_string()),
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        #[rustfmt::skip]
        let msg = format!("{}: `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, user_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_put_user_invalid_dto_password_empty() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some("".to_string()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[user_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_user_invalid_dto_password_min() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some(UserModelsTest::password_min()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[user_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_user_invalid_dto_password_max() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some(UserModelsTest::password_max()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[user_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_user_invalid_dto_password_wrong() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some(UserModelsTest::password_wrong()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[user_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_user_user_not_exist() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id + 1))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some("passwdQ0W0".to_string()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_user_valid_id() {
        let new_password = "passwdQ0W0";
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1_id = user1.id;
        let mut user1mod: User = user1.clone();
        user1mod.password = new_password.to_string();
        user1mod.updated_at = Utc::now();
        let user1mod_dto = UserDto::from(user1mod.clone());
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/users/{}", user1_id))
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some(new_password.to_string()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(user1mod_dto).to_string();
        let user1mod_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1mod_dto_ser.id);
        assert_eq!(user_dto_res.nickname, user1mod_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1mod_dto_ser.email);
        assert_eq!(user_dto_res.password, user1mod_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1mod_dto_ser.role);
        assert_eq!(user_dto_res.created_at, user1mod_dto_ser.created_at);
    }

    // ** delete_user **
    #[actix_web::test]
    async fn test_delete_user_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user_id_bad = format!("{}a", user1.id);
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/users/{}", user_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        #[rustfmt::skip]
        let msg = format!("{}: `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, user_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_delete_user_user_not_exist() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user_id = user1.id;
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/users/{}", user_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_user_user_exists() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        let user1copy_dto = UserDto::from(user1.clone());
        let data_c = (user_vec, data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_user).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/users/{}", user1copy_dto.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(user1copy_dto).to_string();
        let user1copy_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1copy_dto_ser.id);
        assert_eq!(user_dto_res.nickname, user1copy_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1copy_dto_ser.email);
        assert_eq!(user_dto_res.password, user1copy_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1copy_dto_ser.role);
        assert_eq!(user_dto_res.created_at, user1copy_dto_ser.created_at);
        assert_eq!(user_dto_res.updated_at, user1copy_dto_ser.updated_at);
    }

    // ** get_user_current **
    #[actix_web::test]
    async fn test_get_user_current_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();
        let user1_dto = UserDto::from(user1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_user_current).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/users_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(user1_dto).to_string();
        let user1b_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res, user1b_dto_ser);
        assert_eq!(user_dto_res.password, "");
    }

    // ** put_user_current **
    #[actix_web::test]
    async fn test_put_user_current_valid_id() {
        let new_password = "passwdJ3S9";
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user1mod: User = data_c.0.get(0).unwrap().clone();
        user1mod.password = new_password.to_string();
        let user1mod_dto = UserDto::from(user1mod);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_user_current).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/users_current")
            .insert_header(header_auth(&token))
            .set_json(PasswordUserDto { password: Some(new_password.to_string()) })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(user1mod_dto).to_string();
        let user1mod_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1mod_dto_ser.id);
        assert_eq!(user_dto_res.nickname, user1mod_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1mod_dto_ser.email);
        assert_eq!(user_dto_res.password, user1mod_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1mod_dto_ser.role);
        assert_eq!(user_dto_res.created_at, user1mod_dto_ser.created_at);
    }

    // ** delete_user_current **
    #[actix_web::test]
    async fn test_delete_user_current_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1: User = data_c.0.get(0).unwrap().clone();
        let user1copy_dto = UserDto::from(user1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_user_current).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/users_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(user1copy_dto).to_string();
        let user1copy_dto_ser: UserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1copy_dto_ser.id);
        assert_eq!(user_dto_res.nickname, user1copy_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1copy_dto_ser.email);
        assert_eq!(user_dto_res.password, user1copy_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1copy_dto_ser.role);
        assert_eq!(user_dto_res.created_at, user1copy_dto_ser.created_at);
        assert_eq!(user_dto_res.updated_at, user1copy_dto_ser.updated_at);
    }
}

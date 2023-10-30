use actix_web::{delete, get, put, web, HttpResponse};
use log;
use std::ops::Deref;
use validator::Validate;

use crate::errors::{AppError, CN_VALIDATION};
use crate::extractors::authentication::{Authenticated, RequireAuth};
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::{user_models, user_orm::UserOrm};
use crate::utils::{
    err,
    parser::{parse_i32, CD_PARSE_INT_ERROR},
};

pub const CD_NOT_FOUND: &str = "NotFound";
pub const MSG_NOT_FOUND: &str = "The user with the specified ID was not found.";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/users/{id}
    cfg.service(get_user_by_id)
        // GET api/users/nickname/{nickname}
        .service(get_user_by_nickname)
        // GET api/user_current
        .service(get_user_current)
        // PUT api/users/{id}
        .service(put_user)
        // DELETE api/users/{id}
        .service(delete_user);
}

fn err_database(err: String) -> AppError {
    log::error!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}

// GET api/users/{id}
#[get("/users/{id}")]
pub async fn get_user_by_id(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parse_i32(&id_str).map_err(|err| {
        log::error!("{CD_PARSE_INT_ERROR}: id: {err}");
        AppError::new(CD_PARSE_INT_ERROR, &format!("id: {err}")).set_status(400)
    })?;

    let result_user = web::block(move || {
        // Find user by id.
        let existing_user =
            user_orm.find_user_by_id(id).map_err(|e| err_database(e.to_string())).ok()?;

        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

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
            .map_err(|e| err_database(e.to_string()))
            .ok()?;

        res_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}

// GET api/user_current
#[rustfmt::skip]
#[get("/user_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_user_current(
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

    let id = parse_i32(&id_str).map_err(|err| {
        log::error!("{CD_PARSE_INT_ERROR}: id: {err}");
        AppError::new(CD_PARSE_INT_ERROR, &format!("id: {err}")).set_status(400)
    })?;

    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::error!("{}: {}", CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let modify_user: user_models::ModifyUserDto = json_user_dto.0.clone();

    let result_user = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_user =
            user_orm.modify_user(id, modify_user).map_err(|e| err_database(e.to_string()));

        res_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

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

    let id = parse_i32(&id_str).map_err(|err| {
        log::error!("{CD_PARSE_INT_ERROR}: id: {err}");
        AppError::new(CD_PARSE_INT_ERROR, &format!("id: {err}")).set_status(400)
    })?;

    let mut new_user: user_models::UserDto = json_user_dto.0.clone();

    if new_user.verify_for_edit???() {
        log::error!("{}: {}", CN_INCORRECT_PARAM, MSG_INCORRECT_PARAM);
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
            log::error!("{}: {}", err::CD_DATA_BASE, e.to_string());
            AppError::new(err::CD_DATA_BASE, &e.to_string()).set_status(500)
        });

        res_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    if let Some(user) = result_user {
        Ok(HttpResponse::Ok().json(user_models::UserDto::from(user)))
    } else {
        Ok(HttpResponse::NoContent().json(""))
    }
}
*/
// DELETE api/users/{id}
#[rustfmt::skip]
#[delete("/users/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn delete_user(
    user_orm: web::Data<UserOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parse_i32(&id_str).map_err(|err| {
        log::error!("{CD_PARSE_INT_ERROR}: id: {err}");
        AppError::new(CD_PARSE_INT_ERROR, &format!("id: {err}")).set_status(400)
    })?;

    let result_count = web::block(move || {
        // Modify the entity (user) with new data. Result <user_models::User>.
        let res_count = user_orm.delete_user(id)
        .map_err(|e| err_database(e.to_string()));

        res_count
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    if 0 == result_count {
        Err(AppError::new(CD_NOT_FOUND, MSG_NOT_FOUND).set_status(404))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::Utc;

    use crate::errors::AppError;
    use crate::sessions::{
        config_jwt::{self, ConfigJwt},
        session_models::Session,
        session_orm::tests::SessionOrmApp,
        tokens::encode_dual_token,
    };
    use crate::users::{
        user_models::{ModifyUserDto, User, UserDto, UserRole, UserValidateTest},
        user_orm::tests::{UserOrmApp, USER_ID_1},
    };
    use crate::utils::parser::{CD_PARSE_INT_ERROR, MSG_PARSE_INT_ERROR};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(
            USER_ID_1,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
        );
        user.role = UserRole::User;
        user
    }

    async fn call_service_inn(
        user_vec: Vec<User>,
        factory: impl dev::HttpServiceFactory + 'static,
        test_request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));

        let app = test::init_service(
            App::new().app_data(web::Data::clone(&data_user_orm)).service(factory),
        )
        .await;
        let req = test_request.to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_get_user_by_id_invalid_id() {
        let user_id_bad = "1a";
        let req = test::TestRequest::get().uri(&format!("/users/{}", &user_id_bad)); // GET /users/${id}

        let resp = call_service_inn(vec![], get_user_by_id, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = format!("id: {MSG_PARSE_INT_ERROR} `{user_id_bad}` - {MSG_CASTING_TO_TYPE}");
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn test_get_user_by_id_valid_id() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();
        let user1b_dto = UserDto::from(user1.clone());

        let req = test::TestRequest::get().uri(&format!("/users/{}", &user_id)); // GET /users/${id}

        let resp = call_service_inn(vec![user1], get_user_by_id, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_user1b_dto = serde_json::json!(user1b_dto).to_string();
        let user1_dto_ser: UserDto =
            serde_json::from_slice(json_user1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res, user1_dto_ser);
    }

    #[test]
    async fn test_get_user_by_id_non_existent_id() {
        let req = test::TestRequest::get().uri(&"/users/9999"); // GET /users/${id}

        let resp = call_service_inn(vec![], get_user_by_id, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn test_get_user_by_nickname_non_existent_nickname() {
        let req = test::TestRequest::get().uri(&"/users/nickname/JAMES_SMITH"); // GET /users/nickname/${nickname}

        let resp = call_service_inn(vec![], get_user_by_nickname, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn test_get_user_by_nickname_existent_nickname() {
        let user1: User = create_user();
        let user1b_dto = UserDto::from(user1.clone());
        let nickname = user1.nickname.to_uppercase().to_string();

        let req = test::TestRequest::get().uri(&format!("/users/nickname/{}", nickname)); // GET /users/nickname/${nickname}

        let resp = call_service_inn(vec![user1], get_user_by_nickname, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_user1b_dto = serde_json::json!(user1b_dto).to_string();
        let user1_dto_ser: UserDto =
            serde_json::from_slice(json_user1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res, user1_dto_ser);
    }

    async fn call_service_auth(
        user_vec: Vec<User>,
        session_vec: Vec<Session>,
        config_jwt: ConfigJwt,
        token: &str,
        factory: impl dev::HttpServiceFactory + 'static,
        test_request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(config_jwt);
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_session_orm = web::Data::new(SessionOrmApp::create(session_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(factory),
        )
        .await;
        let req = test_request
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_get_user_current_valid_token() {
        let user1: User = create_user();
        let user1b_dto = UserDto::from(user1.clone());

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::get().uri("/user_current");
        let user_v = vec![user1];
        let resp =
            call_service_auth(user_v, session_v, config_jwt, &token, get_user_current, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_user1b_dto = serde_json::json!(user1b_dto).to_string();
        let user1b_dto_ser: UserDto =
            serde_json::from_slice(json_user1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1b_dto_ser.id);
        assert_eq!(user_dto_res.nickname, user1b_dto_ser.nickname);
        assert_eq!(user_dto_res.email, user1b_dto_ser.email);
        assert_eq!(user_dto_res.password, user1b_dto_ser.password);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1b_dto_ser.role);
        assert_eq!(user_dto_res.created_at, user1b_dto_ser.created_at);
        assert_eq!(user_dto_res.updated_at, user_dto_res.created_at);
    }

    #[test]
    async fn test_put_user_invalid_id() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();
        let user_id_bad = format!("{}a", user_id).to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id_bad)) // PUT users/{id}a
            .set_json(ModifyUserDto {
                nickname: Some("Oliver_Taylor".to_string()),
                email: Some("Oliver_Taylor@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = format!("id: {MSG_PARSE_INT_ERROR} `{user_id_bad}` - {MSG_CASTING_TO_TYPE}");
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn test_put_user_non_existent_id() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}9", &user_id)) // PUT users/{id}9
            .set_json(ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    #[test]
    async fn test_put_user_valid_id() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();
        let new_password = "passwdJ3S9";

        let mut user1b: User = UserOrmApp::new_user(
            user1.id,
            "James_Smith",
            "James_Smith@gmail.com",
            new_password.clone(),
        );
        user1b.role = UserRole::Admin;
        user1b.created_at = user1.created_at.clone();
        user1b.updated_at = Utc::now();

        let user1b_dto = UserDto::from(user1b.clone());

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(ModifyUserDto {
                nickname: Some(user1b.nickname),
                email: Some(user1b.email),
                password: Some(new_password.to_string()),
                role: Some(user1b.role),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
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
    async fn test_put_user_invalid_dto_nickname_min() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some(UserValidateTest::nickname_min()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MIN);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_nickname_max() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some(UserValidateTest::nickname_max()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MAX);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_wrong_nickname() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some(UserValidateTest::nickname_wrong()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_REGEX);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_email_min() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some(UserValidateTest::email_min()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MIN);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_email_max() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some(UserValidateTest::email_max()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MAX);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_wrong_email() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some(UserValidateTest::email_wrong()),
                password: Some("passwordD1T1".to_string()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_password_min() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some(UserValidateTest::password_min()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MIN);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_put_user_invalid_dto_password_max() {
        let user1: User = create_user();
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/users/{}", &user_id)) // PUT users/{id}
            .set_json(user_models::ModifyUserDto {
                nickname: Some("James_Smith".to_string()),
                email: Some("James_Smith@gmail.com".to_string()),
                password: Some(UserValidateTest::password_max()),
                role: Some(UserRole::User),
            });
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, put_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MAX);
        assert_eq!(app_err.message, msg_err);
    }

    #[test]
    async fn test_delete_user_invalid_id() {
        let mut user1: User = create_user();
        user1.role = UserRole::Admin;
        let user_id = user1.id.to_string();
        let user_id_bad = format!("{}a", user_id).to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::delete().uri(&format!("/users/{}", &user_id_bad)); // DELETE /user/{id}a
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, delete_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        let msg = format!("id: {MSG_PARSE_INT_ERROR} `{user_id_bad}` - {MSG_CASTING_TO_TYPE}");
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn test_delete_user_non_existent_id() {
        let mut user1: User = create_user();
        user1.role = UserRole::Admin;
        let user_id_bad = format!("{}", user1.id + 1).to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token =      collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::delete().uri(&format!("/users/{}", user_id_bad)); // DELETE /user/1002
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, delete_user, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_NOT_FOUND);
        assert_eq!(app_err.message, MSG_NOT_FOUND);
    }

    #[test]
    async fn test_delete_user_existent_id() {
        let mut user1: User = create_user();
        user1.role = UserRole::Admin;
        let user_id = user1.id.to_string();

        let num_token = 1234;
        let session_v = vec![SessionOrmApp::new_session(user1.id, Some(num_token))];

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        // let jwt_access = config_jwt.jwt_access;
        // let token = collect_token(user1.id, num_token, jwt_secret, jwt_access).unwrap();
        let token =
            encode_dual_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let req = test::TestRequest::delete().uri(&format!("/users/{}", &user_id)); // DELETE /user/{id}
        let user_v = vec![user1];
        let resp = call_service_auth(user_v, session_v, config_jwt, &token, delete_user, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK); // 200
    }
}

use actix_web::{cookie::time::Duration as ActixWebDuration, cookie::Cookie, post, web, HttpResponse};
use utoipa;

use crate::errors::AppError;
use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(feature = "mockdata")]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    profile_models::{self, Profile, LoginProfileDto},
    profile_err as p_err,
    profile_orm::ProfileOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_orm::SessionOrm, tokens};
use crate::settings::err;
use crate::validators::{msg_validation, Validator};

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        // POST /api/login
        config
            .service(login);
    }
}

/// login
///
/// User authentication to enter an authorized state.
///
/// Open a session for the current user.
///
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/login \
/// -d '{"nickname": "user01", "password": "password"}' \
/// -H 'Content-Type: application/json'
/// ```
///
/// Returns the current user's profile (`ProfileDto`) and open session token (`ProfileTokensDto`) with status 200.
///
#[utoipa::path(
    responses(
        ( status = 200, description = "The current user's profile and the open session token.",
            body = LoginProfileResponseDto),
        (status = 401, description = "The nickname or password is incorrect.", body = AppError, examples(
            ("Nickname" = (summary = "Nickname is incorrect",
                description = "The nickname is incorrect.",
                value = json!(AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL)))),
            ("Password" = (summary = "Password is incorrect", 
                description = "The password is incorrect.",
                value = json!(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT))))
        )),
        (status = 417, body = [AppError], description = 
            "Validation error. `curl -i -X POST http://localhost:8080/api/login -d '{ \"nickname\": \"us\", \"password\": \"pas\" }'`",
            example = json!(AppError::validations(
                (LoginProfileDto { nickname: "us".to_string(), password: "pas".to_string() }).validate().err().unwrap()) )),
        ( status = 406, description = "Error opening session.", body = AppError,
            example = json!(AppError::not_acceptable406(&format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, 1)))),
        (status = 409, description = "Error when comparing password hashes.", body = AppError,
            example = json!(AppError::conflict409(&format!("{}: {}", err::MSG_INVALID_HASH, "Parameter is empty.")))),
        ( status = 422, description = "Token encoding error.", body = AppError,
            example = json!(AppError::unprocessable422(&format!("{}: {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, "InvalidKeyFormat")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
)]
#[post("/api/login")]
pub async fn login(
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    profile_orm: web::Data<ProfileOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<LoginProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors)); // 417
        return Ok(AppError::to_response(&AppError::validations(validation_errors)));
    }

    let login_profile_dto: LoginProfileDto = json_body.into_inner();
    let nickname = login_profile_dto.nickname.clone();
    let email = login_profile_dto.nickname.clone();
    let password = login_profile_dto.password.clone();

    let profile = web::block(move || {
        let is_password = true;
        // Find user's profile by nickname or email.
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(Some(&nickname), Some(&email), is_password)
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        existing_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let profile: Profile = profile.ok_or_else(|| {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
    })?;
    eprintln!("profile.password: {}", profile.password.to_string());
    let profile_password = profile.password.to_string();
    let password_matches = hash_tools::compare_hash(&password, &profile_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_HASH, &e);
        log::error!("{}: {}", err::CD_CONFLICT, &message);
        AppError::conflict409(&message) // 409
    })?;

    if !password_matches {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_PASSWORD_INCORRECT);
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    let num_token = tokens::generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Packing two parameters (user_id, num_token) into access_token.
    let access_token = tokens::encode_token(profile.user_id, num_token, jwt_secret, config_jwt.jwt_access).map_err(|e| {
        let message = format!("{}: {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message) // 422
    })?;

    // Packing two parameters (user_id, num_token) into refresh_token.
    let refresh_token = tokens::encode_token(profile.user_id, num_token, jwt_secret, config_jwt.jwt_refresh).map_err(|e| {
        let message = format!("{}: {}", p_err::MSG_JSON_WEB_TOKEN_ENCODE, &e);
        log::error!("{}: {}", err::CD_UNPROCESSABLE_ENTITY, &message);
        AppError::unprocessable422(&message)
    })?;

    let opt_session = web::block(move || {
        // Modify the entity (session) with new data. Result <Option<Session>>.
        let res_session = session_orm.modify_session(profile.user_id, Some(num_token)).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_session
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if opt_session.is_none() {
        let message = format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, profile.user_id);
        log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
        return Err(AppError::not_acceptable406(&message)); // 406
    }

    let profile_tokens_dto = profile_models::ProfileTokensDto {
        access_token: access_token.to_owned(),
        refresh_token,
    };

    let login_profile_response_dto = profile_models::LoginProfileResponseDto {
        profile_dto: profile_models::ProfileDto::from(profile),
        profile_tokens_dto,
    };

    let cookie = Cookie::build("token", access_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config_jwt.jwt_access, 0))
        .http_only(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).json(login_profile_response_dto)) // 200
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use profile_models::{ProfileDto, ProfileTest};
    use serde_json::json;

    use crate::{
        sessions::{config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token},
        users::user_models::UserRole,
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_profile() -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);
        profile
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_profile_pwd(nickname: &str, email: &str, password: &str) -> Profile {
        let mut profile = ProfileOrmApp::new_profile(1, nickname, email, UserRole::User);
        profile.password = hash_tools::encode_hash(password).unwrap(); // hashed
        profile
    }

    fn get_cfg_data() -> (config_jwt::ConfigJwt, (Vec<Profile>, Vec<Session>), String) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile());
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(profile1.user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(profile1.user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let data_c = (vec![profile1], vec![session1]);

        (config_jwt, data_c, token)
    }
    fn configure_profile(
        config_jwt: config_jwt::ConfigJwt,    // configuration
        data_c: (Vec<Profile>, Vec<Session>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm));
        }
    }
    fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** login **
    #[actix_web::test]
    async fn test_login_no_data() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        let req = test::TestRequest::post().uri("/api/login").to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_login_empty_json_object() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        let req = test::TestRequest::post().uri("/api/login").set_json(json!({})).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "".to_string(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_min(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_max(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_nickname_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::nickname_wrong(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_min(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_max(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_email_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: ProfileTest::email_wrong(), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: "".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_min()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_max()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_login_invalid_dto_password_wrong() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: "James_Smith".to_string(), password: ProfileTest::password_wrong()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_login_if_nickname_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = data_c.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: format!("a{}", nickname), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (A)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_email_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let email = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: format!("a{}", email), password: "passwordD1T1".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (A)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_WRONG_NICKNAME_EMAIL);
    }
    #[actix_web::test]
    async fn test_login_if_password_invalid_hash() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Oliver_Taylor".to_string();
        let password = "hash_password_R2B2";
        let mut profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &"1".to_string());
        profile1.password = password.to_string();
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: nickname.to_string(), password: password.to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONFLICT);
        assert!(app_err.message.starts_with(err::MSG_INVALID_HASH));
    }
    #[actix_web::test]
    async fn test_login_if_password_incorrect() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto {
                nickname: nickname.to_string(), password: format!("{}b", password)
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401 (B)

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_login_err_jsonwebtoken_encode() {
        let (mut cfg_c, data_c, _token) = get_cfg_data();
        let nickname = "Robert_Brown".to_string();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        cfg_c.jwt_secret = "".to_string();
        let data_c = (vec![profile1], data_c.1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNPROCESSABLE_ENTITY); // 422

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNPROCESSABLE_ENTITY);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: InvalidKeyFormat", p_err::MSG_JSON_WEB_TOKEN_ENCODE));
    }
    #[actix_web::test]
    async fn test_login_if_session_not_exist() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let data_c = (vec![profile1], vec![]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, profile1_id));
    }
    #[actix_web::test]
    async fn test_login_valid_credentials() {
        let (cfg_c, data_c, _token) = get_cfg_data();
        let profile1 = data_c.0.get(0).unwrap().clone();
        let nickname = profile1.nickname.clone();
        let password = "passwdR2B2";
        let profile1 = create_profile_pwd(&nickname, &format!("{}@gmail.com", &nickname), &password.to_string());
        let profile_vec = ProfileOrmApp::create(&vec![profile1.clone()]).profile_vec;
        let profile1_dto = ProfileDto::from(profile_vec.get(0).unwrap().clone());

        let data_c = (vec![profile1], data_c.1);
        let jwt_access = cfg_c.jwt_access;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(login).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/login")
            .set_json(LoginProfileDto { nickname: nickname.to_string(), password: password.to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let token_cookie_opt = resp.response().cookies().find(|cookie| cookie.name() == "token");
        assert!(token_cookie_opt.is_some());

        let token = token_cookie_opt.unwrap();
        let token_value = token.value().to_string();
        assert!(token_value.len() > 0);
        let max_age = token.max_age();
        assert!(max_age.is_some());
        let max_age_value = max_age.unwrap();
        assert_eq!(max_age_value, ActixWebDuration::new(jwt_access, 0));
        assert_eq!(true, token.http_only().unwrap());

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let login_resp: profile_models::LoginProfileResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let access_token: String = login_resp.profile_tokens_dto.access_token;
        assert!(!access_token.is_empty());
        let refresh_token: String = login_resp.profile_tokens_dto.refresh_token;
        assert!(refresh_token.len() > 0);

        let profile_dto_res = login_resp.profile_dto;
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }

}
use std::{borrow::Cow::Borrowed, ops::Deref};

use actix_web::{get, web, HttpResponse};
use log;
use serde_json::json;
use utoipa;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    // config_prfl,
    profile_models::{Profile, ProfileDto, UniquenessProfileDto, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF},
    profile_orm::ProfileOrm,
};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::{user_models::UserRole, user_registr_orm::UserRegistrOrm};
use crate::utils::parser;

// None of the parameters are specified.
const MSG_PARAMETERS_NOT_SPECIFIED: &str = "parameters_not_specified";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /api/profiles/uniqueness
            .service(uniqueness_check)
            // GET /api/profiles/{id}
            .service(get_profile_by_id)
            // GET /api/profiles_current
            .service(get_profile_current);
    }
}

/// uniqueness_check
///
/// Checking the uniqueness of the user's "nickname" or "email".
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles/uniqueness?nickname=demo1
/// ```
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles/uniqueness?email=demo1@gmail.us
/// ```
///
/// If the value is already in use, then "{\"uniqueness\":false}" is returned with status 200.
/// If the value is not yet used, then "{\"uniqueness\":true}" is returned with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The result of checking whether nickname (email) is already in use.", body = JSON, examples(
            ("already_use" = (summary = "already in use", 
                description = "If the nickname (email) is already in use.",
                value = json!({ "uniqueness": false }))),
            ("not_use" = (summary = "not yet in use", 
                description = "If the nickname (email) is not yet used.",
                value = json!({ "uniqueness": true })))
        )),
        (status = 406, description = "None of the parameters are specified.", body = AppError,
            example = json!(AppError::not_acceptable406(MSG_PARAMETERS_NOT_SPECIFIED)
                .add_param(Borrowed("invalidParams"), &serde_json::json!({ "nickname": "null", "email": "null" })))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
)]
#[get("/api/profiles/uniqueness")]
pub async fn uniqueness_check(
    profile_orm: web::Data<ProfileOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    query_params: web::Query<UniquenessProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get search parameters.
    let uniqueness_user_dto: UniquenessProfileDto = query_params.clone().into_inner();

    let nickname = uniqueness_user_dto.nickname.unwrap_or("".to_string());
    let email = uniqueness_user_dto.email.unwrap_or("".to_string());

    let is_nickname = nickname.len() > 0;
    let is_email = email.len() > 0;
    if !is_nickname && !is_email {
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        #[rustfmt::skip]
        log::error!("{}: {}: {}", err::CD_NOT_ACCEPTABLE, MSG_PARAMETERS_NOT_SPECIFIED, json.to_string());
        return Err(AppError::not_acceptable406(MSG_PARAMETERS_NOT_SPECIFIED) // 406
            .add_param(Borrowed("invalidParams"), &json));
    }

    #[rustfmt::skip]
    let opt_nickname = if nickname.len() > 0 { Some(nickname) } else { None };
    let opt_email = if email.len() > 0 { Some(email) } else { None };

    let opt_nickname2 = opt_nickname.clone();
    let opt_email2 = opt_email.clone();

    // Find in the "profile" table an entry by nickname or email.
    let opt_profile = web::block(move || {
        let existing_profile = profile_orm
            .find_profile_by_nickname_or_email(opt_nickname2.as_deref(), opt_email2.as_deref(), false)
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            })
            .ok()?;
        existing_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })?;

    let mut uniqueness = false;
    // If such an entry exists, then exit.
    if opt_profile.is_none() {
        let opt_nickname2 = opt_nickname.clone();
        let opt_email2 = opt_email.clone();

        // Find in the "user_registr" table an entry with an active date, by nickname or email.
        let opt_user_registr = web::block(move || {
            let existing_user_registr = user_registr_orm
                .find_user_registr_by_nickname_or_email(opt_nickname2.as_deref(), opt_email2.as_deref())
                .map_err(|e| {
                    log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e) // 507
                })
                .ok()?;
            existing_user_registr
        })
        .await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })?;

        uniqueness = opt_user_registr.is_none();
    }

    Ok(HttpResponse::Ok().json(json!({ "uniqueness": uniqueness }))) // 200
}

/// get_profile_by_id
///
/// Search for a user profile by its ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles/1
/// ```
///
/// Return the found specified user (`ProfileDto`) with status 200 or 204 (no content) if the user is not found.
/// 
/// The "admin" role is required.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "A user with the specified ID was found.", body = ProfileDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                        Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK)))
            )))),
        ),
        (status = 204, description = "The user with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X GET http://localhost:8080/api/users/2a`", 
            body = AppError, example = json!(AppError::range_not_satisfiable416(
                &format!("{}: {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/profiles/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())" )]
pub async fn get_profile_by_id(
    profile_orm: web::Data<ProfileOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;

    let opt_profile = web::block(move || {
        // Find profile by user id.
        let profile =
            profile_orm.get_profile_user_by_id(id, false).map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            }).ok()?;

        profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })?;

    if let Some(profile_user) = opt_profile {
        let profile_dto = ProfileDto::from(profile_user);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// get_profile_current
/// 
/// Get information about the current user's profile (`ProfileDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles_current
/// ```
///
/// Return the current user's profile (`ProfileDto`) with status 200.
///
/// The "theme" parameter takes values:
/// - "light" light theme;
/// - "dark" dark theme;
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Profile information about the current user.", body = ProfileDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                        Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                    Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK)))
            )))),
        ),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[get("/api/profiles_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_profile_current(
    authenticated: Authenticated,
) -> actix_web::Result<HttpResponse, AppError> {
    let profile = authenticated.deref();
    let profile_dto = ProfileDto::from(profile.clone());
    Ok(HttpResponse::Ok().json(profile_dto)) // 200
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
    use chrono::{DateTime, Duration, Utc};

    use crate::extractors::authentication::BEARER;
    use crate::profiles::config_prfl;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::users::user_models::{UserRegistr, UserRole};

    use super::*;

    const ADMIN: u8 = 0;
    const USER: u8 = 1;
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_profile(role: u8) -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = if role == ADMIN { UserRole::Admin } else { UserRole::User };
        let profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);
        profile
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_user_registr() -> UserRegistr {
        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);

        let user_registr =
            UserRegistrOrmApp::new_user_registr(1, "Robert_Brown", "Robert_Brown@gmail.com", "passwdR2B2", final_date);
        user_registr
    }
    fn user_registr_with_id(user_registr: UserRegistr) -> UserRegistr {
        let user_reg_orm = UserRegistrOrmApp::create(&vec![user_registr]);
        user_reg_orm.user_registr_vec.get(0).unwrap().clone()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data(is_registr: bool, role: u8) -> ((config_jwt::ConfigJwt, config_prfl::ConfigPrfl), (Vec<Profile>, Vec<Session>, Vec<UserRegistr>), String) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile(role));
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(profile1.user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(profile1.user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let user_registr_vec:Vec<UserRegistr> = if is_registr {
            vec![user_registr_with_id(create_user_registr())]
        } else { vec![] };

        let config_prfl = config_prfl::get_test_config();
        let cfg_c = (config_jwt, config_prfl);
        let data_c = (vec![profile1], vec![session1], user_registr_vec);

        (cfg_c, data_c, token)
    }
    fn configure_profile(
        cfg_c: (config_jwt::ConfigJwt, config_prfl::ConfigPrfl), // configuration
        data_c: (Vec<Profile>, Vec<Session>, Vec<UserRegistr>),  // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(cfg_c.0);
            let data_config_prfl = web::Data::new(cfg_c.1);

            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_prfl))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm));
        }
    }

    // ** uniqueness_check **
    #[actix_web::test]
    async fn test_uniqueness_check_non_params() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles/uniqueness")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_nickname_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles/uniqueness?nickname=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_email_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles/uniqueness?email=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, MSG_PARAMETERS_NOT_SPECIFIED);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_non_existent_nickname() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let nickname = format!("a{}", data_c.0.get(0).unwrap().nickname.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":true}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_non_existent_email() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let email = format!("a{}", data_c.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":true}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_existent_nickname() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let nickname = data_c.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_existent_email() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let email = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_existent_registr_nickname() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, USER);
        let nickname = data_c.2.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?nickname={}", nickname))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }
    #[actix_web::test]
    async fn test_uniqueness_check_existent_registr_email99() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, USER);
        let email = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":false}");
    }

    // ** get_profile_by_id **
    #[actix_web::test]
    async fn test_get_profile_by_id_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let user_id = data_c.0.get(0).unwrap().user_id;
        let user_id_bad = format!("{}a", user_id);
        #[rustfmt::skip]
            let app = test::init_service(
                App::new().service(get_profile_by_id).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
            let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", user_id_bad))
                .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
            assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_RANGE_NOT_SATISFIABLE);
        #[rustfmt::skip]
            let msg = format!("{}: `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, user_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_get_profile_by_id_valid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile2 = ProfileOrmApp::new_profile(2, "Logan_Lewis", "Logan_Lewis@gmail.com", UserRole::User);

        let profile_vec = ProfileOrmApp::create(&vec![profile1, profile2]).profile_vec;
        let profile2_dto = ProfileDto::from(profile_vec.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;

        let data_c = (profile_vec, data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", &profile2_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile2_dto).to_string();
        let profile2b_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile2b_dto_ser);
    }
    #[actix_web::test]
    async fn test_get_profile_by_id_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile2 = ProfileOrmApp::new_profile(2, "Logan_Lewis", "Logan_Lewis@gmail.com", UserRole::User);

        let profile_vec = ProfileOrmApp::create(&vec![profile1, profile2]).profile_vec;
        let profile2_dto = ProfileDto::from(profile_vec.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;

        let data_c = (profile_vec, data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_by_id).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles/{}", profile2_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }

    // ** get_profile_current **
    #[actix_web::test]
    async fn test_get_profile_current_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
    }
}

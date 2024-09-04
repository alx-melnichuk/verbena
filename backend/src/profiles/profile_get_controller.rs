use std::{borrow::Cow, ops::Deref};

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
    config_prfl::ConfigPrfl,
    profile_checks,
    profile_models::{
        Profile, ProfileConfigDto, ProfileDto, UniquenessProfileDto, UniquenessProfileResponseDto, PROFILE_THEME_DARK,
        PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
use crate::settings::err;
use crate::users::user_models::UserRole;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::utils::parser;

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // GET /api/profiles/{id}
            .service(get_profile_by_id)
            // GET /api/profiles_config
            .service(get_profile_config)
            // GET /api/profiles_current
            .service(get_profile_current)
            // GET /api/profiles_uniqueness
            .service(uniqueness_check);
    }
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

/// profiles_config
///
/// Get information about the image configuration settings in the user profile (`ProfileConfigDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles_config
/// ```
///
/// Returns the configuration settings for the user's profile image (`ProfileConfigDto`) with status 200.
///
/// The structure is returned:
/// ```text
/// {
///   avatar_max_size?: Number,      // optional - Maximum size for avatar files;
///   avatar_valid_types: String[],  //          - List of valid input mime types for avatar files;
/// //                                           ["image/bmp", "image/gif", "image/jpeg", "image/png"]
///   avatar_ext?: String,           // optional - Avatar files will be converted to this MIME type;
/// //                                  Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
///   avatar_max_width?: Number,     // optional - Maximum width of avatar image after saving;
///   avatar_max_height?: Number,    // optional - Maximum height of avatar image after saving;
/// }
/// ```
///
#[utoipa::path(
    responses(
        (status = 200, description = "Get information about the image configuration settings in the user profile",
            body = ProfileConfigDto,
            examples(
            ("max_config" = (summary = "maximum configuration", description = "Maximum configuration for user image.",
                value = json!(ProfileConfigDto::new(
                    Some(2*1024*1024), ConfigPrfl::image_types(), Some(ConfigPrfl::image_types()[0].clone()), Some(512), Some(512)))
            )),
            ("min_config" = (summary = "minimum configuration", description = "Minimum configuration for user image.",
                value = json!(ProfileConfigDto::new(None, ConfigPrfl::image_types(), None, None, None))
            )), ),
        ),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/profiles_config", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_profile_config(config_prfl: web::Data<ConfigPrfl>) -> actix_web::Result<HttpResponse, AppError> {
    let cfg_prfl = config_prfl;
    let max_size = if cfg_prfl.prfl_avatar_max_size > 0 { Some(cfg_prfl.prfl_avatar_max_size) } else { None };
    let valid_types = cfg_prfl.prfl_avatar_valid_types.clone();
    let ext = cfg_prfl.prfl_avatar_ext.clone();
    let max_width = if cfg_prfl.prfl_avatar_max_width > 0 { Some(cfg_prfl.prfl_avatar_max_width) } else { None };
    let max_height = if cfg_prfl.prfl_avatar_max_height > 0 { Some(cfg_prfl.prfl_avatar_max_height) } else { None };
    // Get configuration data.
    let profile_config_dto = ProfileConfigDto::new(max_size, valid_types, ext, max_width, max_height);

    Ok(HttpResponse::Ok().json(profile_config_dto)) // 200
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

/// uniqueness_check
///
/// Checking the uniqueness of the user's "nickname" or "email".
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles_uniqueness?nickname=demo1
/// ```
/// Or you could call with the next curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles_uniqueness?email=demo1@gmail.us
/// ```
///
/// If the value is already in use, then `{"uniqueness":false}` is returned with status 200.
/// If the value is not yet used, then `{"uniqueness":true}` is returned with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The result of checking whether nickname (email) is already in use.", 
            body = UniquenessProfileResponseDto, 
            examples(
            ("already_use" = (summary = "already in use",  description = "If the nickname (email) is already in use.",
                value = json!(UniquenessProfileResponseDto::new(false)))),
            ("not_use" = (summary = "not yet in use", description = "If the nickname (email) is not yet used.",
                value = json!(UniquenessProfileResponseDto::new(true))))
        )),
        (status = 406, description = "None of the parameters are specified.", body = AppError,
            example = json!(AppError::not_acceptable406(err::MSG_PARAMS_NOT_SPECIFIED)
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "nickname": "null", "email": "null" })))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
)]
#[get("/api/profiles_uniqueness")]
pub async fn uniqueness_check(
    profile_orm: web::Data<ProfileOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    query_params: web::Query<UniquenessProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get search parameters.
    let uniqueness_user_dto: UniquenessProfileDto = query_params.clone().into_inner();

    let opt_nickname = uniqueness_user_dto.nickname.clone();
    let opt_email = uniqueness_user_dto.email.clone();

    let profile_orm = profile_orm.get_ref().clone();
    let registr_orm = user_registr_orm.get_ref().clone();

    let res_search = profile_checks::uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, registr_orm)
        .await
        .map_err(|err| {
            #[rustfmt::skip]
            let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
            log::error!("{}:{}; {}", &err.code, &err.message, &prm1);
            err
        })?;
    let uniqueness = res_search.is_none();

    let response_dto = UniquenessProfileResponseDto::new(uniqueness);

    Ok(HttpResponse::Ok().json(response_dto)) // 200
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev,
        http::{self, header::{HeaderValue, CONTENT_TYPE}, StatusCode},
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
        let json = json!(profile2_dto).to_string();
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

    // ** get_profile_config **
    #[actix_web::test]
    async fn test_get_profile_config_data() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let cfg_prfl = cfg_c.1.clone();
        #[rustfmt::skip]
        let profile_config_dto = ProfileConfigDto::new(
            if cfg_prfl.prfl_avatar_max_size > 0 { Some(cfg_prfl.prfl_avatar_max_size) } else { None },
            cfg_prfl.prfl_avatar_valid_types.clone(),
            cfg_prfl.prfl_avatar_ext.clone(),
            if cfg_prfl.prfl_avatar_max_width > 0 { Some(cfg_prfl.prfl_avatar_max_width) } else { None },
            if cfg_prfl.prfl_avatar_max_height > 0 { Some(cfg_prfl.prfl_avatar_max_height) } else { None },
        );
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_config).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_config")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_config_dto_res: ProfileConfigDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(profile_config_dto_res, profile_config_dto);
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
        let json = json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
    }

    // ** uniqueness_check **
    #[actix_web::test]
    async fn test_uniqueness_check_by_non_params() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_nickname_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness?nickname=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_email_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_uniqueness?email=")
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_check_by_nickname_profile() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let nickname = data_c.0.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
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
    async fn test_uniqueness_check_by_email_profile() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let email = data_c.0.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
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
    async fn test_uniqueness_check_by_nickname_registr() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, USER);
        let nickname = data_c.2.get(0).unwrap().nickname.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
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
    async fn test_uniqueness_check_by_email_registr() {
        let (cfg_c, data_c, _token) = get_cfg_data(true, USER);
        let email = data_c.2.get(0).unwrap().email.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
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
    async fn test_uniqueness_check_by_new_nickname() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let nickname = format!("a{}", data_c.0.get(0).unwrap().nickname.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?nickname={}", nickname))
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
    async fn test_uniqueness_check_by_new_email() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, USER);
        let email = format!("a{}", data_c.0.get(0).unwrap().email.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(uniqueness_check).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&format!("/api/profiles_uniqueness?email={}", email))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let nickname_res = std::str::from_utf8(&body).unwrap();
        assert_eq!(nickname_res, "{\"uniqueness\":true}");
    }

}

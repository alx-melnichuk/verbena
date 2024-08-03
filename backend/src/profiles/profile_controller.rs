use std::ops::Deref;

use actix_web::{get, web, HttpResponse};
use log;
use utoipa;

use crate::{
    errors::AppError,
    extractors::authentication::{Authenticated, RequireAuth},
    profiles::{
        profile_models::{
            ProfileUser, ProfileUserDto, PROFILE_DESCRIPT_DEF, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF,
        },
        profile_orm::ProfileOrm,
    },
    settings::err,
    users::user_models::UserRole,
};

#[cfg(not(feature = "mockdata"))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(feature = "mockdata")]
use crate::profiles::profile_orm::tests::ProfileOrmApp;

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        // GET /api/profiles_current
        config.service(get_profile_current);
    }
}

/// get_profile_current
/// Get information about the current user's profile (`ProfileUserDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/profiles_current
/// ```
///
/// Return the current user (`ProfileUserDto`) with status 200 or 204 (no content) if the user is not found.
///
/// The "theme" parameter takes values:
/// - "light" light theme;
/// - "dark" dark theme;
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Profile information about the current user.", body = ProfileUserDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileUserDto::from(
                    ProfileUser::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                        "Description Emma_Johnson", PROFILE_THEME_LIGHT_DEF))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileUserDto::from(
                    ProfileUser::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, PROFILE_DESCRIPT_DEF,
                        PROFILE_THEME_DARK))
            )))),
        ),
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
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[get("/api/profiles_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_profile_current(
    authenticated: Authenticated,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let profile_user0 = authenticated.deref();
    let user_id = profile_user0.user_id;

    let opt_profile_user = web::block(move || {
        // Find profile by user id.
        let profile_user =
            profile_orm.get_profile_by_user_id(user_id).map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            }).ok()?;

        profile_user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })?;

    if let Some(profile_user) = opt_profile_user {
        let profile_user_dto = ProfileUserDto::from(profile_user);
        Ok(HttpResponse::Ok().json(profile_user_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };

    use crate::{
        extractors::authentication::BEARER,
        hash_tools,
        profiles::profile_models::{PROFILE_DESCRIPT_DEF, PROFILE_THEME_LIGHT_DEF},
        sessions::{config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token},
        users::{
            user_models::{User, UserDto, UserRole},
            user_orm::tests::UserOrmApp,
        },
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user(is_hash_password: bool) -> User {
        let nickname = "Oliver_Taylor".to_string();
        let mut password: String = "passwdT1R1".to_string();
        if is_hash_password {
            password = hash_tools::encode_hash(password).unwrap(); // hashed
        }
        let mut user = UserOrmApp::new_user(1, &nickname, &format!("{}@gmail.com", &nickname), &password);
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_profile(user: User) -> ProfileUser {
        ProfileUser::new(
            user.id,
            &user.nickname,
            &user.email,
            user.role.clone(),
            None,
            PROFILE_DESCRIPT_DEF,
            PROFILE_THEME_LIGHT_DEF,
        )
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data() -> (config_jwt::ConfigJwt, (Vec<User>, Vec<ProfileUser>, Vec<Session>), String) {
        let user1: User = user_with_id(create_user(true));
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // Create profile values.
        let profile1 = create_profile(user1.clone());

        let data_c = (vec![user1], vec![profile1], vec![session1]);

        (config_jwt, data_c, token)
    }

    fn configure_profile(
        config_jwt: config_jwt::ConfigJwt,                   // configuration
        data_c: (Vec<User>, Vec<ProfileUser>, Vec<Session>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.1));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_profile_orm));
        }
    }

    // ** get_profile_current **
    #[actix_web::test]
    async fn test_get_profile_current_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();
        let user1_dto = UserDto::from(user1);
        let profile1 = data_c.1.get(0).unwrap().clone();
        let profile1_dto = ProfileUserDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileUserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileUserDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
        assert_eq!(profile_dto_res.nickname, user1_dto.nickname);
        assert_eq!(profile_dto_res.email, user1_dto.email);
    }
}

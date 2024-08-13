use actix_web::{get, put, web, HttpResponse};
use utoipa;

use crate::hash_tools;
// use crate::profiles::profile_models::ProfileDto;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    // profile_models::{/*CreateProfile,*/ CreateProfileRecoveryDto},
    profile_orm::ProfileOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::send_email::mailer::impls::MailerApp;
#[cfg(feature = "mockdata")]
use crate::send_email::mailer::tests::MailerApp;
// use crate::send_email::mailer::Mailer;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{
    config_jwt,
    session_orm::SessionOrm,
    tokens::{decode_token, encode_token /*generate_num_token*/},
};
use crate::settings::{/*config_app,*/ err};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_orm::impls::UserOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_recovery_orm::impls::UserRecoveryOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_recovery_orm::tests::UserRecoveryOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::{
    user_err as u_err,
    user_models::{
        self,
        ClearForExpiredResponseDto,
        RecoveryDataDto,
        // RecoveryUserDto,
        // RecoveryUserResponseDto,
        //RegistrUserDto,
        //RegistrUserResponseDto,
        UserDto,
    },
    user_orm::UserOrm,
    user_recovery_orm::UserRecoveryOrm,
    user_registr_orm::UserRegistrOrm,
};
use crate::validators::{msg_validation, Validator};
use crate::{errors::AppError, extractors::authentication::RequireAuth};

// pub const MSG_EMAIL_ALREADY_USE: &str = "email_already_use";
// pub const MSG_NICKNAME_ALREADY_USE: &str = "nickname_already_use";
// 510 Not Extended - Error when sending email.
pub const MSG_ERROR_SENDING_EMAIL: &str = "error_sending_email";
// 404 Not Found - Registration record not found.
// pub const MSG_REGISTR_NOT_FOUND: &str = "registration_not_found";
// 404 Not Found - Recovery record not found.
pub const MSG_RECOVERY_NOT_FOUND: &str = "recovery_not_found";
// 404 Not Found - User not found.
pub const MSG_USER_NOT_FOUND: &str = "user_not_found";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // PUT /api/recovery/{recovery_token}
            .service(confirm_recovery);
    }
}

#[put("/api/recovery/{recovery_token}")]
pub async fn confirm_recovery(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    user_orm: web::Data<UserOrmApp>,
    profile_orm: web::Data<ProfileOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<RecoveryDataDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors)));
        // 417
    }

    let recovery_data_dto: RecoveryDataDto = json_body.into_inner();

    // Prepare a password hash.
    let password_hashed = hash_tools::encode_hash(&recovery_data_dto.password).map_err(|e| {
        let message = format!("{}: {}", err::MSG_ERROR_HASHING_PASSWORD, e.to_string());
        log::error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
        AppError::internal_err500(&message) // 500
    })?;

    let recovery_token = request.match_info().query("recovery_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “recovery_token".
    let dual_token = decode_token(&recovery_token, jwt_secret).map_err(|e| {
        let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
        log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
        AppError::unauthorized401(&message) // 401
    })?;

    // Get "user_recovery ID" from "recovery_token".
    let (user_recovery_id, _) = dual_token;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Find a record with the specified ID in the “user_recovery" table.
    let opt_user_recovery = web::block(move || {
        let user_recovery = user_recovery_orm2.get_user_recovery_by_id(user_recovery_id).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        user_recovery
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    // Delete entries in the "user_recovery" table, that are already expired.
    let _ = web::block(move || user_recovery_orm2.delete_inactive_final_date(None)).await;

    // If no such entry exists, then exit with code 404.
    let user_recovery = opt_user_recovery.ok_or_else(|| {
        let message = format!("{}: user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, user_recovery_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;
    let user_id = user_recovery.user_id;

    // If there is "user_recovery" with this ID, then move on to the next step.
    let opt_profile = web::block(move || {
        // Find profile by user id.
        let res_profile = profile_orm.get_profile_user_by_id(user_id, false).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });

        res_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })??;

    // If no such entry exists, then exit with code 404.
    let profile = opt_profile.ok_or_else(|| {
        let message = format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;

    let user_orm2 = user_orm.clone();
    let modify_user_dto = user_models::ModifyUserDto {
        nickname: None,
        email: None,
        password: Some(password_hashed),
        role: None,
    };
    // Update the password hash for the entry in the "user" table.
    let opt_user = web::block(move || {
        let user = user_orm2.modify_user(profile.user_id, modify_user_dto).map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        user
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    let _ = web::block(move || {
        // Delete entries in the “user_recovery" table.
        let user_recovery_res = user_recovery_orm2.delete_user_recovery(user_recovery_id);

        // Clear the user session in the "session" table.
        let session_res = session_orm.modify_session(user_id, None);

        (user_recovery_res, session_res)
    })
    .await;

    let user = opt_user.ok_or_else(|| {
        let message = format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id);
        log::error!("{}: {}", err::CD_NOT_FOUND, &message);
        AppError::not_found404(&message) // 404
    })?;
    let user_dto = UserDto::from(user);

    Ok(HttpResponse::Ok().json(user_dto)) // 200
}

/// clear_for_expire
///
/// Clean up expired user registration and password recovery requests.
///
/// One could call with following curl.
/// ```text
/// curl -i -X GET http://localhost:8080/api/clear_for_expired
/// ```
///
/// Returns the number of expired records deleted (`ClearForExpiredResponseDto`) with status 200.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The number of deleted user registration and expired password recovery records.",
            body = ClearForExpiredResponseDto),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/clear_for_expired", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn clear_for_expired(
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Delete entries in the "user_registr" table, that are already expired.
    let count_inactive_registr_res = 
        web::block(move || user_registr_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
        ).await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })?;

    let count_inactive_registr = count_inactive_registr_res.unwrap_or(0);

    // Delete entries in the "user_recovery" table, that are already expired.
    let count_inactive_recover_res = 
        web::block(move || user_recovery_orm.delete_inactive_final_date(None)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        })
        ).await
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
            AppError::blocking506(&e.to_string()) // 506
        })?;

    let count_inactive_recover = count_inactive_recover_res.unwrap_or(0);

    let clear_for_expired_response_dto = ClearForExpiredResponseDto {
        count_inactive_registr,
        count_inactive_recover,
    };
    
    Ok(HttpResponse::Ok().json(clear_for_expired_response_dto)) // 200
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        body, dev, http,
        http::header::{HeaderValue, CONTENT_TYPE},
        test, web, App,
    };
    use chrono::{DateTime, Duration, Utc};
    // use serde_json::json;

    use crate::errors::AppError;
    use crate::extractors::authentication::BEARER;
    use crate::profiles::profile_models::{self, Profile};
    use crate::send_email::config_smtp;
    use crate::sessions::{config_jwt, session_models::Session, tokens::decode_token};
    use crate::settings::{config_app, err};
    use crate::users::{
        user_models::{
            RecoveryDataDto, RecoveryUserDto, RecoveryUserResponseDto, User, UserDto, UserModelsTest, UserRecovery,
            UserRegistr, UserRole,
        },
        user_orm::tests::UserOrmApp,
        user_registr_orm::tests::UserRegistrOrmApp,
    };

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
    fn create_profile(user: User) -> Profile {
        Profile::new(
            user.id,
            &user.nickname,
            &user.email,
            user.role.clone(),
            None,
            None,
            None,
        )
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
    fn create_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
        UserRecoveryOrmApp::new_user_recovery(id, user_id, final_date)
    }
    fn create_user_recovery_with_id(user_recovery: UserRecovery) -> UserRecovery {
        let user_recovery_orm = UserRecoveryOrmApp::create(&vec![user_recovery]);
        user_recovery_orm.user_recovery_vec.get(0).unwrap().clone()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    fn get_cfg_data(is_registr: bool, opt_recovery_duration: Option<i64>) -> (
        (config_app::ConfigApp, config_jwt::ConfigJwt), 
        (Vec<User>, Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<UserRecovery>),
        String) {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // Create profile values.
        let profile1 = create_profile(user1.clone());

        let config_app = config_app::get_test_config();

        let user_registr_vec:Vec<UserRegistr> = if is_registr {
            vec![user_registr_with_id(create_user_registr())]
        } else { vec![] };

        let user_recovery_vec:Vec<UserRecovery> = if let Some(recovery_duration) = opt_recovery_duration {
            let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
            let user_recovery = UserRecoveryOrmApp::new_user_recovery(1, user1.id, final_date_utc);
            let user_recovery_orm = UserRecoveryOrmApp::create(&vec![user_recovery]);
            let user_recovery1 = user_recovery_orm.user_recovery_vec.get(0).unwrap().clone();
            vec![user_recovery1]
        } else { vec![] };
        
        let cfg_c = (config_app, config_jwt);
        let data_c = (vec![user1], vec![profile1], vec![session1], user_registr_vec,  user_recovery_vec);

        (cfg_c, data_c, token)
    }
    fn configure_user(
        cfg_c: (config_app::ConfigApp, config_jwt::ConfigJwt), // cortege of configurations
        data_c: (
            Vec<User>,
            Vec<Profile>,
            Vec<Session>,
            Vec<UserRegistr>,
            Vec<UserRecovery>,
        ), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_app = web::Data::new(cfg_c.0);
            let data_config_jwt = web::Data::new(cfg_c.1);
            let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));

            let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.1));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.2));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.3));
            let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(&data_c.4));

            config
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm));
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

    // ** confirm recovery **
    #[actix_web::test]
    async fn test_confirm_recovery_invalid_dto_password_empty() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: "".to_string() })
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
    async fn test_confirm_recovery_invalid_dto_password_min() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: UserModelsTest::password_min() })
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
    async fn test_confirm_recovery_invalid_dto_password_max() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "recovery_token"))
            .set_json(RecoveryDataDto { password: UserModelsTest::password_max() })
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
    async fn test_confirm_recovery_invalid_recovery_token() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", "invalid_recovery_token"))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_final_date_has_expired() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.4.get(0).unwrap().clone();

        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, -recovery_duration).unwrap();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(
            app_err.message,
            format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, "ExpiredSignature")
        );
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user_recovery() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user_recovery1 = data_c.4.get(0).unwrap().clone();
        let user_recovery_id = user_recovery1.id + 1;
        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let recovery_token = encode_token(user_recovery_id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(
            app_err.message,
            format!("{}: user_recovery_id: {}", MSG_RECOVERY_NOT_FOUND, user_recovery_id)
        );
    }
    #[actix_web::test]
    async fn test_confirm_recovery_no_exists_in_user() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, None);
        let user1 = data_c.0.get(0).unwrap().clone();

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::seconds(recovery_duration);
        let user_id = user1.id + 1;
        let user_recovery1 = create_user_recovery_with_id(create_user_recovery(0, user_id, final_date_utc));
        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();

        let data_c = (data_c.0, data_c.1, data_c.2, data_c.3, vec![user_recovery1]);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, format!("{}: user_id: {}", MSG_USER_NOT_FOUND, user_id));
    }
    #[actix_web::test]
    async fn test_confirm_recovery_success() {
        let (cfg_c, data_c, _token) = get_cfg_data(false, Some(600));
        let user1b = data_c.0.get(0).unwrap().clone();
        let user_recovery1 = data_c.4.get(0).unwrap().clone();
        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();

        let num_token = 1234;
        let jwt_secret: &[u8] = cfg_c.1.jwt_secret.as_bytes();
        let recovery_token = encode_token(user_recovery1.id, num_token, jwt_secret, recovery_duration).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(confirm_recovery).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/recovery/{}", recovery_token))
            .set_json(RecoveryDataDto { password: "passwordQ2V2".to_string() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(user_dto_res.id, user1b.id);
        assert_eq!(user_dto_res.nickname, user1b.nickname);
        assert_eq!(user_dto_res.email, user1b.email);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1b.role);
    }

    // ** clear_for_expired **
    #[actix_web::test]
    async fn test_clear_for_expired_user_recovery() {
        let (cfg_c, data_c, token) = get_cfg_data(true, Some(600));
        let mut user1 = data_c.0.get(0).unwrap().clone();
        user1.role = UserRole::Admin;
        let profile1 = create_profile(user1.clone());

        let registr_duration: i64 = cfg_c.0.app_registr_duration.try_into().unwrap();
        let final_date_registr = Utc::now() - Duration::seconds(registr_duration);
        let mut user_registr1 = data_c.3.get(0).unwrap().clone();
        user_registr1.final_date = final_date_registr;

        let recovery_duration: i64 = cfg_c.0.app_recovery_duration.try_into().unwrap();
        let final_date_recovery = Utc::now() - Duration::seconds(recovery_duration);
        let mut user_recovery1 = data_c.4.get(0).unwrap().clone();
        user_recovery1.final_date = final_date_recovery;

        let data_c = (
            vec![user1],
            vec![profile1],
            data_c.2,
            vec![user_registr1],
            vec![user_recovery1],
        );
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(clear_for_expired).configure(configure_user(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get().uri(&"/api/clear_for_expired")
            .insert_header(header_auth(&token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response_dto: user_models::ClearForExpiredResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(response_dto.count_inactive_registr, 1);
        assert_eq!(response_dto.count_inactive_recover, 1);
    }
}

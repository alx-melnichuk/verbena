use actix_web::{post, put, web, HttpResponse};
use chrono::{Duration, Utc};
use validator::{Validate, ValidationErrors};

#[cfg(not(feature = "mockdata"))]
use crate::email::mailer::inst::MailerApp;
#[cfg(feature = "mockdata")]
use crate::email::mailer::tests::MailerApp;
use crate::email::mailer::Mailer;
use crate::errors::{AppError, CN_VALIDATION};
use crate::hash_tools;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{
    config_jwt,
    session_models::Session,
    session_orm::SessionOrm,
    tokens::{decode_dual_token, encode_dual_token, generate_num_token},
};
use crate::users::user_models::CreateUserRecoveryDto;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_recovery_orm::inst::UserRecoveryOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_recovery_orm::tests::UserRecoveryOrmApp;
use crate::users::{
    user_models, user_orm::UserOrm, user_recovery_orm::UserRecoveryOrm,
    user_registr_models::CreateUserRegistrDto, user_registr_orm::UserRegistrOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::users::{user_orm::inst::UserOrmApp, user_registr_orm::inst::UserRegistrOrmApp};
#[cfg(feature = "mockdata")]
use crate::users::{user_orm::tests::UserOrmApp, user_registr_orm::tests::UserRegistrOrmApp};
use crate::utils::{
    config_app,
    err::{
        CD_BLOCKING, CD_DATABASE, CD_HASHING_PASSWD, CD_JSONWEBTOKEN, CD_NOT_FOUND, CD_NO_CONFIRM,
        MSG_CONFIRM_NOT_FOUND, MSG_NOT_FOUND_BY_EMAIL,
    },
};

pub const CD_WRONG_EMAIL: &str = "WrongEmail";
pub const MSG_WRONG_EMAIL: &str = "The specified email is incorrect!";

pub const CD_WRONG_NICKNAME: &str = "WrongNickname";
pub const MSG_WRONG_NICKNAME: &str = "The specified nickname is incorrect!";

pub const CD_ERROR_SENDING_EMAIL: &str = "ErrorSendingEmail";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/registration
    cfg.service(registration)
        // PUT api/registration/{registr_token}
        .service(confirm_registration)
        // POST api/recovery
        .service(recovery)
        // PUT api/recovery/{recovery_token}
        .service(confirm_recovery);
}

fn err_database(err: String) -> AppError {
    log::error!("{CD_DATABASE}: {}", err);
    AppError::new(CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{CD_BLOCKING}: {}", err);
    AppError::new(CD_BLOCKING, &err).set_status(500)
}
fn err_wrong_email_or_nickname(is_nickname: bool) -> AppError {
    let val = if is_nickname {
        (CD_WRONG_NICKNAME, MSG_WRONG_NICKNAME)
    } else {
        (CD_WRONG_EMAIL, MSG_WRONG_EMAIL)
    };
    log::error!("{}: {}", val.0, val.1);
    AppError::new(val.0, val.1).set_status(409)
}
fn err_validate(error: ValidationErrors) -> AppError {
    log::error!("{CN_VALIDATION}: {}", error.to_string());
    AppError::from(error)
}
fn err_json_web_token(err: String) -> AppError {
    log::error!("{CD_JSONWEBTOKEN}: {}", err);
    AppError::new(CD_JSONWEBTOKEN, &err).set_status(500)
}
fn err_sending_email(err: String) -> AppError {
    log::error!("{CD_ERROR_SENDING_EMAIL}: {err}");
    AppError::new(CD_ERROR_SENDING_EMAIL, &err).set_status(500)
}
fn err_encode_hash(err: String) -> AppError {
    log::error!("{CD_HASHING_PASSWD}: {err}");
    AppError::new(CD_HASHING_PASSWD, &err).set_status(500)
}
fn err_not_found() -> AppError {
    log::error!("{CD_NO_CONFIRM}: {MSG_CONFIRM_NOT_FOUND}");
    AppError::new(CD_NO_CONFIRM, MSG_CONFIRM_NOT_FOUND).set_status(404)
}

// Send a confirmation email to register the user.
// POST api/registration
#[post("/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_body: web::Json<user_models::RegistrUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_body.validate().map_err(|error| err_validate(error))?;

    let mut registr_user_dto: user_models::RegistrUserDto = json_body.0.clone();
    registr_user_dto.nickname = registr_user_dto.nickname.to_lowercase();
    registr_user_dto.email = registr_user_dto.email.to_lowercase();

    let password = registr_user_dto.password.clone();
    let password_hashed = hash_tools::encode_hash(&password).map_err(|err| err_encode_hash(err))?;

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();

    // Find in the "user" table an entry by nickname or email.
    let user_opt = web::block(move || {
        let existing_user = user_orm
            .find_user_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()));
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let nickname = registr_user_dto.nickname.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user) = user_opt {
        return Err(err_wrong_email_or_nickname(user.nickname == nickname));
    }

    let email = registr_user_dto.email.clone();
    let user_registr_orm2 = user_registr_orm.clone();

    // Find in the "user_registr" table an entry with an active date, by nickname or email.
    let user_registr_opt = web::block(move || {
        let existing_user_registr = user_registr_orm2
            .find_user_registr_by_nickname_or_email(&nickname, &email)
            .map_err(|e| err_database(e.to_string()));
        existing_user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let nickname = registr_user_dto.nickname.clone();

    // If such an entry exists, then exit with code 409.
    if let Some(user_registr) = user_registr_opt {
        let is_match_nickname = user_registr.nickname == nickname;
        return Err(err_wrong_email_or_nickname(is_match_nickname));
    }

    // If there is no such record, then add the specified data to the "user_registr" table.

    let app_registr_duration: i64 = config_app.app_registr_duration.try_into().unwrap();
    let final_date_utc = Utc::now() + Duration::minutes(app_registr_duration.into());

    let create_user_registr_dto = CreateUserRegistrDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        password: password_hashed,
        final_date: final_date_utc,
    };
    // Create a new entity (user).
    let user_registr = web::block(move || {
        let user_registr = user_registr_orm
            .create_user_registr(&create_user_registr_dto)
            .map_err(|e| err_database(e));
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_registr.id, num_token) into a registr_token.
    let registr_token =
        encode_dual_token(user_registr.id, num_token, jwt_secret, app_registr_duration)
            .map_err(|err| err_json_web_token(err))?;

    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let nickname = registr_user_dto.nickname.clone();
    let receiver = registr_user_dto.email.clone();

    let result = mailer.send_verification_code(&receiver, &domain, &nickname, &registr_token);

    if result.is_err() {
        return Err(err_sending_email(result.unwrap_err()));
    }

    let registr_user_response_dto = user_models::RegistrUserResponseDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        registr_token: registr_token.clone(),
    };

    Ok(HttpResponse::Created().json(registr_user_response_dto))
}

// Confirm user registration.
// PUT api//registration/{registr_token}
#[put("/registration/{registr_token}")]
pub async fn confirm_registration(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let registr_token = request.match_info().query("registr_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “registr_token".
    let dual_token = decode_dual_token(&registr_token, jwt_secret).map_err(|err| {
        log::error!("{}: {}", err.code, err.message);
        err // 403
    })?;

    let user_registr_orm2 = user_registr_orm.clone();

    // Get "user_registr ID" from "registr_token".
    let (user_registr_id, _) = dual_token;

    // Find a record with the specified ID in the “user_registr" table.
    let user_registr_opt = web::block(move || {
        let user_registr = user_registr_orm2
            .find_user_registr_by_id(user_registr_id)
            .map_err(|e| err_database(e));
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user_registr = user_registr_opt.ok_or_else(|| err_not_found())?;

    let user_registr_orm2 = user_registr_orm.clone();
    // If such an entry exists, then add a new user.
    let create_user_dto = user_models::CreateUserDto {
        nickname: user_registr.nickname,
        email: user_registr.email,
        password: user_registr.password,
    };

    let user = web::block(move || {
        // Create a new entity (user).
        let user_res =
            user_orm.create_user(&create_user_dto).map_err(|e| err_database(e.to_string()));
        user_res
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let session = Session {
        user_id: user.id,
        num_token: None,
    };
    let _ = web::block(move || {
        // Create a new entity (session).
        let session_res =
            session_orm.create_session(&session).map_err(|e| err_database(e.to_string()));

        user_registr_orm2.delete_user_registr(user_registr_id).ok();

        session_res
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    // Delete the registration record for this user in the “user_registr" table.
    let _ = web::block(move || user_registr_orm.delete_inactive_final_date())
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let user_dto = user_models::UserDto::from(user);

    Ok(HttpResponse::Created().json(user_dto))
}

// Send a confirmation email to recover the user's password.
// POST api//recovery
#[post("/recovery")]
pub async fn recovery(
    config_app: web::Data<config_app::ConfigApp>,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    mailer: web::Data<MailerApp>,
    user_orm: web::Data<UserOrmApp>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    json_body: web::Json<user_models::RecoveryUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_body.validate().map_err(|error| err_validate(error))?;

    let mut recovery_user_dto: user_models::RecoveryUserDto = json_body.0.clone();
    recovery_user_dto.email = recovery_user_dto.email.to_lowercase();

    let email = recovery_user_dto.email.clone();

    // Find in the "user" table an entry by email.
    let user_opt = web::block(move || {
        let existing_user =
            user_orm.find_user_by_email(&email).map_err(|e| err_database(e.to_string()));
        existing_user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If such an entry does not exist, then exit with code 404.
    let user = match user_opt {
        Some(v) => v,
        None => {
            log::error!("{CD_NOT_FOUND}: {MSG_NOT_FOUND_BY_EMAIL}");
            return Err(AppError::new(CD_NOT_FOUND, MSG_NOT_FOUND_BY_EMAIL).set_status(404));
        }
    };
    let user_id = user.id;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is a user with this ID, then move on to the next stage.

    // For this user, find an entry in the "user_recovery" table.
    let user_recovery_opt = web::block(move || {
        let existing_user_recovery = user_recovery_orm2
            .find_user_recovery_by_user_id(user_id)
            .map_err(|e| err_database(e.to_string()));
        existing_user_recovery
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // Prepare data for writing to the "user_recovery" table.
    let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
    let final_date_utc = Utc::now() + Duration::minutes(app_recovery_duration.into());

    let create_user_recovery_dto = CreateUserRecoveryDto {
        user_id: user_id,
        final_date: final_date_utc,
    };
    let user_recovery_id: i32;
    let user_recovery_orm2 = user_recovery_orm.clone();

    // If there is an entry for this user in the "user_recovery" table, then update it with a new token.
    if let Some(user_recovery) = user_recovery_opt {
        user_recovery_id = user_recovery.id;
        let _ = web::block(move || {
            let user_recovery = user_recovery_orm2
                .modify_user_recovery(user_recovery_id, &create_user_recovery_dto)
                .map_err(|e| err_database(e));
            user_recovery
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))??;
    } else {
        // If there is no entry for this user in the "user_recovery" table, then add a new entry.
        // Create a new entity (user_recovery).
        let user_recovery = web::block(move || {
            let user_recovery = user_recovery_orm2
                .create_user_recovery(&create_user_recovery_dto)
                .map_err(|e| err_database(e));
            user_recovery
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))??;

        user_recovery_id = user_recovery.id;
    }

    let num_token = generate_num_token();
    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Pack two parameters (user_recovery_id, num_token) into a recovery_token.
    let recovery_token = encode_dual_token(
        user_recovery_id,
        num_token,
        jwt_secret,
        app_recovery_duration,
    )
    .map_err(|err| err_json_web_token(err))?;

    // Prepare a letter confirming this recovery.
    let domain = &config_app.app_domain;
    let nickname = user.nickname.clone();
    let receiver = user.email.clone();

    // Send an email to this user.
    let result = mailer.send_password_recovery(&receiver, &domain, &nickname, &recovery_token);

    if result.is_err() {
        return Err(err_sending_email(result.unwrap_err()));
    }

    let recovery_user_response_dto = user_models::RecoveryUserResponseDto {
        id: user_recovery_id,
        email: user.email.clone(),
        recovery_token: recovery_token.clone(),
    };

    Ok(HttpResponse::Created().json(recovery_user_response_dto))
}

// Confirm user password recovery.
// PUT api//recovery/{recovery_token}
#[put("/recovery/{recovery_token}")]
pub async fn confirm_recovery(
    request: actix_web::HttpRequest,
    config_jwt: web::Data<config_jwt::ConfigJwt>,
    user_recovery_orm: web::Data<UserRecoveryOrmApp>,
    user_orm: web::Data<UserOrmApp>,
    session_orm: web::Data<SessionOrmApp>,
    json_body: web::Json<user_models::RecoveryDataDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_body.validate().map_err(|error| err_validate(error))?;

    let recovery_data_dto: user_models::RecoveryDataDto = json_body.0.clone();

    // Prepare a password hash.
    let password_hashed =
        hash_tools::encode_hash(&recovery_data_dto.password).map_err(|err| err_encode_hash(err))?;

    let recovery_token = request.match_info().query("recovery_token").to_string();

    let config_jwt = config_jwt.get_ref().clone();
    let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

    // Check the signature and expiration date on the received “recovery_token".
    let dual_token = decode_dual_token(&recovery_token, jwt_secret).map_err(|err| {
        log::error!("{}: {}", err.code, err.message);
        err // 403
    })?;

    let user_recovery_orm2 = user_recovery_orm.clone();

    // Get "user_recovery ID" from "recovery_token".
    let (user_recovery_id, _) = dual_token;

    // Find a record with the specified ID in the “user_recovery" table.
    let user_recovery_opt = web::block(move || {
        let user_recovery = user_recovery_orm2
            .find_user_recovery_by_id(user_recovery_id)
            .map_err(|e| err_database(e));
        user_recovery
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user_recovery = user_recovery_opt.ok_or_else(|| err_not_found())?;
    let user_id = user_recovery.user_id;

    // If there is "user_recovery" with this ID, then move on to the next step.
    let user_orm2 = user_orm.clone();
    // Find an entry in "user" with the specified ID.
    let user_opt = web::block(move || {
        let user = user_orm2.find_user_by_id(user_id).map_err(|e| err_database(e));
        user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    // If no such entry exists, then exit with code 404.
    let user = user_opt.ok_or_else(|| err_not_found())?;

    let user_orm2 = user_orm.clone();
    let modify_user_dto = user_models::ModifyUserDto {
        nickname: None,
        email: None,
        password: Some(password_hashed),
        role: None,
    };
    // Update the password hash for the entry in the "user" table.
    let user_opt = web::block(move || {
        let user = user_orm2.modify_user(user.id, modify_user_dto).map_err(|e| err_database(e));
        user
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let user_recovery_orm2 = user_recovery_orm.clone();
    let _ = web::block(move || {
        // Delete entries in the “user_recovery" table.
        let user_recovery_res = user_recovery_orm2
            .delete_user_recovery(user_recovery_id)
            .map_err(|e| err_database(e));

        // Clear the user session in the "session" table.
        let session_res = session_orm
            .modify_session(user_id, None)
            .map_err(|e| err_database(e.to_string()));

        (user_recovery_res, session_res)
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let user = user_opt.ok_or_else(|| err_not_found())?;
    let user_dto = user_models::UserDto::from(user);

    Ok(HttpResponse::Ok().json(user_dto))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;

    use crate::email::config_smtp;
    use crate::errors::{AppError, CN_VALIDATION};
    use crate::sessions::tokens::decode_dual_token;
    use crate::users::user_recovery_orm::tests::{USER_RECOVERY_ID_1, USER_RECOVERY_ID_2};
    use crate::users::{
        user_models::{
            RecoveryDataDto, RecoveryUserDto, RecoveryUserResponseDto, RegistrUserDto, User,
            UserDto, UserRecovery, UserRole, UserValidateTest,
        },
        user_orm::tests::{UserOrmApp, USER_ID_1, USER_ID_2},
        user_registr_models::UserRegistr,
        user_registr_orm::tests::{UserRegistrOrmApp, USER_REGISTR_ID_1, USER_REGISTR_ID_2},
    };
    use crate::utils::config_app;
    use crate::utils::err::{CD_INVALID_TOKEN, MSG_INVALID_TOKEN};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

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
    fn create_user_registr() -> UserRegistr {
        let today = Utc::now();
        let final_date: DateTime<Utc> = today + Duration::minutes(20);

        let user_registr = UserRegistrOrmApp::new_user_registr(
            USER_REGISTR_ID_1,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
            final_date,
        );

        user_registr
    }

    // ** registration **

    async fn call_service_registr(
        user_vec: Vec<User>,
        user_registr_vec: Vec<UserRegistr>,
        test_request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_app = web::Data::new(config_app::get_test_config());
        let data_config_jwt = web::Data::new(config_jwt::get_test_config());
        let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(user_registr_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .service(registration),
        )
        .await;

        let req = test_request
            .uri("/registration") //POST /registration
            .to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_registration_no_data() {
        let req = test::TestRequest::post();

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_registration_empty_json_object() {
        let req = test::TestRequest::post().set_json(json!({}));

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_min() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: UserValidateTest::nickname_min(),
            email: "Oliver_Taylor@gmail.com".to_string(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MIN);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_max() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: UserValidateTest::nickname_max(),
            email: "Oliver_Taylor@gmail.com".to_string(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_MAX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_nickname_wrong() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: UserValidateTest::nickname_wrong(),
            email: "Oliver_Taylor@gmail.com".to_string(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("nickname: {}", user_models::MSG_NICKNAME_REGEX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_email_min() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Oliver_Taylor".to_string(),
            email: UserValidateTest::email_min(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MIN);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_email_max() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Oliver_Taylor".to_string(),
            email: UserValidateTest::email_max(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MAX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_email_wrong() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Oliver_Taylor".to_string(),
            email: UserValidateTest::email_wrong(),
            password: "passwordD1T1".to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_password_min() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Oliver_Taylor".to_string(),
            email: "Oliver_Taylor@gmail.com".to_string(),
            password: UserValidateTest::password_min(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MIN);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_invalid_dto_password_max() {
        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Oliver_Taylor".to_string(),
            email: "Oliver_Taylor@gmail.com".to_string(),
            password: UserValidateTest::password_max(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MAX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_registration_if_nickname_exists_in_users() {
        let user1: User = create_user();
        let nickname1: String = user1.nickname.to_string();

        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: nickname1,
            email: "Mary_Williams@gmail.com".to_string(),
            password: "passwordD2T2".to_string(),
        });

        let resp = call_service_registr(vec![user1], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_WRONG_NICKNAME);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME);
    }
    #[test]
    async fn test_registration_if_email_exists_in_users() {
        let user1: User = create_user();
        let email1: String = user1.email.to_string();

        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Mary_Williams".to_string(),
            email: email1,
            password: "passwordD2T2".to_string(),
        });

        let resp = call_service_registr(vec![user1], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_WRONG_EMAIL);
        assert_eq!(app_err.message, MSG_WRONG_EMAIL);
    }
    #[test]
    async fn test_registration_if_nickname_exists_in_registr() {
        let user_registr1: UserRegistr = create_user_registr();
        let nickname1: String = user_registr1.nickname.to_string();

        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: nickname1,
            email: "Mary_Williams@gmail.com".to_string(),
            password: "passwordD2T2".to_string(),
        });

        let resp = call_service_registr(vec![], vec![user_registr1], req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_WRONG_NICKNAME);
        assert_eq!(app_err.message, MSG_WRONG_NICKNAME);
    }
    #[test]
    async fn test_registration_if_email_exists_in_registr() {
        let user_registr1: UserRegistr = create_user_registr();
        let email1: String = user_registr1.email.to_string();

        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: "Mary_Williams".to_string(),
            email: email1,
            password: "passwordD2T2".to_string(),
        });

        let resp = call_service_registr(vec![], vec![user_registr1], req).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT); // 409

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_WRONG_EMAIL);
        assert_eq!(app_err.message, MSG_WRONG_EMAIL);
    }
    #[test]
    async fn test_registration_new_user() {
        let nickname = "Mary_Williams".to_string();
        let email = "Mary_Williams@gmail.com".to_string();
        let password = "passwordD2T2".to_string();

        let req = test::TestRequest::post().set_json(RegistrUserDto {
            nickname: nickname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;

        let registr_user_resp: user_models::RegistrUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let final_date: DateTime<Utc> = Utc::now() + Duration::minutes(20);
        let user_registr = UserRegistrOrmApp::new_user_registr(
            1,
            &nickname.to_string(),
            &email.to_string(),
            &password.to_string(),
            final_date,
        );

        assert_eq!(user_registr.nickname, registr_user_resp.nickname);
        assert_eq!(user_registr.email, registr_user_resp.email);

        let registr_token = registr_user_resp.registr_token;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let (user_registr_id, _) = decode_dual_token(&registr_token, jwt_secret).unwrap();
        assert_eq!(USER_REGISTR_ID_2, user_registr_id);
    }

    // ** confirm_registration **

    async fn call_service_conf_registr(
        user_vec: Vec<User>,
        user_registr_vec: Vec<UserRegistr>,
        session_vec: Vec<Session>,
        test_request: TestRequest,
        registr_token: &str,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(config_jwt::get_test_config());
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(user_registr_vec));
        let data_session_orm = web::Data::new(SessionOrmApp::create(session_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(confirm_registration),
        )
        .await;

        let req = test_request
            .uri(&format!("/registration/{registr_token}")) //PUT /registration/{registr_token}
            .to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_registration_confirm_invalid_registr_token() {
        let reg_token = "invalid_registr_token";

        let req = test::TestRequest::put();
        let resp = call_service_conf_registr(vec![], vec![], vec![], req, &reg_token).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_INVALID_TOKEN);
        assert_eq!(app_err.message, MSG_INVALID_TOKEN);
    }
    #[test]
    async fn test_registration_confirm_final_date_has_expired() {
        let user_reg1: UserRegistr = create_user_registr();
        let user_reg1_id = user_reg1.id;
        let user_reg_v = vec![user_reg1];

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let registr_token =
            encode_dual_token(user_reg1_id, num_token, jwt_secret, -reg_duration).unwrap();

        let req = test::TestRequest::put();
        let resp = call_service_conf_registr(vec![], user_reg_v, vec![], req, &registr_token).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_INVALID_TOKEN);
        assert_eq!(app_err.message, MSG_INVALID_TOKEN);
    }
    #[test]
    async fn test_registration_confirm_no_exists_in_user_regist() {
        let user_reg1: UserRegistr = create_user_registr();
        let user_reg1_id = user_reg1.id;
        let user_reg_v = vec![user_reg1];

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let registr_token =
            encode_dual_token(user_reg1_id + 1, num_token, jwt_secret, reg_duration).unwrap();

        let req = test::TestRequest::put();
        let resp = call_service_conf_registr(vec![], user_reg_v, vec![], req, &registr_token).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_NO_CONFIRM);
        assert_eq!(app_err.message, MSG_CONFIRM_NOT_FOUND);
    }
    #[test]
    async fn test_registration_confirm_exists_in_user_regist() {
        let user_reg1: UserRegistr = create_user_registr();
        let user_reg1_id = user_reg1.id;
        let user_reg1b = user_reg1.clone();
        let user_reg_v = vec![user_reg1];

        let num_token = 1234;

        let config_app = config_app::get_test_config();
        let reg_duration: i64 = config_app.app_registr_duration.try_into().unwrap();

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let registr_token =
            encode_dual_token(user_reg1_id, num_token, jwt_secret, reg_duration).unwrap();

        let req = test::TestRequest::put();
        let resp = call_service_conf_registr(vec![], user_reg_v, vec![], req, &registr_token).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, USER_ID_2);
        assert_eq!(user_dto_res.nickname, user_reg1b.nickname);
        assert_eq!(user_dto_res.email, user_reg1b.email);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, UserRole::User);
    }

    // ** recovery **

    async fn call_service_recovery(
        user_vec: Vec<User>,
        user_recovery_vec: Vec<UserRecovery>,
        test_request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_app = web::Data::new(config_app::get_test_config());
        let data_config_jwt = web::Data::new(config_jwt::get_test_config());
        let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(user_recovery_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm))
                .service(recovery),
        )
        .await;

        let req = test_request
            .uri("/recovery") //POST /recovery
            .to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_recovery_no_data() {
        let req = test::TestRequest::post();

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_recovery_empty_json_object() {
        let req = test::TestRequest::post().set_json(json!({}));

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_recovery_invalid_dto_email_min() {
        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: UserValidateTest::email_min(),
        });

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MIN);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_recovery_invalid_dto_email_max() {
        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: UserValidateTest::email_max(),
        });

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL_MAX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_recovery_invalid_dto_email_wrong() {
        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: UserValidateTest::email_wrong(),
        });

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("email: {}", user_models::MSG_EMAIL);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_recovery_if_user_with_email_does_not_exist() {
        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: "Oliver_Taylor@gmail.com".to_string(),
        });

        let resp = call_service_recovery(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_NOT_FOUND);
        assert_eq!(app_err.message, MSG_NOT_FOUND_BY_EMAIL);
    }
    #[test]
    async fn test_recovery_if_user_recovery_does_not_exist() {
        let user1: User = create_user();
        let user1_email = user1.email.to_string();

        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: user1_email.to_string(),
        });

        let resp = call_service_recovery(vec![user1], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_recov_res: RecoveryUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, USER_RECOVERY_ID_2);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) =
            decode_dual_token(&recovery_token, jwt_secret).expect("decode_dual_token error");
        assert_eq!(USER_RECOVERY_ID_2, user_recovery_id);
    }
    #[test]
    async fn test_recovery_if_user_recovery_already_exists() {
        let user1: User = create_user();
        let user1_id = user1.id;
        let user1_email = user1.email.to_string();

        let config_app = config_app::get_test_config();
        let app_recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::minutes(app_recovery_duration.into());

        let user_recovery1: UserRecovery =
            UserRecoveryOrmApp::new_user_recovery(USER_RECOVERY_ID_1, user1_id, final_date_utc);
        let user_recovery1_id = user_recovery1.id;

        let req = test::TestRequest::post().set_json(RecoveryUserDto {
            email: user1_email.to_string(),
        });

        let resp = call_service_recovery(vec![user1], vec![user_recovery1], req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let user_recov_res: RecoveryUserResponseDto =
            serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_recov_res.id, user_recovery1_id);
        assert_eq!(user_recov_res.email, user1_email.to_string());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token = user_recov_res.recovery_token;
        // Check the signature and expiration date on the “recovery_token".
        let (user_recovery_id, _) =
            decode_dual_token(&recovery_token, jwt_secret).expect("decode_dual_token error");
        assert_eq!(user_recovery1_id, user_recovery_id);
    }

    // ** confirm recovery **

    async fn call_service_conf_recovery(
        user_vec: Vec<User>,
        user_recovery_vec: Vec<UserRecovery>,
        session_vec: Vec<Session>,
        test_request: TestRequest,
        recovery_token: &str,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(config_jwt::get_test_config());
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_user_recovery_orm = web::Data::new(UserRecoveryOrmApp::create(user_recovery_vec));
        let data_session_orm = web::Data::new(SessionOrmApp::create(session_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_recovery_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(confirm_recovery),
        )
        .await;

        let req = test_request
            .uri(&format!("/recovery/{recovery_token}")) //PUT /recovery/{recovery_token}
            .to_request();

        test::call_service(&app, req).await
    }

    #[test]
    async fn test_recovery_confirm_invalid_dto_password_min() {
        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: UserValidateTest::password_min(),
        });

        let resp = call_service_conf_recovery(vec![], vec![], vec![], req, &"token").await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MIN);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_recovery_confirm_invalid_dto_password_max() {
        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: UserValidateTest::password_max(),
        });

        let resp = call_service_conf_recovery(vec![], vec![], vec![], req, &"token").await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CN_VALIDATION);
        let msg_err = format!("password: {}", user_models::MSG_PASSWORD_MAX);
        assert_eq!(app_err.message, msg_err);
    }
    #[test]
    async fn test_recovery_confirm_invalid_recovery_token() {
        let reg_token = "invalid_recovery_token";

        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: "passwordQ2V2".to_string(),
        });
        let resp = call_service_conf_recovery(vec![], vec![], vec![], req, &reg_token).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_INVALID_TOKEN);
        assert_eq!(app_err.message, MSG_INVALID_TOKEN);
    }
    #[test]
    async fn test_recovery_confirm_final_date_has_expired() {
        let user1: User = create_user();
        let user1_id = user1.id;
        let user1_v = vec![user1];

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::minutes(-recovery_duration);

        let user_recovery1: UserRecovery =
            UserRecoveryOrmApp::new_user_recovery(USER_RECOVERY_ID_1, user1_id, final_date_utc);
        let user_recovery1_id = user_recovery1.id;
        let user_recov1_v = vec![user_recovery1];

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token =
            encode_dual_token(user_recovery1_id, num_token, jwt_secret, -recovery_duration)
                .unwrap();

        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: "passwordQ2V2".to_string(),
        });
        let resp =
            call_service_conf_recovery(user1_v, user_recov1_v, vec![], req, &recovery_token).await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN); // 403

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_INVALID_TOKEN);
        assert_eq!(app_err.message, MSG_INVALID_TOKEN);
    }
    #[test]
    async fn test_recovery_confirm_no_exists_in_user_recovery() {
        let user1: User = create_user();
        let user1_id = user1.id;
        let user1_v = vec![user1];

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::minutes(recovery_duration);

        let user_recovery1: UserRecovery =
            UserRecoveryOrmApp::new_user_recovery(USER_RECOVERY_ID_1, user1_id, final_date_utc);
        let user_recovery1_id = user_recovery1.id;
        let user_recov1_v = vec![user_recovery1];

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token = encode_dual_token(
            user_recovery1_id + 1,
            num_token,
            jwt_secret,
            recovery_duration,
        )
        .unwrap();

        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: "passwordQ2V2".to_string(),
        });
        let resp =
            call_service_conf_recovery(user1_v, user_recov1_v, vec![], req, &recovery_token).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_NO_CONFIRM);
        assert_eq!(app_err.message, MSG_CONFIRM_NOT_FOUND);
    }
    #[test]
    async fn test_recovery_confirm_no_exists_in_user() {
        let user1: User = create_user();
        let user1_id = user1.id;
        let user1_v = vec![user1];

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::minutes(recovery_duration);

        let user_recovery1: UserRecovery =
            UserRecoveryOrmApp::new_user_recovery(USER_RECOVERY_ID_1, user1_id + 1, final_date_utc);
        let user_recovery1_id = user_recovery1.id;
        let user_recov1_v = vec![user_recovery1];

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token = encode_dual_token(
            user_recovery1_id + 1,
            num_token,
            jwt_secret,
            recovery_duration,
        )
        .unwrap();

        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: "passwordQ2V2".to_string(),
        });
        let resp =
            call_service_conf_recovery(user1_v, user_recov1_v, vec![], req, &recovery_token).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, CD_NO_CONFIRM);
        assert_eq!(app_err.message, MSG_CONFIRM_NOT_FOUND);
    }
    #[test]
    async fn test_recovery_confirm_success() {
        let user1: User = create_user();
        let user1_id = user1.id;
        let user1b = user1.clone();
        let user1_v = vec![user1];

        let config_app = config_app::get_test_config();
        let recovery_duration: i64 = config_app.app_recovery_duration.try_into().unwrap();
        let final_date_utc = Utc::now() + Duration::minutes(recovery_duration);

        let user_recovery1: UserRecovery =
            UserRecoveryOrmApp::new_user_recovery(USER_RECOVERY_ID_1, user1_id, final_date_utc);
        let user_recovery1_id = user_recovery1.id;
        let user_recov1_v = vec![user_recovery1];

        let num_token = 1234;

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes().clone();
        let recovery_token =
            encode_dual_token(user_recovery1_id, num_token, jwt_secret, recovery_duration).unwrap();

        let req = test::TestRequest::put().set_json(RecoveryDataDto {
            password: "passwordQ2V2".to_string(),
        });
        let resp =
            call_service_conf_recovery(user1_v, user_recov1_v, vec![], req, &recovery_token).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let user_dto_res: UserDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(user_dto_res.id, user1_id);
        assert_eq!(user_dto_res.nickname, user1b.nickname);
        assert_eq!(user_dto_res.email, user1b.email);
        assert_eq!(user_dto_res.password, "");
        assert_eq!(user_dto_res.role, user1b.role);
    }
}

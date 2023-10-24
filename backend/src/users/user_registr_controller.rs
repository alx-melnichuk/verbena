use actix_web::{post, web, HttpResponse};
use chrono::{Duration, Utc};
use validator::Validate;

#[cfg(not(feature = "mockdata"))]
use crate::email::mailer::inst::MailerApp;
#[cfg(feature = "mockdata")]
use crate::email::mailer::tests::MailerApp;
use crate::email::mailer::Mailer;
use crate::errors::{AppError, ERR_CN_VALIDATION};
use crate::hash_tools;
use crate::users::{
    user_models, user_orm::UserOrm, user_registr_models, user_registr_orm::UserRegistrOrm,
};
#[cfg(not(feature = "mockdata"))]
use crate::users::{user_orm::inst::UserOrmApp, user_registr_orm::inst::UserRegistrOrmApp};
#[cfg(feature = "mockdata")]
use crate::users::{user_orm::tests::UserOrmApp, user_registr_orm::tests::UserRegistrOrmApp};

use crate::utils::{config_app, err};

pub const CD_WRONG_EMAIL: &str = "WrongEmail";
pub const MSG_WRONG_EMAIL: &str = "The specified email is incorrect!";

pub const CD_WRONG_NICKNAME: &str = "WrongNickname";
pub const MSG_WRONG_NICKNAME: &str = "The specified nickname is incorrect!";

pub const CD_ERROR_SENDING_EMAIL: &str = "ErrorSendingEmail";

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     POST api/registration
    cfg.service(registration);
}

fn err_database(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::debug!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_wrong_email_or_nickname(is_nickname: bool) -> AppError {
    let val = if is_nickname {
        (CD_WRONG_NICKNAME, MSG_WRONG_NICKNAME)
    } else {
        (CD_WRONG_EMAIL, MSG_WRONG_EMAIL)
    };
    log::debug!("{}: {}", val.0, val.1);
    AppError::new(val.0, val.1).set_status(409)
}

// Send a confirmation email to register the user.
// POST api/registration
#[post("/registration")]
pub async fn registration(
    config_app: web::Data<config_app::ConfigApp>,
    mailer: web::Data<MailerApp>,
    user_orm: web::Data<UserOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    json_user_dto: web::Json<user_models::RegistrUserDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Checking the validity of the data model.
    json_user_dto.validate().map_err(|errors| {
        log::debug!("{}: {}", ERR_CN_VALIDATION, errors.to_string());
        AppError::from(errors)
    })?;

    let mut registr_user_dto: user_models::RegistrUserDto = json_user_dto.0.clone();
    registr_user_dto.nickname = registr_user_dto.nickname.to_lowercase();
    registr_user_dto.email = registr_user_dto.email.to_lowercase();

    let password = registr_user_dto.password.clone();
    let password_hashed = hash_tools::ctreate_hash(&password).map_err(|e| {
        log::debug!("{}: {}", err::CD_HASHING_PASSWD, e.to_string());
        AppError::new(err::CD_HASHING_PASSWD, &e.to_string()).set_status(500)
    })?;

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

    // Find in the "user_registration" table an entry with an active date, by nickname or email.
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

    // If there is no such record, then add the specified data to the "user_registration" table.

    let nickname = registr_user_dto.nickname.clone();
    let email = registr_user_dto.email.clone();
    // let dt = Local::now().dt.naive_utc();
    // let final_date_utc = Utc::now();
    let app_registr_duration = config_app.app_registr_duration.try_into().unwrap();
    let final_date_utc = Utc::now() + Duration::minutes(app_registr_duration);

    let insert_user_registr_dto = user_registr_models::CreateUserRegistrDto {
        nickname,
        email,
        password: password_hashed,
        final_date: final_date_utc,
    };

    let user_registr = web::block(move || {
        // Create a new entity (user).
        let user_registr = user_registr_orm
            .create_user_registr(&insert_user_registr_dto)
            .map_err(|e| err_database(e));
        user_registr
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))??;

    let target = format!("target_{}", user_registr.id);

    // Prepare a letter confirming this registration.
    let domain = &config_app.app_domain;
    let nickname = registr_user_dto.nickname.clone();
    let receiver = registr_user_dto.email.clone();

    let result = mailer.send_verification_code(&receiver, &domain, &nickname, &target);
    if result.is_err() {
        let err = result.unwrap_err();
        eprintln!("Failed to send email: {:?}", err);
        log::debug!("{CD_ERROR_SENDING_EMAIL}: {err}");
        return Err(AppError::new(CD_ERROR_SENDING_EMAIL, &err).set_status(500));
    }

    let registr_user_response_dto = user_models::RegistrUserResponseDto {
        nickname: registr_user_dto.nickname.clone(),
        email: registr_user_dto.email.clone(),
        target: target.clone(),
    };

    Ok(HttpResponse::Ok().json(registr_user_response_dto))
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::test::TestRequest;
    use actix_web::{dev, http, test, web, App};
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;

    use crate::email::config_smtp;
    use crate::errors::{AppError, ERR_CN_VALIDATION};
    use crate::users::user_models::{RegistrUserDto, User, UserRole, UserValidateTest};
    use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
    use crate::users::{user_orm::tests::UserOrmApp, user_registr_models};
    use crate::utils::config_app;

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(
            1001,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
        );
        user.role = UserRole::User;
        user
    }
    fn create_user_registr() -> user_registr_models::UserRegistr {
        let today = Utc::now();
        let final_date: DateTime<Utc> = today + Duration::seconds(20);

        let user_registr = UserRegistrOrmApp::new_user_registr(
            1001,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwdT1R1",
            final_date,
        );

        user_registr
    }

    async fn call_service_registr(
        user_vec: Vec<User>,
        user_registr_vec: Vec<user_registr_models::UserRegistr>,
        test_request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_app = web::Data::new(config_app::get_test_config());
        let data_mailer = web::Data::new(MailerApp::new(config_smtp::get_test_config()));
        let data_user_orm = web::Data::new(UserOrmApp::create(user_vec));
        let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(user_registr_vec));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_app))
                .app_data(web::Data::clone(&data_mailer))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .service(registration),
        )
        .await;
        let req = test_request
            .uri("/registration") //POST /registration
            .to_request();

        let resp = test::call_service(&app, req).await;

        resp
    }

    #[test]
    async fn test_registration_no_data() {
        let req = test::TestRequest::post();

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Content type error";
        assert!(body_str.contains(expected_message));
    }
    #[test]
    async fn test_registration_empty_json_object() {
        let req = test::TestRequest::post().set_json(json!({}));

        let resp = call_service_registr(vec![], vec![], req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        assert_eq!(app_err.code, ERR_CN_VALIDATION);
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
        let user_registr1: user_registr_models::UserRegistr = create_user_registr();
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
        let user_registr1: user_registr_models::UserRegistr = create_user_registr();
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
}

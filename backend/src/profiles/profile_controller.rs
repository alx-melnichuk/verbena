use std::{borrow::Cow::Borrowed, ops::Deref};

use actix_web::{delete, get, put, web, HttpResponse};
use log;
use serde_json::json;
use utoipa;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::hash_tools;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    profile_models::{
        self, NewPasswordProfileDto, Profile, ProfileDto, UniquenessProfileDto, PROFILE_THEME_DARK,
        PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::users::{user_models::UserRole, user_registr_orm::UserRegistrOrm};
use crate::utils::parser;
use crate::validators::{msg_validation, Validator};

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
            .service(get_profile_current)
            // PUT /api/profiles_new_password
            .service(put_profile_new_password)
            // DELETE /api/profiles/{id}
            .service(delete_profile)
            // DELETE /api/profiles_current
            .service(delete_profile_current);
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

/// put_profile_new_password
///
/// Update the password of the current user (`ProfileDto`).
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/profiles_new_password  \
/// -d {"password": "Pass_123", "new_password": "Pass#3*0"} \
/// -H 'Content-Type: application/json'
/// ```
///
/// Return the current user (`ProfileDto`) with status 200 or 204 (no content) if the user is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Data about the current user.", body = ProfileDto,
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
        (status = 204, description = "The current user was not found."),
        (status = 401, description = "The nickname or password is incorrect or the token is missing.", body = AppError, 
            example = json!(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 409, description = "Error when comparing password hashes.", body = AppError,
            example = json!(AppError::conflict409(&format!("{}: {}", err::MSG_INVALID_HASH, "Parameter is empty.")))),
        (status = 417, body = [AppError],
            description = "Validation error. `curl -i -X PUT http://localhost:8080/api/profiles_new_password \
            -d '{\"password\": \"pas\" \"new_password\": \"word\"}'`",
            example = json!(AppError::validations(
                (NewPasswordProfileDto {password: "pas".to_string(), new_password: "word".to_string()}).validate().err().unwrap()) )),
        (status = 500, description = "Error while calculating the password hash.", body = AppError, 
            example = json!(AppError::internal_err500(&format!("{}: {}", err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[put("/api/profiles_new_password", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_profile_new_password(
    authenticated: Authenticated,
    profile_orm: web::Data<ProfileOrmApp>,
    json_body: web::Json<NewPasswordProfileDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // 1.308634s
    let profile = authenticated.deref();
    let user_id = profile.user_id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let new_password_user: NewPasswordProfileDto = json_body.into_inner();
    let new_password = new_password_user.new_password.clone();
    // Get a hash of the new password.
    let new_password_hashed = hash_tools::encode_hash(&new_password).map_err(|e| {
        let message = format!("{}: {}", err::MSG_ERROR_HASHING_PASSWORD, e.to_string());
        log::error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
        AppError::internal_err500(&message) // 500
    })?;

    let profile_orm2 = profile_orm.clone();
    let opt_profile2 = web::block(move || {
        // Find user by nickname or email.
        let existing_profile = profile_orm2.get_profile_user_by_id(user_id, true)
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

    let profile_pwd = opt_profile2.ok_or_else(|| {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401
    })?;

    // Get the value of the old password.
    let old_password = new_password_user.password.clone();
    // Get a hash of the old password.
    let profile_hashed_old_password = profile_pwd.password.to_string();
    // Check whether the hash for the specified password value matches the old password hash.
    let password_matches = hash_tools::compare_hash(&old_password, &profile_hashed_old_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_HASH, &e);
        log::error!("{}: {}", err::CD_CONFLICT, &message);
        AppError::conflict409(&message) // 409
    })?;
    // If the hash for the specified password does not match the old password hash, then return an error.
    if !password_matches {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_PASSWORD_INCORRECT);
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401
    }

    // Set a new user password.

    // Create a model to update the "password" field in the user profile.
    let modify_profile = profile_models::ModifyProfile{
        nickname: None, email: None, password: Some(new_password_hashed), role: None, avatar: None, descript: None, theme: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(user_id, modify_profile)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        opt_profile1
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    // If the user profile was updated successfully, return the user profile.
    if let Some(profile) = opt_profile {
        let profile_dto = ProfileDto::from(profile);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        // Otherwise, return empty content.
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_profile
///
/// Delete a user profile for the specified ID.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/profiles/1
/// ```
///
/// Return the deleted user's profile (`ProfileDto`) with status 200 or 204 (no content) if the user's profile is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified user profile was deleted successfully.", body = ProfileDto,
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
        (status = 204, description = "The specified user profile was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X DELETE http://localhost:8080/api/users/2a`",
            body = AppError, example = json!(AppError::range_not_satisfiable416(
                &format!("{}; {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "`id` - invalid digit found in string (2a)")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/profiles/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
pub async fn delete_profile(
    profile_orm: web::Data<ProfileOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;

    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(id)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_profile
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if let Some(profile) = opt_profile {
        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_profile_current
///
/// Delete the current user's profile.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/profiles_current
/// ```
///
/// Return the deleted current user's profile (`ProfileDto`) with status 200 or 204 (no content) if the current user's profile is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The current user's profile has been successfully deleted.", body = ProfileDto,
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
        (status = 204, description = "The current user's profile was not found."),
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
#[delete("/api/profiles_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_profile_current(
    authenticated: Authenticated,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    let profile = authenticated.deref();
    let id = profile.user_id;

    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(id)
        .map_err(|e| {
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

    if let Some(profile) = opt_profile {
        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
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
    use chrono::{DateTime, Duration, Utc};

    use crate::extractors::authentication::BEARER;
    use crate::profiles::profile_models::ProfileTest;
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
    fn create_profile_pwd(role: u8, password: &str) -> Profile {
        let mut profile = create_profile(role);
        profile.password = hash_tools::encode_hash(password.to_string()).unwrap(); // hashed
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
    fn get_cfg_data(is_registr: bool, role: u8) -> (config_jwt::ConfigJwt, (Vec<Profile>, Vec<Session>, Vec<UserRegistr>), String) {
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

        let data_c = (vec![profile1], vec![session1], user_registr_vec);

        (config_jwt, data_c, token)
    }
    fn configure_profile(
        config_jwt: config_jwt::ConfigJwt,                      // configuration
        data_c: (Vec<Profile>, Vec<Session>, Vec<UserRegistr>), // cortege of data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);

            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm));
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
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

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
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

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
        assert_eq!(resp.status(), http::StatusCode::NOT_ACCEPTABLE); // 406

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::RANGE_NOT_SATISFIABLE); // 416

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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

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
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
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
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res, profile_dto_ser);
    }

    // ** put_profile_new_password **
    #[actix_web::test]
    async fn test_put_profile_new_password_no_data() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .to_request();
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
    async fn test_put_profile_new_password_empty_json_object() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(json!({}))
            .to_request();
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
    async fn test_put_profile_new_password_invalid_dto_password_empty() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: "".to_string(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_password_min() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_min(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_password_max() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_max(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_password_wrong() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: ProfileTest::password_wrong(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_dto_new_password_empty() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: "".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NEW_PASSWORD_REQUIRED]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_min() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_min()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NEW_PASSWORD_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_max() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_max()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NEW_PASSWORD_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_wrong() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password, new_password: ProfileTest::password_wrong()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NEW_PASSWORD_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_dto_new_password_equal_old_value() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password.clone(), new_password: old_password
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NEW_PASSWORD_EQUAL_OLD_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_invalid_hash_password() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile(USER);
        profile1.password = "invali_hash_password".to_string();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;

        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password.to_string(), new_password: "passwdJ3S9".to_string()
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
    async fn test_put_profile_new_password_invalid_password() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: format!("{}a", old_password), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED); // 401

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_PASSWORD_INCORRECT);
    }
    #[actix_web::test]
    async fn test_put_profile_new_password_valid_data() {
        let old_password = "passwdP1C1".to_string();
        let mut profile1: Profile = create_profile_pwd(USER, &old_password);
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        profile1.user_id = data_c.0.get(0).unwrap().user_id;
        let profile1_dto = ProfileDto::from(profile1.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2);

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile_new_password).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles_new_password")
            .insert_header(header_auth(&token))
            .set_json(NewPasswordProfileDto {
                password: old_password.to_string(), new_password: "passwdJ3S9".to_string()
            })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();

        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile_dto_ser.id);
        assert_eq!(profile_dto_res.nickname, profile_dto_ser.nickname);
        assert_eq!(profile_dto_res.email, profile_dto_ser.email);
        assert_eq!(profile_dto_res.role, profile_dto_ser.role);
        assert_eq!(profile_dto_res.avatar, profile_dto_ser.avatar);
        assert_eq!(profile_dto_res.descript, profile_dto_ser.descript);
        assert_eq!(profile_dto_res.theme, profile_dto_ser.theme);
        assert_eq!(profile_dto_res.created_at, profile_dto_ser.created_at);
    }

    // ** delete_profile **
    #[actix_web::test]
    async fn test_delete_profile_profile_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile_id_bad = format!("{}a", profile1.user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_RANGE_NOT_SATISFIABLE);
        #[rustfmt::skip]
        let msg = format!("{}; `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, profile_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_delete_profile_profile_not_exist() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile_id = profile1.user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_profile_profile_exists() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile2 = ProfileOrmApp::new_profile(2, "Logan_Lewis", "Logan_Lewis@gmail.com", UserRole::User);

        let profile_vec = ProfileOrmApp::create(&vec![profile1, profile2]).profile_vec;
        let profile2_dto = ProfileDto::from(profile_vec.get(1).unwrap().clone());
        let profile2_id = profile2_dto.id;

        let data_c = (profile_vec, data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile2_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile2_dto).to_string();
        let profile2b_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile2b_dto_ser);
    }

    // ** delete_profile_current **
    #[actix_web::test]
    async fn test_delete_profile_current_valid_token() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }
}

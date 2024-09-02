use std::{borrow::Cow, ops::Deref, path};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use log;
use mime::IMAGE;
use serde_json::json;
use utoipa;

use crate::cdis::coding;
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::hash_tools;
use crate::loading::dynamic_image;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    config_prfl,
    profile_models::{
        ModifyProfile, ModifyProfileDto, NewPasswordProfileDto, Profile, ProfileDto, PROFILE_THEME_DARK,
        PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
use crate::settings::err;
use crate::users::user_models::UserRole;
use crate::utils::parser;
use crate::validators::{self, msg_validation, ValidationChecks, Validator};

pub const ALIAS_AVATAR_FILES_DIR: &str = "/avatar";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        config
            // PUT /api/profiles
            .service(put_profile)
            // PUT /api/profiles_new_password
            .service(put_profile_new_password)
            // DELETE /api/profiles/{id}
            .service(delete_profile)
            // DELETE /api/profiles_current
            .service(delete_profile_current);
    }
}

fn remove_file_and_log(file_name: &str, msg: &str) {
    if file_name.len() > 0 {
        let res_remove = std::fs::remove_file(file_name);
        if let Err(err) = res_remove {
            log::error!("{} remove_file({}): error: {:?}", msg, file_name, err);
        }
    }
}
fn get_file_name(user_id: i32, date_time: DateTime<Utc>) -> String {
    format!("{}_{}", user_id, coding::encode(date_time, 1))
}
// Convert the file to another mime type.
#[rustfmt::skip]
fn convert_avatar_file(file_img_path: &str, config_prfl: config_prfl::ConfigPrfl, name: &str) -> Result<Option<String>, String> {
    let path: path::PathBuf = path::PathBuf::from(&file_img_path);
    let file_source_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
    let img_file_ext = config_prfl.prfl_avatar_ext.clone().unwrap_or(file_source_ext);
    // If you need to save in the specified format or convert
    // to the specified size (img_max_width > 0 || img_max_height > 0), then do the following.
    if config_prfl.prfl_avatar_ext.is_some()
        || config_prfl.prfl_avatar_max_width > 0
        || config_prfl.prfl_avatar_max_height > 0
    {
        // Convert the file to another mime type.
        let path_file = dynamic_image::convert_file(
            &file_img_path,
            &img_file_ext,
            config_prfl.prfl_avatar_max_width,
            config_prfl.prfl_avatar_max_height,
        )?;
        if !path_file.eq(&file_img_path) {
            remove_file_and_log(&file_img_path, name);
        }
        Ok(Some(path_file))
    } else {
        Ok(None)
    }
}

#[derive(Debug, MultipartForm)]
pub struct ModifyProfileForm {
    pub nickname: Option<Text<String>>,
    pub email: Option<Text<String>>,
    pub role: Option<Text<String>>,
    pub descript: Option<Text<String>>,
    pub theme: Option<Text<String>>,
    pub avatarfile: Option<TempFile>,
}

impl ModifyProfileForm {
    pub fn convert(modify_profile_form: ModifyProfileForm) -> (ModifyProfileDto, Option<TempFile>) {
        (
            ModifyProfileDto {
                nickname: modify_profile_form.nickname.map(|v| v.into_inner()),
                email: modify_profile_form.email.map(|v| v.into_inner()),
                role: modify_profile_form.role.map(|v| v.into_inner()),
                descript: modify_profile_form.descript.map(|v| v.into_inner()),
                theme: modify_profile_form.theme.map(|v| v.into_inner()),
            },
            modify_profile_form.avatarfile,
        )
    }
}

/// put_profiles
///
/// Update the current user profile with new data.
///
/// Multipart/form-data is used to transfer data.
///
/// Request structure:
/// ```text
/// {
///   nickname?: String,     // optional - user nickname;
///   email?: String,        // optional - user email;
///   role?: String,         // optional - user role;
///   descript?: String,     // optional - user description;
///   theme?: String,        // optional - user color theme ("light", "dark");
///   avatarfile?: TempFile, // optional - attached user image file (jpeg,gif,png,bmp);
/// }
/// ```
///
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/profiles -F "descript=Descript"
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/profiles -F "nickname=user2000" -F "email=user2000@gmail.ua" \
///   -F "role=user" -F "descript=Descript"  -F "theme=dark"
/// ```
/// Additionally, you can specify the name of the image file.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/profiles -F "descript=descript2" -F "avatarfile=@image.jpg"
/// ```
///  
/// Return the profile with updated data (`ProfileDto`) with status 200 or 204 (no content) if the profile is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "Update the current user profile with new data.", body = ProfileDto,
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
        (status = 413, description = "Invalid image file size. `curl -i -X PUT http://localhost:8080/api/profiles
            -F 'avatarfile=@image.jpg'`", body = AppError,
            example = json!(AppError::content_large413(err::MSG_INVALID_FILE_SIZE).add_param(Cow::Borrowed("invalidFileSize"),
                &json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X PUT http://localhost:8080/api/profiles
            -F 'avatarfile=@image.svg'`", body = AppError,
            example = json!(AppError::unsupported_type415(err::MSG_INVALID_FILE_TYPE).add_param(Cow::Borrowed("invalidFileType"),
                &json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 417, description = "Validation error. `curl -X PUT http://localhost:8080/api/profiles
            -F 'descript=Description' -F 'theme=light' -F 'avatarfile=@image.png'`", body = [AppError],
            example = json!(AppError::validations(
                (ModifyProfileDto { nickname: None, email: None, role: None,
                    descript: Some("d".to_string()), theme: Some("light".to_string()) }).validate().err().unwrap()
            ) )),
        (status = 500, description = "Error loading file.", body = AppError, example = json!(
            AppError::internal_err500(&format!("{}; {} - {}", err::MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg", "File not found.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = AppError,
            example = json!(AppError::not_extended510(
                &format!("{}; {}", err::MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\"")))),
    ),
    security(("bearer_auth" = [])),
)]
// PUT /api/profiles
#[rustfmt::skip]
#[put("/api/profiles", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_profile(
    authenticated: Authenticated,
    config_prfl: web::Data<config_prfl::ConfigPrfl>,
    profile_orm: web::Data<ProfileOrmApp>,
    MultipartForm(modify_profile_form): MultipartForm<ModifyProfileForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get the current user's profile.
    let profile = authenticated.deref();
    let curr_user_id = profile.user_id;
    let opt_curr_avatar = profile.avatar.clone();

    // Get data from MultipartForm.
    let (modify_profile_dto, avatar_file) = ModifyProfileForm::convert(modify_profile_form);
    
    // If there is not a single field in the MultipartForm, it gives an error 400 "Multipart stream is incomplete".

    // Checking the validity of the data model.
    let validation_res = modify_profile_dto.validate();
    
    if let Err(validation_errors) = validation_res {
        let mut is_no_fields_to_update = false;
        let errors = validation_errors.iter().map(|err| {
            if !is_no_fields_to_update && err.params.contains_key(validators::NM_NO_FIELDS_TO_UPDATE) {
                is_no_fields_to_update = true;
                let valid_names = [ModifyProfileDto::valid_names(), vec!["avatarfile"]].concat().join(",");
                ValidationChecks::no_fields_to_update(&[false], &valid_names, err::MSG_NO_FIELDS_TO_UPDATE).err().unwrap()
            } else {
                err.clone()
            }
        }).collect();
        if !is_no_fields_to_update || avatar_file.is_none() {
            log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&errors));
            return Ok(AppError::to_response(&AppError::validations(errors))); // 417
        }
    }

    let mut avatar: Option<Option<String>> = None;

    let config_prfl = config_prfl.get_ref().clone();
    let avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();
    let mut path_new_avatar_file = "".to_string();

    while let Some(temp_file) = avatar_file {
        // Delete the old version of the avatar file.
        if temp_file.size == 0 {
            avatar = Some(None); // Set the "avatar" field to `NULL`.
            break;
        }

        // Check file size for maximum value.
        let avatar_max_size = usize::try_from(config_prfl.prfl_avatar_max_size).unwrap();
        if avatar_max_size > 0 && temp_file.size > avatar_max_size {
            let json = json!({ "actualFileSize": temp_file.size, "maxFileSize": avatar_max_size });
            log::error!("{}: {}; {}", err::CD_CONTENT_TOO_LARGE, err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(AppError::content_large413(err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }
        
        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types: Vec<String> = config_prfl.prfl_avatar_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            log::error!("{}: {}; {}", err::CD_UNSUPPORTED_TYPE, err::MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(AppError::unsupported_type415(err::MSG_INVALID_FILE_TYPE) // 415
                .add_param(Cow::Borrowed("invalidFileType"), &json));
        }

        // Get the file stem and extension for the new file.
        #[rustfmt::skip]
        let name = format!("{}.{}", get_file_name(curr_user_id, Utc::now()), file_mime_type.replace(&format!("{}/", IMAGE), ""));
        // Add 'file path' + 'file name'.'file extension'.
        let path: path::PathBuf = [&avatar_files_dir, &name].iter().collect();
        let full_path_file = path.to_str().unwrap().to_string();
        // Persist the temporary file at the target path.
        // Note: if a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            let message = format!("{}; {} - {}", err::MSG_ERROR_UPLOAD_FILE, &full_path_file, err.to_string());
            log::error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
            return Err(AppError::internal_err500(&message)); // 500
        }
        path_new_avatar_file = full_path_file;
        
        // Convert the file to another mime type.
        let res_convert_img_file = convert_avatar_file(&path_new_avatar_file, config_prfl.clone(), "put_profile()")
        .map_err(|e| {
            let message = format!("{}; {}", err::MSG_ERROR_CONVERT_FILE, e);
            log::error!("{}: {}", err::CD_NOT_EXTENDED, &message);
            AppError::not_extended510(&message) // 510
        })?;
        if let Some(new_path_file) = res_convert_img_file {
            path_new_avatar_file = new_path_file;
        }
    
        let alias_avatar_file = path_new_avatar_file.replace(&avatar_files_dir, ALIAS_AVATAR_FILES_DIR);
        avatar = Some(Some(alias_avatar_file));

        break;
    }
    let mut modify_profile: ModifyProfile = modify_profile_dto.into();
    modify_profile.avatar = avatar;

    let res_data = web::block(move || {
        let mut old_avatar_file = "".to_string();

        if modify_profile.avatar.is_some() {
            // Get the current avatar file name for the profile.
            if let Some(old_avatar) = opt_curr_avatar {
                old_avatar_file = old_avatar.replace(ALIAS_AVATAR_FILES_DIR, &avatar_files_dir);
            }
        }
        // Modify an entity (profile).
        let res_profile = profile_orm.modify_profile(curr_user_id, modify_profile)
        .map_err(|e| {
            log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });

        (old_avatar_file, res_profile)
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let (path_old_avatar_file, res_profile) = res_data;

    let mut opt_profile_dto: Option<ProfileDto> = None;

    if let Ok(Some(profile)) = res_profile {
        opt_profile_dto = Some( ProfileDto::from(profile) );

        remove_file_and_log(&path_old_avatar_file, &"put_profile()");
    } else {
        remove_file_and_log(&path_new_avatar_file, &"put_profile()");
    }
    if let Some(profile_dto) = opt_profile_dto {
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
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
    let profile_id = profile.user_id;

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
    let opt_profile_pwd = web::block(move || {
        // Find user by nickname or email.
        let existing_profile = profile_orm2.get_profile_user_by_id(profile_id, true)
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

    let profile_pwd = opt_profile_pwd.ok_or_else(|| {
        log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
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
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    // Set a new user password.

    // Create a model to update the "password" field in the user profile.
    let modify_profile = ModifyProfile{
        nickname: None, email: None, password: Some(new_password_hashed), role: None, avatar: None, descript: None, theme: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(profile_id, modify_profile)
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
    config_prfl: web::Data<config_prfl::ConfigPrfl>,
    profile_orm: web::Data<ProfileOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
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
        // Get the path to the "avatar" file.
        let path_file_img: String = profile.avatar.clone().unwrap_or("".to_string());
        if path_file_img.starts_with(ALIAS_AVATAR_FILES_DIR) {
            // If there is a "avatar" file, then delete this file.
            let config_prfl = config_prfl.get_ref().clone();
            let avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();
            let avatar_name_full_path = path_file_img.replace(ALIAS_AVATAR_FILES_DIR, &avatar_files_dir);
            remove_file_and_log(&avatar_name_full_path, &"delete_profile()");
        }
        
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
    config_prfl: web::Data<config_prfl::ConfigPrfl>,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
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
        // Get the path to the "avatar" file.
        let path_file_img: String = profile.avatar.clone().unwrap_or("".to_string());
        if path_file_img.starts_with(ALIAS_AVATAR_FILES_DIR) {
            // If there is a "avatar" file, then delete this file.
            let config_prfl = config_prfl.get_ref().clone();
            let avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();
            let avatar_name_full_path = path_file_img.replace(ALIAS_AVATAR_FILES_DIR, &avatar_files_dir);
            remove_file_and_log(&avatar_name_full_path, &"delete_profile_current()");
        }

        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{borrow::Cow::Borrowed, fs, io::Write};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        body, dev,
        http::{
            header::{self, HeaderValue, CONTENT_TYPE},
            StatusCode,
        },
        test, web, App,
    };
    use chrono::{DateTime, Duration, SecondsFormat, Utc};

    use crate::extractors::authentication::BEARER;
    use crate::profiles::{
        config_prfl,
        profile_models::{self, ProfileTest},
    };
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::users::user_models::{UserRegistr, UserRole};
    use crate::users::user_registr_orm::tests::UserRegistrOrmApp;

    use super::*;

    const ADMIN: u8 = 0;
    const USER: u8 = 1;
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    // //const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";
    const MSG_MULTIPART_STREAM_INCOMPLETE: &str = "Multipart stream is incomplete";
    const MSG_CONTENT_TYPE_ERROR: &str = "Could not find Content-Type header";

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
    fn header_auth(token: &str) -> (header::HeaderName, header::HeaderValue) {
        let header_value = header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (header::AUTHORIZATION, header_value)
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
    fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
    fn save_empty_file(path_file: &str) -> Result<String, String> {
        let _ = fs::File::create(path_file).map_err(|e| e.to_string())?;
        Ok(path_file.to_string())
    }
    fn save_file_png(path_file: &str, code: u8) -> Result<(u64, String), String> {
        let header: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let footer: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
        #[rustfmt::skip]
        let buf1: Vec<u8> = vec![                            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04,  0x08, 0x06, 0x00, 0x00, 0x00, 0xA9, 0xF1, 0x9E,
            0x7E, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47,  0x42, 0x00, 0xAE, 0xCE, 0x1C, 0xE9, 0x00, 0x00,
            0x00, 0x4F, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57,  0x01, 0x44, 0x00, 0xBB, 0xFF, 0x01, 0xF3, 0xF5,
            0xF5, 0xFF, 0x3A, 0x98, 0x35, 0x00, 0xF2, 0xFE,  0xFD, 0x00, 0xD7, 0x6A, 0xCC, 0x00, 0x01, 0x05,
            0x7E, 0x09, 0xFF, 0xFD, 0x75, 0xFC, 0x00, 0x02,  0xFF, 0x02, 0x00, 0xFF, 0x95, 0xFD, 0x00, 0x01,
            0x09, 0x7B, 0x0A, 0xFF, 0xF7, 0x7B, 0xF8, 0x00,  0x00, 0x01, 0xFF, 0x00, 0x04, 0x8E, 0x03, 0x00,
            0x01, 0xF6, 0xF5, 0xF4, 0xFF, 0x13, 0x89, 0x18,  0x00, 0x02, 0x03, 0xFF, 0x00, 0xED, 0x77, 0xED,
            0x00, 0x78, 0x18, 0x1E, 0xE2, 0xBA, 0x4A, 0xF4,  0x76
        ];
        #[rustfmt::skip]
        let buf2: Vec<u8> = vec![                            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05,  0x08, 0x06, 0x00, 0x00, 0x00, 0x8D, 0x6F, 0x26,
            0xE5, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47,  0x42, 0x00, 0xAE, 0xCE, 0x1C, 0xE9, 0x00, 0x00,
            0x00, 0x74, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57,  0x01, 0x69, 0x00, 0x96, 0xFF, 0x01, 0xF3, 0xF4,
            0xF4, 0xFF, 0xFA, 0xFD, 0xF9, 0x00, 0x5A, 0xAB,  0x5A, 0x00, 0x9E, 0x54, 0x9E, 0x00, 0x10, 0x06,
            0x0E, 0x00, 0x01, 0xEB, 0xF1, 0xE5, 0xFF, 0x17,  0x02, 0x20, 0x00, 0x5B, 0xFF, 0x5F, 0x00, 0x29,
            0xFF, 0x19, 0x00, 0x4D, 0xF7, 0x56, 0x00, 0x01,  0x15, 0x7F, 0x15, 0xFF, 0xEB, 0x77, 0xEC, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x06, 0xFD, 0x05, 0x00,  0xFE, 0x94, 0x01, 0x00, 0x01, 0xE5, 0xF0, 0xE4,
            0xFF, 0x1E, 0x05, 0x1C, 0x00, 0xFD, 0x02, 0x01,  0x00, 0x05, 0xFD, 0x01, 0x00, 0xC3, 0xEE, 0xC3,
            0x00, 0x01, 0xF6, 0xF4, 0xF2, 0xFF, 0xF4, 0xFC,  0xF4, 0x00, 0x35, 0x9C, 0x3F, 0x00, 0xC1, 0x63,
            0xBA, 0x00, 0x18, 0x09, 0x19, 0x00, 0x50, 0xDE,  0x2B, 0x56, 0xC3, 0xBD, 0xEC, 0xAA,
        ];
        #[rustfmt::skip]
        let buf3: Vec<u8> = vec![                            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x17, 0x00, 0x00, 0x00, 0x13,  0x08, 0x06, 0x00, 0x00, 0x00, 0x7B, 0xBB, 0x96,
            0xB6, 0x00, 0x00, 0x00, 0x04, 0x73, 0x42, 0x49,  0x54, 0x08, 0x08, 0x08, 0x08, 0x7C, 0x08, 0x64,
            0x88, 0x00, 0x00, 0x00, 0x6C, 0x49, 0x44, 0x41,  0x54, 0x38, 0x8D, 0xED, 0x94, 0x4B, 0x0A, 0x80,
            0x30, 0x0C, 0x05, 0x5F, 0xC5, 0x83, 0x28, 0x78,  0x3D, 0xDB, 0xDE, 0xA4, 0x1F, 0x0F, 0x3C, 0xAE,
            0x15, 0x6B, 0xAB, 0xD0, 0x5D, 0x07, 0x02, 0x81,  0x90, 0x49, 0xC8, 0x22, 0x06, 0x40, 0x9D, 0x98,
            0x7A, 0x89, 0x87, 0x7C, 0xC8, 0xBF, 0x31, 0x97,  0x0A, 0x31, 0x66, 0xA5, 0x7C, 0xBC, 0x36, 0x3B,
            0xBB, 0xCB, 0x7B, 0x5B, 0xAC, 0x17, 0x37, 0xF7,  0xDE, 0xCA, 0xD9, 0xFD, 0xB7, 0x58, 0x92, 0x44,
            0x85, 0x10, 0x12, 0xCB, 0xBA, 0x5D, 0x22, 0x84,  0x54, 0x6B, 0x03, 0xA0, 0x2A, 0xBF, 0x0F, 0x68,
            0x15, 0x03, 0x14, 0x6F, 0x7E, 0x3F, 0xD1, 0x53,  0x5E, 0xC3, 0xC0, 0x78, 0x5C, 0x43, 0xDE, 0xC8,
            0x09, 0xFC, 0x22, 0xB8, 0x69, 0x88, 0xAE, 0x67,  0xA8
        ];

        // if path::Path::new(&path_file).exists() {
        //     let _ = fs::remove_file(&path_file);
        // }

        let mut file = fs::File::create(path_file).map_err(|e| e.to_string())?;
        file.write_all(header.as_ref()).map_err(|e| e.to_string())?;
        #[rustfmt::skip]
        let buf: Vec<u8> = match code { 3 => buf3, 2 => buf2, _ => buf1 };
        file.write_all(buf.as_ref()).map_err(|e| e.to_string())?;
        file.write_all(footer.as_ref()).map_err(|e| e.to_string())?;

        let size = file.metadata().unwrap().len();

        Ok((size, path_file.to_string()))
    }

    // ** put_profile **
    #[actix_web::test]
    async fn test_put_profile_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_put_profile_empty_form() {
        let (header, body) = MultiPartFormDataBuilder::new().build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_MULTIPART_STREAM_INCOMPLETE));
    }
    #[actix_web::test]
    async fn test_put_profile_form_with_invalid_name() {
        let name1_file = "test_put_profile_form_with_invalid_name.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile1", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.message, err::MSG_NO_FIELDS_TO_UPDATE);
        let key = Borrowed(validators::NM_NO_FIELDS_TO_UPDATE);
        #[rustfmt::skip]
        let names1 = app_err.params.get(&key).unwrap().get("validNames").unwrap().as_str().unwrap();
        let names2 = [ModifyProfileDto::valid_names(), vec!["avatarfile"]].concat().join(",");
        assert_eq!(names1, &names2);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_nickname_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", ProfileTest::nickname_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_NICKNAME_REGEX]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_email_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("email", ProfileTest::email_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_EMAIL_EMAIL_TYPE]);
    }
    #[actix_web::test]
    async fn test_put_profile_role_wrong() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("role", ProfileTest::role_wrong())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_USER_ROLE_INVALID_VALUE]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", ProfileTest::descript_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_descript_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", ProfileTest::descript_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("theme", ProfileTest::theme_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_THEME_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_theme_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("theme", ProfileTest::theme_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[profile_models::MSG_THEME_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_profile_invalid_file_size() {
        let name1_file = "test_put_profile_invalid_file_size.png";
        let path_name1_file = format!("./{}", &name1_file);
        let (size, _name) = save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let mut config_prfl = config_prfl::get_test_config();
        let prfl_avatar_max_size = 160;
        config_prfl.prfl_avatar_max_size = prfl_avatar_max_size;
        let cfg_c = (cfg_c.0, config_prfl);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONTENT_TOO_LARGE);
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_SIZE);
        #[rustfmt::skip]
        let json = json!({ "actualFileSize": size, "maxFileSize": prfl_avatar_max_size });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_profile_invalid_file_type() {
        let name1_file = "test_put_profile_invalid_file_type.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/bmp", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        assert_eq!(app_err.message, err::MSG_INVALID_FILE_TYPE);
        #[rustfmt::skip]
        let json = json!({ "actualFileType": "image/bmp", "validFileType": "image/jpeg,image/png" });
        assert_eq!(*app_err.params.get("invalidFileType").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_profile_valid_data_without_file() {
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let profile = data_c.0.get(0).unwrap().clone();
        let nickname_s = format!("{}_a", profile.nickname.clone());
        let email_s = format!("{}_a", profile.email.clone());
        let user_role = UserRole::Admin;
        let descript_s = format!("{}_a", profile.descript.clone());
        let theme_s = format!("{}_a", profile.theme.clone());

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("nickname", &nickname_s)
            .with_text("email", &email_s)
            .with_text("role", &user_role.to_string())
            .with_text("descript", &descript_s)
            .with_text("theme", &theme_s)
            .build();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(profile_dto_res.id, profile.user_id);
        assert_eq!(profile_dto_res.nickname, nickname_s);
        assert_eq!(profile_dto_res.email, email_s);
        assert_eq!(profile_dto_res.role, user_role);
        assert_eq!(profile_dto_res.descript, descript_s);
        assert_eq!(profile_dto_res.theme, theme_s);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = profile_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_created_at = profile.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, old_created_at);
        let res_updated_at = profile_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_updated_at = profile.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, old_updated_at);
    }
    #[actix_web::test]
    async fn test_put_profile_a_with_old0_new1() {
        let name1_file = "test_put_profile_a_with_old0_new1.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let prfl_avatar_files_dir = cfg_c.1.prfl_avatar_files_dir.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_b_with_old0_new1_convert() {
        let name1_file = "test_put_profile_b_with_old0_new1_convert.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 3).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let profile1_id = data_c.0.get(0).unwrap().user_id;
        let mut config_prfl = cfg_c.1.clone();
        let file_ext = "jpeg".to_string();
        config_prfl.prfl_avatar_ext = Some(file_ext.clone());
        config_prfl.prfl_avatar_max_width = 18;
        config_prfl.prfl_avatar_max_height = 18;
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();

        let cfg_c = (cfg_c.0, config_prfl);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let path = path::Path::new(&img_name_full_path);
        let receiver_ext = path.extension().map(|s| s.to_str().unwrap().to_string()).unwrap();
        let is_exists_img_new = path.exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert_eq!(file_ext, receiver_ext);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_c_with_old1_new1() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_c_with_old1_new1.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_c_with_old1_new1_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        let now = Utc::now();
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        let img_name_full_path = profile_dto_res_img.replacen(ALIAS_AVATAR_FILES_DIR, &prfl_avatar_files_dir, 1);
        let is_exists_img_new = path::Path::new(&img_name_full_path).exists();
        let _ = fs::remove_file(&img_name_full_path);
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
        assert!(is_exists_img_new);

        let path_img = path::PathBuf::from(profile_dto_res_img);
        let file_stem = path_img.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, profile1_id.to_string());

        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_profile_d_with_old1_new0() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_d_with_old1_new0.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&path_name0_file, 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", "descript1".to_string())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(path_name0_file.clone());
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let profile_dto_res_img = profile_dto_res.avatar.unwrap_or("".to_string());
        assert!(profile_dto_res_img.len() > 0);
        assert!(profile_dto_res_img.starts_with(ALIAS_AVATAR_FILES_DIR));
        assert_eq!(&path_name0_alias, &profile_dto_res_img);
    }
    #[actix_web::test]
    async fn test_put_profile_e_with_old_1_new_size0() {
        let prfl_avatar_files_dir = config_prfl::get_test_config().prfl_avatar_files_dir;

        let name0_file = "test_put_profile_e_with_old_1_new_size0.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let name1_file = "test_put_profile_e_with_old_1_new_size0_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);

        let mut profile1 = data_c.0.get(0).unwrap().clone();
        profile1.avatar = Some(path_name0_alias.clone());
        let data_c = (vec![profile1], data_c.1, data_c.2);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
    }
    #[actix_web::test]
    async fn test_put_profile_f_with_old0_new_size0() {
        let name1_file = "test_put_profile_f_with_old0_new_size0.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "avatarfile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data(false, USER);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/profiles")
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(profile_dto_res.avatar.is_none());
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
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

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
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

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
        assert_eq!(resp.status(), StatusCode::CONFLICT); // 409

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
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // 401

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
        assert_eq!(resp.status(), StatusCode::OK); // 200

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
    async fn test_delete_profile_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile_id_bad = format!("{}a", data_c.0.get(0).unwrap().user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

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
    async fn test_delete_profile_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile_id = data_c.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_profile_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }

    // ** delete_profile_current **
    #[actix_web::test]
    async fn test_delete_profile_current_without_img() {
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
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_current_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir;

        let name0_file = "test_delete_profile_current_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data(false, ADMIN);
        let profile1 = data_c.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current).configure(configure_profile(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
}

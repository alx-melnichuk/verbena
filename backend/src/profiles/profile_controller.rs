use std::{borrow::Cow, ops::Deref, path};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use mime::IMAGE;
use serde_json::json;
use utoipa;
use vrb_tools::{hash_tools, parser};

use crate::cdis::coding;
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::loading::dynamic_image;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    config_prfl::{self, ConfigPrfl},
    profile_checks,
    profile_models::{
        ModifyProfile, ModifyProfileDto, NewPasswordProfileDto, Profile, ProfileConfigDto, ProfileDto, UniquenessProfileDto,
        UniquenessProfileResponseDto, PROFILE_LOCALE_DEF, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
use crate::settings::err;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm,
    stream_extra::{get_stream_logo_files, remove_stream_logo_files},
};
use crate::users::user_models::UserRole;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
use crate::validators::{self, msg_validation, ValidationChecks, Validator};

pub const ALIAS_AVATAR_FILES_DIR: &str = "/avatar";

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
            .service(uniqueness_check)
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

fn remove_image_file(path_file_img: &str, alias_file_img: &str, img_file_dir: &str, err_msg: &str) {
    // If the image file name starts with the specified alias, then delete the file.
    if path_file_img.len() > 0 && (alias_file_img.len() == 0 || path_file_img.starts_with(alias_file_img)) {
        let avatar_name_full_path = path_file_img.replace(alias_file_img, img_file_dir);
        remove_file_and_log(&avatar_name_full_path, err_msg);
    }
}
fn remove_file_and_log(file_name: &str, msg: &str) {
    if file_name.len() > 0 {
        let res_remove = std::fs::remove_file(file_name);
        if let Err(err) = res_remove {
            error!("{} remove_file({}): error: {:?}", msg, file_name, err);
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

// ** Section: get_profile_by_id **

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
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK), 
                    Some(PROFILE_LOCALE_DEF)))
            )))),
        ),
        (status = 204, description = "The user with the specified ID was not found."),
        (status = 401, description = "An authorization token is required.", body = AppError,
            example = json!(AppError::unauthorized401(err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X GET http://localhost:8080/api/users/2a`", 
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
#[get("/api/profiles/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())" )]
pub async fn get_profile_by_id(
    profile_orm: web::Data<ProfileOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let id_str = request.match_info().query("id").to_string();

    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;

    let opt_profile = web::block(move || {
        // Find profile by user id.
        let profile =
            profile_orm.get_profile_user_by_id(id, false).map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            }).ok()?;

        profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })?;

    if let Some(profile_user) = opt_profile {
        let profile_dto = ProfileDto::from(profile_user);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

// ** Section: profiles_config **

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
    let max_width = if cfg_prfl.prfl_avatar_max_width > 0 {
        Some(cfg_prfl.prfl_avatar_max_width)
    } else {
        None
    };
    let max_height = if cfg_prfl.prfl_avatar_max_height > 0 {
        Some(cfg_prfl.prfl_avatar_max_height)
    } else {
        None
    };
    // Get configuration data.
    let profile_config_dto = ProfileConfigDto::new(max_size, valid_types, ext, max_width, max_height);

    Ok(HttpResponse::Ok().json(profile_config_dto)) // 200
}

// ** Section: get_profile_current **

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
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK),
                    Some(PROFILE_LOCALE_DEF)))
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

// ** Section: uniqueness_check **

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
            error!("{}:{}; {}", &err.code, &err.message, &prm1);
            err
        })?;
    let uniqueness = res_search.is_none();

    let response_dto = UniquenessProfileResponseDto::new(uniqueness);

    Ok(HttpResponse::Ok().json(response_dto)) // 200
}

// ** Section: put_profiles **

#[derive(Debug, MultipartForm)]
pub struct ModifyProfileForm {
    pub nickname: Option<Text<String>>,
    pub email: Option<Text<String>>,
    pub role: Option<Text<String>>,
    pub descript: Option<Text<String>>,
    pub theme: Option<Text<String>>,
    pub locale: Option<Text<String>>,
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
                locale: modify_profile_form.locale.map(|v| v.into_inner()),
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
///   theme?: String,        // optional - default color theme. ('light','dark');
///   locale?: String,       // optional - default locale;
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
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK), 
                    Some(PROFILE_LOCALE_DEF)))
            )))),    
        ),
        (status = 204, description = "The current user's profile was not found."),
        (status = 409, description = "Error: nickname (email) is already in use.", body = AppError, examples(
            ("Nickname" = (summary = "Nickname already used",
                description = "The nickname value has already been used.",
                value = json!(AppError::conflict409(err::MSG_NICKNAME_ALREADY_USE)))),
            ("Email" = (summary = "Email already used", 
                description = "The email value has already been used.",
                value = json!(AppError::conflict409(err::MSG_EMAIL_ALREADY_USE))))
        )),
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
                    descript: Some("d".to_string()), theme: Some("light".to_string()), locale: None }).validate().err().unwrap()
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
    user_registr_orm: web::Data<UserRegistrOrmApp>,
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
            error!("{}: {}", err::CD_VALIDATION, msg_validation(&errors));
            return Ok(AppError::to_response(&AppError::validations(errors))); // 417
        }
    }

    let opt_nickname = modify_profile_dto.nickname.clone();
    let opt_email = modify_profile_dto.email.clone();
    if opt_nickname.is_some() || opt_email.is_some() {
        let profile_orm2 = profile_orm.get_ref().clone();
        let registr_orm2 = user_registr_orm.get_ref().clone();

        let res_search = profile_checks::uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm2, registr_orm2)
            .await
            .map_err(|err| {
                #[rustfmt::skip]
                let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
                error!("{}:{}; {}", &err.code, &err.message, &prm1);
                err
            })?;

        // Since the specified "nickname" or "email" is not unique, return an error.
        if let Some((is_nickname, _)) = res_search {
            #[rustfmt::skip]
            let message = if is_nickname { err::MSG_NICKNAME_ALREADY_USE } else { err::MSG_EMAIL_ALREADY_USE };
            error!("{}: {}", err::CD_CONFLICT, &message);
            return Err(AppError::conflict409(&message)); // 409
        }
    }

    let mut avatar: Option<Option<String>> = None;

    let config_prfl = config_prfl.get_ref().clone();
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
            error!("{}: {}; {}", err::CD_CONTENT_TOO_LARGE, err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(AppError::content_large413(err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }
        
        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types: Vec<String> = config_prfl.prfl_avatar_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            error!("{}: {}; {}", err::CD_UNSUPPORTED_TYPE, err::MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(AppError::unsupported_type415(err::MSG_INVALID_FILE_TYPE) // 415
                .add_param(Cow::Borrowed("invalidFileType"), &json));
        }

        // Get the file stem and extension for the new file.
        #[rustfmt::skip]
        let name = format!("{}.{}", get_file_name(curr_user_id, Utc::now()), file_mime_type.replace(&format!("{}/", IMAGE), ""));
        // Add 'file path' + 'file name'.'file extension'.
        let path: path::PathBuf = [&config_prfl.prfl_avatar_files_dir, &name].iter().collect();
        let full_path_file = path.to_str().unwrap().to_string();
        // Persist the temporary file at the target path.
        // Note: if a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            let message = format!("{}; {} - {}", err::MSG_ERROR_UPLOAD_FILE, &full_path_file, err.to_string());
            error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
            return Err(AppError::internal_err500(&message)); // 500
        }
        path_new_avatar_file = full_path_file;
        
        // Convert the file to another mime type.
        let res_convert_img_file = convert_avatar_file(&path_new_avatar_file, config_prfl.clone(), "put_profile()")
        .map_err(|e| {
            let message = format!("{}; {}", err::MSG_ERROR_CONVERT_FILE, e);
            error!("{}: {}", err::CD_NOT_EXTENDED, &message);
            AppError::not_extended510(&message) // 510
        })?;
        if let Some(new_path_file) = res_convert_img_file {
            path_new_avatar_file = new_path_file;
        }
    
        let alias_avatar_file = path_new_avatar_file.replace(&config_prfl.prfl_avatar_files_dir, ALIAS_AVATAR_FILES_DIR);
        avatar = Some(Some(alias_avatar_file));

        break;
    }
    let mut modify_profile: ModifyProfile = modify_profile_dto.into();
    modify_profile.avatar = avatar;

    let path_old_avatar_file = if modify_profile.avatar.is_some() { 
        opt_curr_avatar.unwrap_or("".to_string())
    } else {
        "".to_string()
    };

    let res_profile = web::block(move || {
        // Modify an entity (profile).
        let res_data_profile = profile_orm.modify_profile(curr_user_id, modify_profile)
        .map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });

        res_data_profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_profile = res_profile
    .map_err(|err| {
        remove_file_and_log(&path_new_avatar_file, &"put_profile()");
        err
    })?;

    if let Some(profile) = opt_profile {
        // If the image file name starts with the specified alias, then delete the file.
        remove_image_file(&path_old_avatar_file, ALIAS_AVATAR_FILES_DIR, &config_prfl.prfl_avatar_files_dir, &"put_profile()");
        let profile_dto = ProfileDto::from(profile);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        remove_file_and_log(&path_new_avatar_file, &"put_profile()");
        Ok(HttpResponse::NoContent().finish()) // 204        
    }
}

// ** Section: put_profile_new_password **

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
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK),
                Some(PROFILE_LOCALE_DEF)))
            )))),
        ),
        (status = 204, description = "The current user was not found."),
        (status = 401, description = "The nickname or password is incorrect or the token is missing.", body = AppError, 
            example = json!(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT))),
        (status = 403, description = "Access denied: insufficient user rights.", body = AppError,
            example = json!(AppError::forbidden403(err::MSG_ACCESS_DENIED))),
        (status = 409, description = "Error when comparing password hashes.", body = AppError,
            example = json!(AppError::conflict409(&format!("{}; {}", err::MSG_INVALID_HASH, "Parameter is empty.")))),
        (status = 417, body = [AppError],
            description = "Validation error. `curl -i -X PUT http://localhost:8080/api/profiles_new_password \
            -d '{\"password\": \"pas\" \"new_password\": \"word\"}'`",
            example = json!(AppError::validations(
                (NewPasswordProfileDto {password: "pas".to_string(), new_password: "word".to_string()}).validate().err().unwrap()) )),
        (status = 500, description = "Error while calculating the password hash.", body = AppError, 
            example = json!(AppError::internal_err500(&format!("{}; {}", err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty.")))),
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
        error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }

    let new_password_user: NewPasswordProfileDto = json_body.into_inner();
    let new_password = new_password_user.new_password.clone();
    // Get a hash of the new password.
    let new_password_hashed = hash_tools::encode_hash(&new_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_ERROR_HASHING_PASSWORD, e.to_string());
        error!("{}: {}", err::CD_INTERNAL_ERROR, &message);
        AppError::internal_err500(&message) // 500
    })?;
    
    let profile_orm2 = profile_orm.clone();
    let opt_profile_pwd = web::block(move || {
        // Find user by nickname or email.
        let existing_profile = profile_orm2.get_profile_user_by_id(profile_id, true)
            .map_err(|e| {
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e) // 507
            });
        existing_profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    let profile_pwd = opt_profile_pwd.ok_or_else(|| {
        error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_WRONG_NICKNAME_EMAIL);
        AppError::unauthorized401(err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
    })?;

    // Get the value of the old password.
    let old_password = new_password_user.password.clone();
    // Get a hash of the old password.
    let profile_hashed_old_password = profile_pwd.password.to_string();
    // Check whether the hash for the specified password value matches the old password hash.
    let password_matches = hash_tools::compare_hash(&old_password, &profile_hashed_old_password).map_err(|e| {
        let message = format!("{}; {}", err::MSG_INVALID_HASH, &e);
        error!("{}: {}", err::CD_CONFLICT, &message);
        AppError::conflict409(&message) // 409
    })?;
    // If the hash for the specified password does not match the old password hash, then return an error.
    if !password_matches {
        error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_PASSWORD_INCORRECT);
        return Err(AppError::unauthorized401(err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    // Set a new user password.

    // Create a model to update the "password" field in the user profile.
    let modify_profile = ModifyProfile{
        nickname: None, email: None, password: Some(new_password_hashed), role: None, avatar: None, descript: None, theme: None,
        locale: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(profile_id, modify_profile)
        .map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        opt_profile1
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
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

// ** Section: delete_profile **

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
/// This will delete the user's image file. All user streams are also deleted, including stream logo files.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified user profile was deleted successfully.", body = ProfileDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileDto::from(
                Profile::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK), 
                    Some(PROFILE_LOCALE_DEF)))
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
    config_strm: web::Data<config_strm::ConfigStrm>,
    profile_orm: web::Data<ProfileOrmApp>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}; `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        error!("{}: {}", err::CD_RANGE_NOT_SATISFIABLE, &message);
        AppError::range_not_satisfiable416(&message) // 416
    })?;

    // Get a list of logo file names for streams of the user with the specified user_id.
    let path_file_img_list: Vec<String> = get_stream_logo_files(stream_orm, id).await?;

    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(id)
        .map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) // 506
    })??;

    if let Some(profile) = opt_profile {
        let config_prfl = config_prfl.get_ref().clone();
        // Get the path to the "avatar" file.
        let path_file_img = profile.avatar.clone().unwrap_or("".to_string());
        // If the image file name starts with the specified alias, then delete the file.
        remove_image_file(&path_file_img, ALIAS_AVATAR_FILES_DIR, &config_prfl.prfl_avatar_files_dir, &"delete_profile()");
        // Delete all specified logo files in the given list.
        let _ = remove_stream_logo_files(path_file_img_list, config_strm.get_ref().clone());

        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

// ** Section: delete_profile_current **

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
/// This will delete the user's image file. All user streams are also deleted, including stream logo files.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "The current user's profile has been successfully deleted.", body = ProfileDto,
            examples(
            ("with_avatar" = (summary = "with an avatar", description = "User profile with avatar.",
                value = json!(ProfileDto::from(
                Profile::new(1, "Emma_Johnson", "Emma_Johnson@gmail.us", UserRole::User, Some("/avatar/1234151234.png"),
                    Some("Description Emma_Johnson"), Some(PROFILE_THEME_LIGHT_DEF), Some(PROFILE_LOCALE_DEF)))
            ))),
            ("without_avatar" = (summary = "without avatar", description = "User profile without avatar.",
                value = json!(ProfileDto::from(
                Profile::new(2, "James_Miller", "James_Miller@gmail.us", UserRole::User, None, None, Some(PROFILE_THEME_DARK), 
                    Some(PROFILE_LOCALE_DEF)))
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
    config_strm: web::Data<config_strm::ConfigStrm>,
    profile_orm: web::Data<ProfileOrmApp>,
    stream_orm: web::Data<StreamOrmApp>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let profile = authenticated.deref();
    let id = profile.user_id;
    
    // Get a list of logo file names for streams of the user with the specified user_id.
    let path_file_img_list: Vec<String> = get_stream_logo_files(stream_orm, id).await?;

    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(id)
        .map_err(|e| {
            error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e) // 507
        });
        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string()) //506
    })??;

    if let Some(profile) = opt_profile {
        let config_prfl = config_prfl.get_ref().clone();
        // Get the path to the "avatar" file.
        let path_file_img = profile.avatar.clone().unwrap_or("".to_string());
        // If the image file name starts with the specified alias, then delete the file.
        remove_image_file(&path_file_img, ALIAS_AVATAR_FILES_DIR, &config_prfl.prfl_avatar_files_dir, &"delete_profile_current()");
        // Delete all specified logo files in the given list.
        let _ = remove_stream_logo_files(path_file_img_list, config_strm.get_ref().clone());

        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{fs, io::Write};

    use actix_web::{http::header, web};
    use chrono::{DateTime, Duration, Utc};
    use vrb_tools::{hash_tools, token::BEARER};
    
    use crate::errors::AppError;
    use crate::profiles::{config_prfl, profile_models::Profile, profile_orm::tests::ProfileOrmApp};
    use crate::sessions::{config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token};
    use crate::streams::{
        config_strm, stream_controller::tests::create_stream, stream_models::StreamInfoDto, stream_orm::tests::StreamOrmApp,
    };
    use crate::users::user_models::{UserRegistr, UserRole};
    use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
    
    pub const ADMIN: u8 = 0;
    pub const USER: u8 = 1;
    pub const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    pub const MSG_MULTIPART_STREAM_INCOMPLETE: &str = "Multipart stream is incomplete";
    pub const MSG_CONTENT_TYPE_ERROR: &str = "Could not find Content-Type header";

    pub fn create_profile(role: u8, opt_password: Option<&str>) -> Profile {
        let nickname = "Oliver_Taylor".to_string();
        let role = if role == ADMIN { UserRole::Admin } else { UserRole::User };
        let mut profile = ProfileOrmApp::new_profile(1, &nickname, &format!("{}@gmail.com", &nickname), role);
        if let Some(password) = opt_password {
            profile.password = hash_tools::encode_hash(password.to_string()).unwrap();
            // hashed
        }
        profile
    }
    fn profile_with_id(profile: Profile) -> Profile {
        let profile_orm = ProfileOrmApp::create(&vec![profile]);
        profile_orm.profile_vec.get(0).unwrap().clone()
    }
    fn create_user_registr() -> UserRegistr {
        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);

        let user_registr = UserRegistrOrmApp::new_user_registr(1, "Robert_Brown", "Robert_Brown@gmail.com", "passwdR2B2", final_date);
        user_registr
    }
    fn user_registr_with_id(user_registr: UserRegistr) -> UserRegistr {
        let user_reg_orm = UserRegistrOrmApp::create(&vec![user_registr]);
        user_reg_orm.user_registr_vec.get(0).unwrap().clone()
    }
    pub fn header_auth(token: &str) -> (header::HeaderName, header::HeaderValue) {
        let header_value = header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (header::AUTHORIZATION, header_value)
    }
    #[rustfmt::skip]
    pub fn get_cfg_data(is_registr: bool, role: u8) -> (
        (config_jwt::ConfigJwt, config_prfl::ConfigPrfl, config_strm::ConfigStrm), // configuration
        (Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<StreamInfoDto>) // data vectors
        , String) {
        // Create profile values.
        let profile1: Profile = profile_with_id(create_profile(role, None));
        let num_token = 1234;
        let session1 = SessionOrmApp::new_session(profile1.user_id, Some(num_token));

        let stream1 = create_stream(0, profile1.user_id, "title0", "tag01,tag02", Utc::now());

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(profile1.user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let user_registr_vec:Vec<UserRegistr> = if is_registr {
            vec![user_registr_with_id(create_user_registr())]
        } else { vec![] };

        let config_prfl = config_prfl::get_test_config();
        let config_strm = config_strm::get_test_config();
        let cfg_c = (config_jwt, config_prfl, config_strm);
        let data_c = (vec![profile1], vec![session1], user_registr_vec, vec![stream1]);

        (cfg_c, data_c, token)
    }
    pub fn configure_profile(
        cfg_c: (config_jwt::ConfigJwt, config_prfl::ConfigPrfl, config_strm::ConfigStrm), // configuration
        data_c: (Vec<Profile>, Vec<Session>, Vec<UserRegistr>, Vec<StreamInfoDto>),       // data vectors
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(cfg_c.0);
            let data_config_prfl = web::Data::new(cfg_c.1);
            let data_config_strm = web::Data::new(cfg_c.2);

            let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_user_registr_orm = web::Data::new(UserRegistrOrmApp::create(&data_c.2));
            let data_stream_orm = web::Data::new(StreamOrmApp::create(&data_c.3));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_prfl))
                .app_data(web::Data::clone(&data_config_strm))
                .app_data(web::Data::clone(&data_profile_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_user_registr_orm))
                .app_data(web::Data::clone(&data_stream_orm));
        }
    }
    pub fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
    pub fn save_empty_file(path_file: &str) -> Result<String, String> {
        let _ = fs::File::create(path_file).map_err(|e| e.to_string())?;
        Ok(path_file.to_string())
    }
    pub fn save_file_png(path_file: &str, code: u8) -> Result<(u64, String), String> {
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
}

use std::{borrow::Cow, env, ops::Deref, path};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, http::StatusCode, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use mime::IMAGE;
use serde_json::json;
use utoipa;
use vrb_authent::authentication::{Authenticated, RequireAuth};
use vrb_common::{
    api_error::{code_to_str, ApiError},
    parser,
    validators::{self, msg_validation, ValidationChecks, Validator},
};
use vrb_dbase::db_enums::UserRole;
use vrb_tools::{cdis::coding, consts, err, hash_tools, loading::dynamic_image};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{
    config_prfl::{self, ConfigPrfl},
    profile_check,
    profile_models::{
        ModifyProfile, ModifyProfileDto, NewPasswordProfileDto, Profile, ProfileConfigDto, ProfileDto, UniquenessProfileDto,
        UniquenessProfileResponseDto, PROFILE_LOCALE_DEF, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF,
    },
    profile_orm::ProfileOrm,
};
#[cfg(not(all(test, feature = "mockdata")))]
use crate::users::user_registr_orm::impls::UserRegistrOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;

pub const ALIAS_AVATAR_FILES_DIR: &str = consts::ALIAS_AVATAR_FILES_DIR;
pub const ALIAS_LOGO_FILES_DIR2: &str = consts::ALIAS_LOGO_FILES_DIR;

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

/** Delete a file if its path starts with the specified alias. */
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
fn get_logo_files_dir() -> String {
    // Directory for storing logo files.
    let logo_files_dir = env::var("STRM_LOGO_FILES_DIR").unwrap_or(consts::LOGO_FILES_DIR.to_string());
    let path_dir: path::PathBuf = path::PathBuf::from(logo_files_dir).iter().collect();
    path_dir.to_str().unwrap().to_string()
}
/** Delete all specified logo files in the given list. */
fn remove_stream_logo_files(path_file_img_list: Vec<String>, img_file_dir: String) -> usize {
    let result = path_file_img_list.len();
    // Remove files from the resulting list of stream logo files.
    for path_file_img in path_file_img_list {
        // Delete a file if its path starts with the specified alias.
        remove_image_file(&path_file_img, ALIAS_LOGO_FILES_DIR2, &img_file_dir, &"remove_stream_logo_files()");
    }
    result
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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X GET http://localhost:8080/api/users/2a`", 
            body = ApiError, example = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                , "`id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    params(("id", description = "Unique user ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[get("/api/profiles/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())" )]
pub async fn get_profile_by_id(
    profile_orm: web::Data<ProfileOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, ApiError> {
    let id_str = request.match_info().query("id").to_string();
    #[rustfmt::skip]
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = &format!("`{}` - {}", "id", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;

    let opt_profile = web::block(move || {
        // Find profile by user id.
        let profile =
            profile_orm.get_profile_user_by_id(id, false).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            }).ok()?;

        profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/profiles_config", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_profile_config(config_prfl: web::Data<ConfigPrfl>) -> actix_web::Result<HttpResponse, ApiError> {
    let cfg_prfl = config_prfl;
    #[rustfmt::skip]
    let max_size = if cfg_prfl.prfl_avatar_max_size > 0 { Some(cfg_prfl.prfl_avatar_max_size) } else { None };
    let valid_types = cfg_prfl.prfl_avatar_valid_types.clone();
    let ext = cfg_prfl.prfl_avatar_ext.clone();
    #[rustfmt::skip]
    let max_width = if cfg_prfl.prfl_avatar_max_width > 0 { Some(cfg_prfl.prfl_avatar_max_width) } else { None };
    #[rustfmt::skip]
    let max_height = if cfg_prfl.prfl_avatar_max_height > 0 { Some(cfg_prfl.prfl_avatar_max_height) } else { None };
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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[get("/api/profiles_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_profile_current(
    authenticated: Authenticated,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let user = authenticated.deref();
    let user_id = user.id;

    let opt_profile = web::block(move || {
        // Find profile by user id.
        let profile =
            profile_orm.get_profile_user_by_id(user_id, false).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            }).ok()?;

        profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })?;

    if let Some(profile_user) = opt_profile {
        let profile_dto = ProfileDto::from(profile_user);
        Ok(HttpResponse::Ok().json(profile_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
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
        (status = 406, description = "None of the parameters are specified.", body = ApiError,
            example = json!(ApiError::new(406, err::MSG_PARAMS_NOT_SPECIFIED)
                .add_param(Cow::Borrowed("invalidParams"), &json!({ "nickname": "null", "email": "null" })))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
)]
#[get("/api/profiles_uniqueness")]
pub async fn uniqueness_check(
    profile_orm: web::Data<ProfileOrmApp>,
    user_registr_orm: web::Data<UserRegistrOrmApp>,
    query_params: web::Query<UniquenessProfileDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get search parameters.
    let uniqueness_user_dto: UniquenessProfileDto = query_params.clone().into_inner();

    let opt_nickname = uniqueness_user_dto.nickname.clone();
    let opt_email = uniqueness_user_dto.email.clone();

    let profile_orm = profile_orm.get_ref().clone();
    let registr_orm = user_registr_orm.get_ref().clone();

    let res_search = profile_check::uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, registr_orm)
        .await
        .map_err(|err| {
            #[rustfmt::skip]
            let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
            error!("{}-{}; {}", &err.code, &err.message, &prm1);
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
        (status = 409, description = "Error: nickname (email) is already in use.", body = ApiError, examples(
            ("Nickname" = (summary = "Nickname already used",
                description = "The nickname value has already been used.",
                value = json!(ApiError::new(409, err::MSG_NICKNAME_ALREADY_USE)))),
            ("Email" = (summary = "Email already used", 
                description = "The email value has already been used.",
                value = json!(ApiError::new(409, err::MSG_EMAIL_ALREADY_USE))))
        )),
        (status = 413, description = "Invalid image file size. `curl -i -X PUT http://localhost:8080/api/profiles
            -F 'avatarfile=@image.jpg'`", body = ApiError,
            example = json!(ApiError::new(413, err::MSG_INVALID_FILE_SIZE).add_param(Cow::Borrowed("invalidFileSize"),
                &json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X PUT http://localhost:8080/api/profiles
            -F 'avatarfile=@image.svg'`", body = ApiError,
            example = json!(ApiError::new(415, err::MSG_INVALID_FILE_TYPE).add_param(Cow::Borrowed("invalidFileType"),
                &json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 417, description = "Validation error. `curl -X PUT http://localhost:8080/api/profiles
            -F 'descript=Description' -F 'theme=light' -F 'avatarfile=@image.png'`", body = [ApiError],
            example = json!(ApiError::validations(
                (ModifyProfileDto { nickname: None, email: None, role: None,
                    descript: Some("d".to_string()), theme: Some("light".to_string()), locale: None }).validate().err().unwrap()
            ) )),
        (status = 500, description = "Error loading file.", body = ApiError, example = json!(
            ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg - File not found."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = ApiError,
            example = json!(ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\""))),
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
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get the current user's profile.
    let user = authenticated.deref();
    let curr_user_id = user.id;
    // #let opt_curr_avatar = user.avatar.clone();

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
            error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&errors));
            return Ok(ApiError::to_response(&ApiError::validations(errors))); // 417
        }
    }

    let opt_nickname = modify_profile_dto.nickname.clone();
    let opt_email = modify_profile_dto.email.clone();
    if opt_nickname.is_some() || opt_email.is_some() {
        let profile_orm2 = profile_orm.get_ref().clone();
        let registr_orm2 = user_registr_orm.get_ref().clone();

        let res_search = profile_check::uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm2, registr_orm2)
            .await
            .map_err(|err| {
                #[rustfmt::skip]
                let prm1 = match err.params.first_key_value() { Some((_, v)) => v.to_string(), None => "".to_string() };
                error!("{}-{}; {}", &err.code, &err.message, &prm1);
                err
            })?;

        // Since the specified "nickname" or "email" is not unique, return an error.
        if let Some((is_nickname, _)) = res_search {
            #[rustfmt::skip]
            let message = if is_nickname { err::MSG_NICKNAME_ALREADY_USE } else { err::MSG_EMAIL_ALREADY_USE };
            error!("{}-{}", code_to_str(StatusCode::CONFLICT), &message);
            return Err(ApiError::new(409, &message)); // 409
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
            #[rustfmt::skip]
            error!("{}-{}: {}", code_to_str(StatusCode::PAYLOAD_TOO_LARGE), err::MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(ApiError::new(413, err::MSG_INVALID_FILE_SIZE) // 413
                .add_param(Cow::Borrowed("invalidFileSize"), &json));
        }
        
        // Checking the mime type file for valid mime types.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_mime_types: Vec<String> = config_prfl.prfl_avatar_valid_types.clone();
        if !valid_file_mime_types.contains(&file_mime_type) {
            let json = json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_mime_types.join(",") });
            #[rustfmt::skip]
            error!("{}-{}; {}", code_to_str(StatusCode::UNSUPPORTED_MEDIA_TYPE), err::MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(ApiError::new(415, err::MSG_INVALID_FILE_TYPE) // 415
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
            let msg = format!("{} - {}", &full_path_file, err.to_string());
            error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_UPLOAD_FILE, &msg);
            return Err(ApiError::create(500, err::MSG_ERROR_UPLOAD_FILE, &msg)); // 500
        }
        path_new_avatar_file = full_path_file;
        
        // Convert the file to another mime type.
        let res_convert_img_file = convert_avatar_file(&path_new_avatar_file, config_prfl.clone(), "put_profile()")
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::NOT_EXTENDED), err::MSG_ERROR_CONVERT_FILE, &e);
            ApiError::create(510, err::MSG_ERROR_CONVERT_FILE, &e) // 510
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

    let mut opt_curr_avatar: Option<String> = None;
    if modify_profile.avatar.is_some() {
        let profile_orm2 = profile_orm.get_ref().clone();
        // Get the current value of the 'avatar' field.
        let res_curr_profile = web::block(move || {
            // Modify an entity (profile).
            let res_data_curr_profile = profile_orm2.get_profile_user_by_id(curr_user_id, false)
            //  modify_profile(curr_user_id, modify_profile)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e)
            });
    
            res_data_curr_profile
        })
        .await
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
            ApiError::create(506, err::MSG_BLOCKING, &e.to_string())
        })?;
    
        if let Ok(Some(curr_profile)) = res_curr_profile {
            opt_curr_avatar = curr_profile.avatar;
        }
    }

    let path_old_avatar_file = if modify_profile.avatar.is_some() { 
        opt_curr_avatar.unwrap_or("".to_string())
    } else {
        "".to_string()
    };

    let res_profile = web::block(move || {
        // Modify an entity (profile).
        let res_data_profile = profile_orm.modify_profile(curr_user_id, modify_profile)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e)
        });

        res_data_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string())
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
        (status = 401, description = "The nickname or password is incorrect or the token is missing.", body = ApiError, 
            example = json!(ApiError::new(401, err::MSG_PASSWORD_INCORRECT))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
        (status = 409, description = "Error when comparing password hashes.", body = ApiError,
            example = json!(ApiError::create(409, err::MSG_INVALID_HASH, "Parameter is empty."))),
        (status = 417, body = [ApiError],
            description = "Validation error. `curl -i -X PUT http://localhost:8080/api/profiles_new_password \
            -d '{\"password\": \"pas\" \"new_password\": \"word\"}'`",
            example = json!(ApiError::validations(
                (NewPasswordProfileDto {password: "pas".to_string(), new_password: "word".to_string()}).validate().err().unwrap()) )),
        (status = 500, description = "Error while calculating the password hash.", body = ApiError, 
            example = json!(ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, "Parameter is empty."))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = []))
)]
#[rustfmt::skip]
#[put("/api/profiles_new_password", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_profile_new_password(
    authenticated: Authenticated,
    profile_orm: web::Data<ProfileOrmApp>,
    json_body: web::Json<NewPasswordProfileDto>,
) -> actix_web::Result<HttpResponse, ApiError> {
    let user = authenticated.deref();
    let user_id = user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        error!("{}-{}", code_to_str(StatusCode::EXPECTATION_FAILED), msg_validation(&validation_errors));
        return Ok(ApiError::to_response(&ApiError::validations(validation_errors))); // 417
    }

    let new_password_user: NewPasswordProfileDto = json_body.into_inner();
    let new_password = new_password_user.new_password.clone();
    // Get a hash of the new password.
    let new_password_hashed = hash_tools::encode_hash(&new_password).map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::INTERNAL_SERVER_ERROR), err::MSG_ERROR_HASHING_PASSWORD, &e.to_string());
        ApiError::create(500, err::MSG_ERROR_HASHING_PASSWORD, &e.to_string()) // 500
    })?;
    
    let profile_orm2 = profile_orm.clone();
    let opt_profile_pwd = web::block(move || {
        // Find user by nickname or email.
        let existing_profile = profile_orm2.get_profile_user_by_id(user_id, true)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
        existing_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    let profile_pwd = opt_profile_pwd.ok_or_else(|| {
        error!("{}-{}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_WRONG_NICKNAME_EMAIL);
        ApiError::new(401, err::MSG_WRONG_NICKNAME_EMAIL) // 401 (A)
    })?;

    // Get the value of the old password.
    let old_password = new_password_user.password.clone();
    // Get a hash of the old password.
    let profile_hashed_old_password = profile_pwd.password.to_string();
    // Check whether the hash for the specified password value matches the old password hash.
    let password_matches = hash_tools::compare_hash(&old_password, &profile_hashed_old_password).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::CONFLICT), err::MSG_INVALID_HASH, &e);
        ApiError::create(409, err::MSG_INVALID_HASH, &e) // 409
    })?;
    // If the hash for the specified password does not match the old password hash, then return an error.
    if !password_matches {
        error!("{}-{}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_PASSWORD_INCORRECT);
        return Err(ApiError::new(401, err::MSG_PASSWORD_INCORRECT)); // 401 (B)
    }

    // Set a new user password.

    // Create a model to update the "password" field in the user profile.
    let modify_profile = ModifyProfile{
        nickname: None, email: None, password: Some(new_password_hashed), role: None, avatar: None, descript: None, theme: None,
        locale: None,
    };
    // Update the password hash for the user profile.
    let opt_profile = web::block(move || {
        let opt_profile1 = profile_orm.modify_profile(user_id, modify_profile)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        opt_profile1
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 403, description = "Access denied: insufficient user rights.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_ACCESS_DENIED))),
        (status = 416, description = "Error parsing input parameter. `curl -i -X DELETE http://localhost:8080/api/users/2a`",
            body = ApiError, example = json!(ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED
                , "`id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
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
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let msg = &format!("`{}` - {}", "id", &e);
        error!("{}-{}; {}", code_to_str(StatusCode::RANGE_NOT_SATISFIABLE), err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg);
        ApiError::create(416, err::MSG_PARSING_TYPE_NOT_SUPPORTED, &msg) // 416
    })?;

    let profile_orm2 = profile_orm.clone();
    // Get a list of logo file names for streams of the user with the specified user_id.
    let path_file_img_list = web::block(move || {
        // Filter for the list of stream logos by user ID.
        profile_orm2.filter_stream_logos(id)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            })
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;
    
    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(id)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    if let Some(profile) = opt_profile {
        let config_prfl = config_prfl.get_ref().clone();
        // Get the path to the "avatar" file.
        let path_file_img = profile.avatar.clone().unwrap_or("".to_string());
        // If the image file name starts with the specified alias, then delete the file.
        remove_image_file(&path_file_img, ALIAS_AVATAR_FILES_DIR, &config_prfl.prfl_avatar_files_dir, &"delete_profile()");
        // Delete all specified logo files in the given list.
        let _ = remove_stream_logo_files(path_file_img_list, get_logo_files_dir());

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
        (status = 401, description = "An authorization token is required.", body = ApiError,
            example = json!(ApiError::new(403, err::MSG_MISSING_TOKEN))),
        (status = 506, description = "Blocking error.", body = ApiError, 
            example = json!(ApiError::create(506, err::MSG_BLOCKING, "Error while blocking process."))),
        (status = 507, description = "Database error.", body = ApiError, 
            example = json!(ApiError::create(507, err::MSG_DATABASE, "Error while querying the database."))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/profiles_current", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_profile_current(
    authenticated: Authenticated,
    config_prfl: web::Data<config_prfl::ConfigPrfl>,
    profile_orm: web::Data<ProfileOrmApp>,
) -> actix_web::Result<HttpResponse, ApiError> {
    // Get current user details.
    let user = authenticated.deref();
    let user_id = user.id;
    
    let profile_orm2 = profile_orm.clone();
    // Get a list of logo file names for streams of the user with the specified user_id.
    let path_file_img_list = web::block(move || {
        // Filter for the list of stream logos by user ID.
        profile_orm2.filter_stream_logos(user_id)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            })
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
    })??;

    let opt_profile = web::block(move || {
        // Delete an entity (profile).
        let res_profile = profile_orm.delete_profile(user_id)
        .map_err(|e| {
            error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
            ApiError::create(507, err::MSG_DATABASE, &e) // 507
        });
        res_profile
    })
    .await
    .map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) //506
    })??;

    if let Some(profile) = opt_profile {
        let config_prfl = config_prfl.get_ref().clone();
        // Get the path to the "avatar" file.
        let path_file_img = profile.avatar.clone().unwrap_or("".to_string());
        // If the image file name starts with the specified alias, then delete the file.
        remove_image_file(&path_file_img, ALIAS_AVATAR_FILES_DIR, &config_prfl.prfl_avatar_files_dir, &"delete_profile_current()");
        // Delete all specified logo files in the given list.
        let _ = remove_stream_logo_files(path_file_img_list, get_logo_files_dir());

        Ok(HttpResponse::Ok().json(ProfileDto::from(profile))) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::{http, web};
    use vrb_common::api_error::ApiError;
    use vrb_tools::{config_app, token_data::BEARER};

    pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }

    pub fn cfg_config_app(config_app: config_app::ConfigApp) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_app = web::Data::new(config_app);
            config.app_data(web::Data::clone(&data_config_app));
        }
    }

    pub fn check_app_err(app_err_vec: Vec<ApiError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
}

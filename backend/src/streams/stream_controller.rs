use std::{borrow::Cow::Borrowed, ffi::OsStr, ops::Deref, path};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use mime::IMAGE;
use utoipa;

use crate::cdis::coding;
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::file_upload::upload;
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm,
    stream_models::{self, CreateStreamInfoDto, ModifyStreamInfoDto, StreamInfoDto},
    stream_orm::StreamOrm,
};
use crate::users::user_models::UserRole;
use crate::utils::parser;
use crate::validators::{msg_validation, Validator};

pub const ALIAS_LOGO_FILES: &str = "logo";
// 406 Not acceptable - Error deserializing field tag.
pub const MSG_INVALID_FIELD_TAG: &str = "invalid_field_tag";
// 413 Content too large - File size exceeds max.
pub const MSG_INVALID_FILE_SIZE: &str = "invalid_file_size";
// 415 Unsupported Media Type - Uploading Image Files. Mime file type is not valid.
pub const MSG_INVALID_FILE_TYPE: &str = "invalid_file_type";
// 500 Internal Server Error - Error uploading file
pub const MSG_ERROR_UPLOAD_FILE: &str = "error_upload_file";
// 510 Not Extended - Error while converting file.
pub const MSG_ERROR_CONVERT_FILE: &str = "error_converting_file";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        // POST /api/streams
        config
            .service(post_stream)
            // PUT /api/streams/{id}
            .service(put_stream)
            // DELETE /api/streams/{id}
            .service(delete_stream);
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
// Convert the file to another mime type.
#[rustfmt::skip]
fn convert_logo_file(path_logo_file: &str, config_strm: config_strm::ConfigStrm, name: &str) -> Result<Option<String>, String> {
    let path: path::PathBuf = path::PathBuf::from(&path_logo_file);
    let file_source_ext = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap().to_string();
    let strm_logo_ext = config_strm.strm_logo_ext.clone().unwrap_or(file_source_ext);
    // If you need to save in the specified format (strm_logo_ext.is_some()) or convert
    // to the specified size (strm_logo_max_width > 0 || strm_logo_max_height > 0), then do the following.
    if config_strm.strm_logo_ext.is_some()
        || config_strm.strm_logo_max_width > 0
        || config_strm.strm_logo_max_height > 0
    {
        // Convert the file to another mime type.
        let path_file = upload::convert_file(
            &path_logo_file,
            &strm_logo_ext,
            config_strm.strm_logo_max_width,
            config_strm.strm_logo_max_height,
        )?;
        if !path_file.eq(&path_logo_file) {
            remove_file_and_log(&path_logo_file, name);
        }
        Ok(Some(path_file))
    } else {
        Ok(None)
    }
}

#[derive(Debug, MultipartForm)]
pub struct CreateStreamForm {
    pub title: Text<String>,
    pub descript: Option<Text<String>>,
    pub starttime: Option<Text<DateTime<Utc>>>,
    pub source: Option<Text<String>>,
    pub tags: Text<String>,
    pub logofile: Option<TempFile>,
}

impl CreateStreamForm {
    pub fn convert(create_stream_form: CreateStreamForm) -> Result<(CreateStreamInfoDto, Option<TempFile>), String> {
        let val = create_stream_form.tags.into_inner();
        let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
        if let Err(err) = res_tags {
            return Err(err.to_string());
        }
        let tags: Vec<String> = res_tags.unwrap();

        Ok((
            CreateStreamInfoDto {
                title: create_stream_form.title.to_string(),
                descript: create_stream_form.descript.map(|v| v.to_string()),
                starttime: create_stream_form.starttime.map(|v| v.into_inner()),
                source: create_stream_form.source.map(|v| v.to_string()),
                tags,
            },
            create_stream_form.logofile,
        ))
    }
}

/// post_stream
/// 
/// Create a new stream.
/// 
/// Multipart/form-data is used to transfer data.
/// 
/// Request structure:
/// ```text
/// {
///   title: String,             // required
///   descript?: String,         // optional
///   starttime?: DateTime<Utc>, // optional
///   source?: String,           // optional
///   tags: String,              // required
///   logofile?: TempFile,       // optional
/// }
/// ```
/// Where:
/// "title" - stream title;
/// "descript" - description of the stream;
/// "starttime" - date and time (in Utc-format "2020-01-20T20:10:57.000Z") of the start of the stream;
/// "source" - source value ("obs" by default) of the stream;
/// "tags" - serialized array of string values of stream tags("['tag1','tag2']");
/// "logofile" - attached stream image file (jpeg,gif,png,bmp);
/// 
/// The "starttime" field indicates the date of the future stream.
/// And cannot contain a past period (date and time).
/// 
/// It is recommended to enter the date and time in ISO 8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// The "tags" field represents a serialized array of string values.
/// ```text
/// { 'tags': JSON.stringify(["tag1", "tag2"]) } // "['tag1', 'tag2']"
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title1" -F "tags=['tag1','tag2']"
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']"
/// ```
/// Additionally, you can specify the name of the image file.
/// ```text
/// curl -i -X POST http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']" -F "logofile=@image.jpg"
/// ```
///  
/// Return a new stream (`StreamInfoDto`) with status 201.
/// 
#[utoipa::path(
    responses(
        (status = 201, description = "Create a new stream.", body = StreamInfoDto),
        (status = 406, description = "Error deserializing field \"tags\". `curl -X POST http://localhost:8080/api/streams
            -F 'title=title' -F 'tags=[\"tag\"'`",
            body = AppError, example = json!(AppError::not_acceptable406(
                &format!("{}: {}", MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6")))),
        (status = 413, description = "Invalid image file size. `curl -i -X POST http://localhost:8080/api/streams
            -F 'title=title2'  -F 'tags=[\"tag1\"]' -F 'logofile=@image.jpg'`", body = AppError,
            example = json!(AppError::content_large413(MSG_INVALID_FILE_SIZE).add_param(Borrowed("invalidFileSize"),
                &serde_json::json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X POST http://localhost:8080/api/streams
            -F 'title=title3'  -F 'tags=[\"tag3\"]' -F 'logofile=@image.svg'`", body = AppError,
            example = json!(AppError::unsupported_type415(MSG_INVALID_FILE_TYPE).add_param(Borrowed("invalidFileType"),
                &serde_json::json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 417, description = "Validation error. `curl -X POST http://localhost:8080/api/streams
            -F 'title=t' -F 'descript=d' -F 'starttime=2020-01-20T20:10:57.000Z' -F 'tags=[]'`", body = [AppError],
            example = json!(AppError::validations(
                (CreateStreamInfoDto {
                    title: "u".to_string(),
                    descript: Some("d".to_string()),
                    starttime: Some(DateTime::parse_from_rfc3339("2020-01-20T20:10:57.000Z").unwrap().with_timezone(&Utc)),
                    source: None,
                    tags: vec!()
                }).validate().err().unwrap()) )),
        (status = 500, description = "Error loading file.", body = AppError, example = json!(
            AppError::internal_err500(&format!("{}: {}: {}", MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg", "File not found.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = AppError,
            example = json!(AppError::not_extended510(
                &format!("{}: {}", MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\"")))),
    ),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[post("/api/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    MultipartForm(create_stream_form): MultipartForm<CreateStreamForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from MultipartForm.
    let (create_stream_info_dto, logofile) = CreateStreamForm::convert(create_stream_form)
        .map_err(|e| {
            let message = format!("{}: {}", MSG_INVALID_FIELD_TAG, e);
            log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message);
            AppError::not_acceptable406(&message) // 406
        })?;

    // Checking the validity of the data model.
    let validation_res = create_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors)); // 417
        return Ok(AppError::to_response(&AppError::validations(validation_errors)));
    }

    let config_strm = config_strm.get_ref().clone();
    let logo_files_dir = config_strm.strm_logo_files_dir.clone();
    let mut path_new_logo_file = "".to_string();

    while let Some(temp_file) = logofile {
        if temp_file.size == 0 {
            break;
        }
        // Check file size for maximum value.
        if config_strm.strm_logo_max_size > 0 && temp_file.size > config_strm.strm_logo_max_size {
            let json = serde_json::json!({ "actualFileSize": temp_file.size, "maxFileSize": config_strm.strm_logo_max_size });
            log::error!("{}: {} {}", err::CD_CONTENT_TOO_LARGE, MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(AppError::content_large413(MSG_INVALID_FILE_SIZE) // 413
                .add_param(Borrowed("invalidFileSize"), &json));
        }
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_types: Vec<String> = config_strm.strm_logo_valid_types.clone();
        // Checking the file for valid mime types.
        if !valid_file_types.contains(&file_mime_type) {
            let json = serde_json::json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_types.join(",") });
            log::error!("{}: {} {}", err::CD_UNSUPPORTED_TYPE, MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(AppError::unsupported_type415(MSG_INVALID_FILE_TYPE) // 415
                .add_param(Borrowed("invalidFileType"), &json));
        }
        // Get the name of the new file.
        let path_file = get_new_path_file(curr_user_id, Utc::now(), &file_mime_type, &logo_files_dir);
        // Persist the temporary file at the target path.
        // If a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&path_file);
        if let Err(err) = res_upload {
            let message = format!("{}: {}: {}", MSG_ERROR_UPLOAD_FILE, &path_file, err.to_string());
            log::error!("{}: {}", err::CD_INTER_ERROR, &message);
            return Err(AppError::internal_err500(&message)) // 500
        }
        path_new_logo_file = path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "post_stream()")
            .map_err(|e| {
                let message = format!("{}: {}", MSG_ERROR_CONVERT_FILE, e);
                log::error!("{}: {}", err::CD_NOT_EXTENDED, &message);
                AppError::not_extended510(&message) //510
            })?;
        if let Some(new_path_file) = res_convert_logo_file {
            path_new_logo_file = new_path_file;
        }

        break;
    }

    let mut create_stream = stream_models::CreateStream::convert(create_stream_info_dto.clone(), curr_user_id);
    let tags = create_stream_info_dto.tags.clone();
    if path_new_logo_file.len() > 0 {
        let alias_logo_file = path_new_logo_file.replace(&logo_files_dir, &format!("/{}", ALIAS_LOGO_FILES));
        create_stream.logo = Some(alias_logo_file);
    }

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data = stream_orm.create_stream(create_stream, &tags).map_err(|e| {
            log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    if res_data.is_err() {
        remove_file_and_log(&path_new_logo_file, &"post_stream()");
    }
    let (stream, stream_tags) = res_data?;
    // Merge a "stream" and a corresponding list of "tags".
    let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
    let stream_info_dto = list[0].clone();

    Ok(HttpResponse::Created().json(stream_info_dto)) // 201
}

fn get_new_path_file(user_id: i32, date_time: DateTime<Utc>, file_mime_type: &str, file_dir: &str) -> String {
    let only_file_name = format!("{}_{}", user_id, coding::encode(date_time, 1));
    // Get file name and file extension.
    let file_ext = file_mime_type.replace(&format!("{}/", IMAGE), "");
    // Add 'file path' + 'file name'.'file extension'.
    let path: path::PathBuf = [file_dir, &format!("{}.{}", only_file_name, file_ext)].iter().collect();
    path.to_str().unwrap().to_string()
}

#[derive(Debug, MultipartForm)]
pub struct ModifyStreamForm {
    pub title: Option<Text<String>>,
    pub descript: Option<Text<String>>,
    pub starttime: Option<Text<DateTime<Utc>>>,
    pub source: Option<Text<String>>,
    pub tags: Option<Text<String>>,
    pub logofile: Option<TempFile>,
}

impl ModifyStreamForm {
    pub fn convert(
        modify_stream_form: ModifyStreamForm,
    ) -> Result<(stream_models::ModifyStreamInfoDto, Option<TempFile>), String> {
        let tags: Option<Vec<String>> = match modify_stream_form.tags {
            Some(v) => {
                let val = v.into_inner();
                let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
                if let Err(err) = res_tags {
                    return Err(err.to_string());
                }
                Some(res_tags.unwrap())
            }
            None => None,
        };
        Ok((
            stream_models::ModifyStreamInfoDto {
                title: modify_stream_form.title.map(|v| v.into_inner()),
                descript: modify_stream_form.descript.map(|v| v.into_inner()),
                starttime: modify_stream_form.starttime.map(|v| v.into_inner()),
                source: modify_stream_form.source.map(|v| v.into_inner()),
                tags,
            },
            modify_stream_form.logofile,
        ))
    }
}

/// put_stream
/// 
/// Update the stream with new data.
/// 
/// Multipart/form-data is used to transfer data.
/// 
/// Request structure:
/// ```text
/// {
///   title?: String,            // optional
///   descript?: String,         // optional
///   starttime?: DateTime<Utc>, // optional
///   source?: String,           // optional
///   tags?: String,             // optional
///   logofile?: TempFile,       // optional
/// }
/// Where:
/// "title" - stream title;
/// "descript" - description of the stream;
/// "starttime" - date and time (in Utc-format "2020-01-20T20:10:57.000Z") of the start of the stream;
/// "source" - source value ("obs" by default) of the stream;
/// "tags" - serialized array of string values of stream tags("['tag1','tag2']");
/// "logofile" - attached stream image file (jpeg,gif,png,bmp);
/// 
/// ```
/// The "starttime" field indicates the date of the future stream.
/// And cannot contain a past period (date and time).
/// 
/// It is recommended to enter the date and time in ISO 8601 format.
/// ```text
/// var d1 = new Date();
/// { starttime: d1.toISOString() } // "2020-01-20T20:10:57.000Z"
/// ```
/// It is allowed to specify the date and time with a time zone value.
/// ```text
/// { "starttime": "2020-01-20T22:10:57+02:00" }
/// ```
/// The "tags" field represents a serialized array of string values.
/// ```text
/// { 'tags': JSON.stringify(["tag1", "tag2"]) } // "['tag1', 'tag2']"
/// ```
/// One could call with following curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams -F "title=title1" -F "tags=['tag1','tag2']"
/// ```
/// Could be called with all fields with the next curl.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']"
/// ```
/// Additionally, you can specify the name of the image file.
/// ```text
/// curl -i -X PUT http://localhost:8080/api/streams -F "title=title2" -F "descript=descript2" \
///   -F "starttime=2020-01-20T20:10:57.000Z" -F "tags=['tag1','tag2']" -F "logofile=@image.jpg"
/// ```
///  
/// Return the stream with updated data (`StreamInfoDto`) with status 200 or 204 (no content) if the stream is not found.
/// 
#[utoipa::path(
    responses(
        (status = 200, description = "Update the stream with new data.", body = StreamInfoDto),
        (status = 204, description = "The stream with the specified ID was not found."),
        (status = 406, description = "Error deserializing field \"tags\". `curl -X PUT http://localhost:8080/api/streams
            -F 'title=title' -F 'tags=[\"tag\"'`",
            body = AppError, example = json!(AppError::not_acceptable406(
                &format!("{}: {}", MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6")))),
        (status = 413, description = "Invalid image file size. `curl -i -X PUT http://localhost:8080/api/streams
            -F 'title=title2'  -F 'tags=[\"tag1\"]' -F 'logofile=@image.jpg'`", body = AppError,
            example = json!(AppError::content_large413(MSG_INVALID_FILE_SIZE).add_param(Borrowed("invalidFileSize"),
                &serde_json::json!({ "actualFileSize": 186, "maxFileSize": 160 })))),
        (status = 415, description = "Uploading a file with an invalid type `svg`. `curl -i -X PUT http://localhost:8080/api/streams
            -F 'title=title3'  -F 'tags=[\"tag3\"]' -F 'logofile=@image.svg'`", body = AppError,
            example = json!(AppError::unsupported_type415(MSG_INVALID_FILE_TYPE).add_param(Borrowed("invalidFileType"),
                &serde_json::json!({ "actualFileType": "image/svg+xml", "validFileType": "image/jpeg,image/png" })))),
        (status = 417, description = "Validation error. `curl -X PUT http://localhost:8080/api/streams
            -F 'title=t' -F 'descript=d' -F 'starttime=2020-01-20T20:10:57.000Z' -F 'tags=[]'`", body = [AppError],
            example = json!(AppError::validations(
                (ModifyStreamInfoDto {
                    title: Some("u".to_string()),
                    descript: Some("d".to_string()),
                    starttime: Some(DateTime::parse_from_rfc3339("2020-01-20T20:10:57.000Z").unwrap().with_timezone(&Utc)),
                    source: None,
                    tags: Some(vec!()),
                }).validate().err().unwrap()) )),
        (status = 500, description = "Error loading file.", body = AppError, example = json!(
            AppError::internal_err500(&format!("{}: {}: {}", MSG_ERROR_UPLOAD_FILE, "/tmp/demo.jpg", "File not found.")))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
        (status = 510, description = "Error while converting file.", body = AppError,
            example = json!(AppError::not_extended510(
                &format!("{}: {}", MSG_ERROR_CONVERT_FILE, "Invalid source file image type \"svg\"")))),
    ),
    security(("bearer_auth" = [])),
)]    
// PUT /api/streams/{id}
#[rustfmt::skip]
#[put("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    MultipartForm(modify_stream_form): MultipartForm<ModifyStreamForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;
    
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_UNSUPPORTED_TYPE, &message);
        AppError::unsupported_type415(&message) // 415
    })?;
    
    // Get data from MultipartForm.
    let (modify_stream_info_dto, logofile) = ModifyStreamForm::convert(modify_stream_form)
        .map_err(|e| {
            let message = format!("{}: {}", MSG_INVALID_FIELD_TAG, e);
            log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message); // 406
            AppError::not_acceptable406(&message)
        })?;

    // If there is not a single field in the MultipartForm, it gives an error 400 "Multipart stream is incomplete".
    
    // Checking the validity of the data model.
    let validation_res = modify_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::to_response(&AppError::validations(validation_errors))); // 417
    }
    let mut logo: Option<Option<String>> = None;
    let mut is_delete_logo = false;

    let config_strm = config_strm.get_ref().clone();
    let logo_files_dir = config_strm.strm_logo_files_dir.clone();
    let mut path_new_logo_file = "".to_string();

    while let Some(temp_file) = logofile {
        // Delete the old version of the logo file.
        is_delete_logo = true;
        if temp_file.size == 0 {
            logo = Some(None); // Set the "logo" field to `NULL`.
            break;
        }
        // Check file size for maximum value.
        if config_strm.strm_logo_max_size > 0 && temp_file.size > config_strm.strm_logo_max_size {
            let json = serde_json::json!({ "actualFileSize": temp_file.size, "maxFileSize": config_strm.strm_logo_max_size });
            log::error!("{}: {} {}", err::CD_CONTENT_TOO_LARGE, MSG_INVALID_FILE_SIZE, json.to_string());
            return Err(AppError::content_large413(MSG_INVALID_FILE_SIZE) // 413
                .add_param(Borrowed("invalidFileSize"), &json));
        }
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_types: Vec<String> = config_strm.strm_logo_valid_types.clone();
        // Checking the file for valid mime types.
        if !valid_file_types.contains(&file_mime_type) {
            let json = serde_json::json!({ "actualFileType": &file_mime_type, "validFileType": &valid_file_types.join(",") });
            log::error!("{}: {} {}", err::CD_UNSUPPORTED_TYPE, MSG_INVALID_FILE_TYPE, json.to_string());
            return Err(AppError::unsupported_type415(MSG_INVALID_FILE_TYPE) // 415
                .add_param(Borrowed("invalidFileType"), &json));
        }
        // Get the name of the new file.
        let path_file = get_new_path_file(curr_user_id, Utc::now(), &file_mime_type, &logo_files_dir);
        // Persist the temporary file at the target path.
        // If a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&path_file);
        if let Err(err) = res_upload {
            let message = format!("{}: {}: {}", MSG_ERROR_UPLOAD_FILE, &path_file, err.to_string());
            log::error!("{}: {}", err::CD_INTER_ERROR, &message);
            return Err(AppError::internal_err500(&message)) //500
        }
        path_new_logo_file = path_file;

        // Convert the file to another mime type.
        let res_convert_logo_file = convert_logo_file(&path_new_logo_file, config_strm.clone(), "put_stream()")
            .map_err(|e| {
                let message = format!("{}: {}", MSG_ERROR_CONVERT_FILE, e);
                log::error!("{}: {}", err::CD_NOT_EXTENDED, &message);
                AppError::not_extended510(&message) //510
            })?;
        if let Some(new_path_file) = res_convert_logo_file {
            path_new_logo_file = new_path_file;
        }

        let alias_logo_file = path_new_logo_file.replace(&logo_files_dir, &format!("/{}", ALIAS_LOGO_FILES));
        logo = Some(Some(alias_logo_file));

        break;
    }

    let tags = modify_stream_info_dto.tags.clone();
    let mut modify_stream = stream_models::ModifyStream::convert(modify_stream_info_dto);
    modify_stream.logo = logo;

    let opt_user_id: Option<i32> = if curr_user.role == UserRole::Admin { None } else { Some(curr_user.id) };
    let res_data = web::block(move || {
        let mut old_logo_file = "".to_string();
        if is_delete_logo {
            // Get the logo file name for an entity (stream) by ID.
            let res_get_stream_logo = stream_orm.get_stream_logo_by_id(id)
                .map_err(|e| {
                    log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                    AppError::database507(&e)        
                });

            if let Ok(Some(old_logo)) = res_get_stream_logo {
                let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
                old_logo_file = old_logo.replace(&alias_logo, &logo_files_dir);
            }
        }
        // Modify an entity (stream).
        let res_stream = stream_orm.modify_stream(id, opt_user_id, modify_stream, tags)
            .map_err(|e| {
                log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });

        (old_logo_file, res_stream)
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let (path_old_logo_file, res_stream) = res_data;

    let mut opt_stream_info_dto: Option<StreamInfoDto> = None;
    if let Ok(Some((stream, stream_tags)))= res_stream {
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
        opt_stream_info_dto = Some(list[0].clone());
        remove_file_and_log(&path_old_logo_file, &"put_stream()");
    } else {
        remove_file_and_log(&path_new_logo_file, &"put_stream()");
    }
    if let Some(stream_info_dto) = opt_stream_info_dto {
        Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/// delete_stream
///
/// Delete the specified stream.
///
/// One could call with following curl.
/// ```text
/// curl -i -X DELETE http://localhost:8080/api/streams/1
/// ```
///
/// Return the deleted stream (`StreamInfoDto`) with status 200 or 204 (no content) if the stream is not found.
///
#[utoipa::path(
    responses(
        (status = 200, description = "The specified stream was deleted successfully.", body = StreamInfoDto),
        (status = 204, description = "The specified stream was not found."),
        (status = 415, description = "Error parsing input parameter.", body = AppError, 
            example = json!(AppError::unsupported_type415(&"parsing_type_not_supported: `id` - invalid digit found in string (2a)"))),
        (status = 506, description = "Blocking error.", body = AppError, 
            example = json!(AppError::blocking506("Error while blocking process."))),
        (status = 507, description = "Database error.", body = AppError, 
            example = json!(AppError::database507("Error while querying the database."))),
    ),
    params(("id", description = "Unique stream ID.")),
    security(("bearer_auth" = [])),
)]
#[rustfmt::skip]
#[delete("/api/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_stream(
    authenticated: Authenticated,
    config_strm: web::Data<config_strm::ConfigStrm>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parser::parse_i32(&id_str).map_err(|e| {
        let message = &format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &e);
        log::error!("{}: {}", err::CD_UNSUPPORTED_TYPE, &message);
        AppError::unsupported_type415(&message)
    })?;

    let opt_user_id: Option<i32> = if curr_user.role == UserRole::Admin { None } else { Some(curr_user.id) };
    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data = stream_orm.delete_stream(id, opt_user_id).map_err(|e| {
            log::error!("{}:{}: {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
            AppError::database507(&e)
        });
        res_data
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}: {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    let opt_stream = res_data?;

    if let Some((stream, stream_tags)) = opt_stream {
        // Get the path to the "logo" file.
        let logo_file: String = stream.logo.clone().unwrap_or("".to_string());
        if logo_file.len() > 0 {
            // If there is a “logo” file, then delete this file.
            let config_strm = config_strm.get_ref().clone();
            let logo_files_dir = config_strm.strm_logo_files_dir.clone();
            let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
            let logo_name_full_path = logo_file.replace(&alias_logo, &logo_files_dir);
            remove_file_and_log(&logo_name_full_path, &"delete_stream()");
        }
    
        // Merge a "stream" and a corresponding list of "tags".
        let list = StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
        let stream_info_dto = list[0].clone();
        Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{fs, io::Write, path};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{
        self, body, dev,
        http::{
            self,
            header::{HeaderValue, CONTENT_TYPE},
            StatusCode,
        },
        test, web, App,
    };
    use chrono::{DateTime, Duration, SecondsFormat, Utc};

    use crate::cdis::coding;
    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        config_strm,
        stream_models::{Stream, StreamInfoDto, StreamModelsTest},
        stream_orm::tests::STREAM_ID,
    };
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";
    const MSG_MULTIPART_STREAM_INCOMPLETE: &str = "Multipart stream is incomplete";
    const MSG_CONTENT_TYPE_ERROR: &str = "No Content-Type header found";

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }
    fn create_stream(idx: i32, user_id: i32, title: &str, tags: &str, starttime: DateTime<Utc>) -> StreamInfoDto {
        let tags1: Vec<String> = tags.split(',').map(|val| val.to_string()).collect();
        let stream = Stream::new(STREAM_ID + idx, user_id, title, starttime);
        StreamInfoDto::convert(stream, user_id, &tags1)
    }

    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }

    fn configure_stream(
        cfg_c: (config_jwt::ConfigJwt, config_strm::ConfigStrm),
        data_c: (Vec<User>, Vec<Session>, Vec<StreamInfoDto>),
    ) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(cfg_c.0);
            let data_config_strm = web::Data::new(cfg_c.1);
            let data_user_orm = web::Data::new(UserOrmApp::create(&data_c.0));
            let data_session_orm = web::Data::new(SessionOrmApp::create(&data_c.1));
            let data_stream_orm = web::Data::new(StreamOrmApp::create(&data_c.2));

            config
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_strm))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_stream_orm));
        }
    }
    #[rustfmt::skip]
    fn get_cfg_data() -> ((config_jwt::ConfigJwt, config_strm::ConfigStrm), (Vec<User>, Vec<Session>, Vec<StreamInfoDto>), String) {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // The "stream" value will be required for the "put_stream" method.
        let stream = create_stream(0, user1.id, "title0", "tag01,tag02", Utc::now());

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let config_strm = config_strm::get_test_config();
        let cfg_c = (config_jwt, config_strm);
        let data_c = (vec![user1], vec![session1], vec![stream_dto]);
        (cfg_c, data_c, token)
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

    // ** post_stream **

    #[actix_web::test]
    async fn test_post_stream_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
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
    async fn test_post_stream_epmty_form() {
        let form_builder = MultiPartFormDataBuilder::new();
        let (header, body) = form_builder.build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
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
    async fn test_post_stream_title_empty() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", "")
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TITLE_REQUIRED, stream_models::MSG_TITLE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_title_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_title_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_descript_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_descript_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_starttime_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let starttime_s = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("starttime", starttime_s)
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_MIN_VALID_STARTTIME]);
    }
    #[actix_web::test]
    async fn test_post_stream_source_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_min())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_source_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_max())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_tags_min_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_min();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_post_stream_tags_max_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_max();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_post_stream_tag_name_min() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_min());
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_tag_name_max() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_max());
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_tag() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", "aaa")
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: {}", MSG_INVALID_FIELD_TAG, "expected value at line 1 column 1"));
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_file_size() {
        let name1_file = "post_circuit5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 2).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let mut config_strm = config_strm::get_test_config();
        config_strm.strm_logo_max_size = 160;
        let cfg_c = (cfg_c.0, config_strm);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONTENT_TOO_LARGE);
        assert_eq!(app_err.message, MSG_INVALID_FILE_SIZE);
        let json = serde_json::json!({ "actualFileSize": 186, "maxFileSize": 160 });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_stream_invalid_file_type() {
        let name1_file = "post_ellipse5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .with_file(path_name1_file.clone(), "logofile", "image/bmp", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        assert_eq!(app_err.message, MSG_INVALID_FILE_TYPE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileType": "image/bmp", "validFileType": "image/jpeg,image/png" });
        assert_eq!(*app_err.params.get("invalidFileType").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_without_logo_file() {
        let title_s = StreamModelsTest::title_enough();
        let descript_s = format!("{}a", StreamModelsTest::descript_min());
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();
        let starttime = Utc::now() + Duration::minutes(2);
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let source_s = format!("{}a", StreamModelsTest::source_min());

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("descript", &descript_s)
            .with_text("starttime", &starttime_s)
            .with_text("source", &source_s)
            .with_text("tags", &tags_s)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();
        let stream1 = data_c.2.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream1.id + 1);
        assert_eq!(stream_dto_res.user_id, user1.id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, descript_s);
        assert!(stream_dto_res.logo.is_none());
        #[rustfmt::skip]
        assert_eq!(stream_dto_res.starttime.to_rfc3339_opts(SecondsFormat::Millis, true), starttime_s);
        assert_eq!(stream_dto_res.source, source_s);
        assert_eq!(stream_dto_res.tags, tags);
        assert_eq!(stream_dto_res.is_my_stream, true);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_with_logo_file_new() {
        let name1_file = "post_circle5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();
        let stream1 = data_c.2.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream1.id + 1);
        assert_eq!(stream_dto_res.user_id, user1.id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.tags, tags);

        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());
        let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
        let logo_name_full_path = stream_dto_res_logo.replacen(&alias_logo, &strm_logo_files_dir, 1);
        let is_exists_logo_new = path::Path::new(&logo_name_full_path).exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string();
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string();
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string();
        assert_eq!(file_stem_part1, user1.id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_with_empty_file() {
        let name1_file = "post_circle_empty.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();
        let stream1 = data_c.2.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res.id, stream1.id + 1);
        assert_eq!(stream_dto_res.user_id, user1.id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, "");
        assert_eq!(stream_dto_res.logo, None);
        assert_eq!(stream_dto_res.tags.len(), tags.len());
        assert_eq!(stream_dto_res.tags, tags);
    }
    #[actix_web::test]
    async fn test_post_stream_valid_data_with_logo_convert_file_new() {
        let name1_file = "post_triangle_23x19.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 3).unwrap();

        let title_s = StreamModelsTest::title_enough();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags.clone()).unwrap();

        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &title_s)
            .with_text("tags", &tags_s)
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1 = data_c.0.get(0).unwrap().clone();

        let mut config_strm = config_strm::get_test_config();
        let file_ext = "jpeg".to_string();
        config_strm.strm_logo_ext = Some(file_ext.clone());
        config_strm.strm_logo_max_width = 18;
        config_strm.strm_logo_max_height = 18;
        let strm_logo_files_dir = config_strm.strm_logo_files_dir.clone();
        let cfg_c = (cfg_c.0, config_strm);

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/streams").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), StatusCode::CREATED); // 201
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());
        let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
        let logo_name_full_path = stream_dto_res_logo.replacen(&alias_logo, &strm_logo_files_dir, 1);
        let path = path::Path::new(&logo_name_full_path);
        let receiver_ext = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap();
        let is_exists_logo_new = path.exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert_eq!(file_ext, receiver_ext);
        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string();
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string();
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string();
        assert_eq!(file_stem_part1, user1.id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }

    // ** put_stream **

    #[actix_web::test]
    async fn test_put_stream_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/1")).insert_header(header_auth(&token))
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
    async fn test_put_stream_epmty_form() {
        let (header, body) = MultiPartFormDataBuilder::new().build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/1")).insert_header(header_auth(&token))
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
    async fn test_put_stream_invalid_id() {
        let stream_id_bad = "100a".to_string();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", "".to_string()).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", &stream_id_bad))
            .insert_header(header_auth(&token)).insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        let error = format!("{} ({})", "invalid digit found in string", stream_id_bad);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", &error));
    }

    #[actix_web::test]
    async fn test_put_stream_title_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_title_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", StreamModelsTest::title_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_descript_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", StreamModelsTest::descript_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_descript_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("descript", StreamModelsTest::descript_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_starttime_now() {
        let starttime_s = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("starttime", starttime_s).build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_MIN_VALID_STARTTIME]);
    }
    #[actix_web::test]
    async fn test_put_stream_source_min() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("source", StreamModelsTest::source_min())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_source_max() {
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("source", StreamModelsTest::source_max())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_tags_min_amount() {
        let tags = StreamModelsTest::tag_names_min();
        if tags.len() <= 0 {
            return;
        }
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_put_stream_tags_max_amount() {
        let tags = StreamModelsTest::tag_names_max();
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_AMOUNT]);
    }
    #[actix_web::test]
    async fn test_put_stream_tag_name_min() {
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_min()];
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_tag_name_max() {
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_max()];
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", serde_json::to_string(&tags).unwrap())
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_LENGTH]);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_tag() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", "aaa").build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        let message = format!("{}: {}", MSG_INVALID_FIELD_TAG, "expected value at line 1 column 1");
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_tag_vec() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("tags", "[\"tag\"").build();

        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE); // 406
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_NOT_ACCEPTABLE);
        #[rustfmt::skip]
        let message = format!("{}: {}", MSG_INVALID_FIELD_TAG, "EOF while parsing a list at line 1 column 6");
        assert_eq!(app_err.message, message);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_file_size() {
        let name1_file = "put_circuit5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        let (size, _name) = save_file_png(&path_name1_file, 2).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut config_strm = config_strm::get_test_config();
        config_strm.strm_logo_max_size = 160;
        let cfg_c = (cfg_c.0, config_strm);
        let strm_logo_max_size = cfg_c.1.strm_logo_max_size;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE); // 413
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_CONTENT_TOO_LARGE);
        assert_eq!(app_err.message, MSG_INVALID_FILE_SIZE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileSize": size, "maxFileSize": strm_logo_max_size });
        assert_eq!(*app_err.params.get("invalidFileSize").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_stream_invalid_file_type() {
        let name1_file = "put_ellipse5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/bmp", name1_file)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let valid_file_types: Vec<String> = cfg_c.1.strm_logo_valid_types.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri("/api/streams/1").insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(path_name1_file);
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        assert_eq!(app_err.message, MSG_INVALID_FILE_TYPE);
        #[rustfmt::skip]
        let json = serde_json::json!({ "actualFileType": "image/bmp", "validFileType": &valid_file_types.join(",") });
        assert_eq!(*app_err.params.get("invalidFileType").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_put_stream_non_existent_id() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", format!("{}a", StreamModelsTest::title_min()))
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_stream_another_user() {
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", format!("{}a", StreamModelsTest::title_min()))
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        #[rustfmt::skip]
        user_vec.push(UserOrmApp::new_user(2, "Liam_Smith", "Liam_Smith@gmail.com", "passwdL2S2"));
        let user_orm = UserOrmApp::create(&user_vec);
        let user2 = user_orm.user_vec.get(1).unwrap().clone();
        let stream2 = create_stream(1, user2.id, "title2", "tag01,tag02", Utc::now());
        let mut stream_vec = data_c.2;
        let stream2_id = stream2.id;
        stream_vec.push(stream2);
        let data_c = (user_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream2_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_put_stream_another_user_by_admin() {
        let new_title = format!("{}b", StreamModelsTest::title_min());
        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", &new_title)
            .build();

        let (cfg_c, data_c, token) = get_cfg_data();
        let mut user_vec = data_c.0;
        let user1 = user_vec.get_mut(0).unwrap();
        user1.role = UserRole::Admin;
        #[rustfmt::skip]
        user_vec.push(UserOrmApp::new_user(2, "Liam_Smith", "Liam_Smith@gmail.com", "passwdL2S2"));
        let user_orm = UserOrmApp::create(&user_vec);
        let user2 = user_orm.user_vec.get(1).unwrap().clone();
        let stream2 = create_stream(1, user2.id, "title2", "tag01,tag02", Utc::now());
        let mut stream_vec = data_c.2;
        stream_vec.push(stream2.clone());
        let data_c = (user_vec, data_c.1, stream_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res.user_id, stream2.user_id);
        assert_eq!(stream_dto_res.title, new_title);
        assert_eq!(stream_dto_res.descript, stream2.descript);
        assert_eq!(stream_dto_res.logo, stream2.logo);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_without_file() {
        let (cfg_c, data_c, token) = get_cfg_data();

        let stream = data_c.2.get(0).unwrap().clone();
        let user_id = stream.user_id;
        let title_s = format!("{}_a", stream.title.clone());
        let descript_s = format!("{}_a", stream.descript.clone());
        let logo = stream.logo.clone();
        let starttime = stream.starttime.clone() + Duration::days(1);
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let source_s = format!("{}_a", stream.source.to_string());
        let tags: Vec<String> = stream.tags.clone().iter().map(|v| format!("{}_a", v)).collect();
        let tags_s = serde_json::to_string(&tags).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", title_s.clone())
            .with_text("descript", descript_s.clone())
            .with_text("starttime", starttime_s.clone())
            .with_text("source", source_s.clone())
            .with_text("tags", tags_s.clone())
            .build();

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream.id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream.id);
        assert_eq!(stream_dto_res.user_id, user_id);
        assert_eq!(stream_dto_res.title, title_s);
        assert_eq!(stream_dto_res.descript, descript_s);
        assert_eq!(stream_dto_res.logo, logo);
        #[rustfmt::skip]
        assert_eq!(stream_dto_res.starttime.to_rfc3339_opts(SecondsFormat::Millis, true), starttime_s);
        assert_eq!(stream_dto_res.live, stream.live);
        assert_eq!(stream_dto_res.state, stream.state);
        assert_eq!(stream_dto_res.started, stream.started);
        assert_eq!(stream_dto_res.stopped, stream.stopped);
        assert_eq!(stream_dto_res.source, source_s);
        assert_eq!(stream_dto_res.tags, tags);
        assert_eq!(stream_dto_res.is_my_stream, stream.is_my_stream);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = stream_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_created_at = stream.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, old_created_at);
        let res_updated_at = stream_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let old_updated_at = stream.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, old_updated_at);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_a_logo_old_0_new_1() {
        let name1_file = "put_circle5x5_a_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();
        let user1_id = data_c.0.get(0).unwrap().id;
        let stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        let stream_orm = StreamOrmApp::create(&[stream.clone()]);

        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
        let logo_name_full_path = stream_dto_res_logo.replacen(&alias_logo, &strm_logo_files_dir, 1);
        let is_exists_logo_new = path::Path::new(&logo_name_full_path).exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, user1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_a_logo_old_0_new_1_convert_file_new() {
        let name1_file = "put_triangle_23x19.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 3).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let mut config_strm = config_strm::get_test_config();
        let file_ext = "jpeg".to_string();
        config_strm.strm_logo_ext = Some(file_ext.clone());
        config_strm.strm_logo_max_width = 18;
        config_strm.strm_logo_max_height = 18;
        let strm_logo_files_dir = config_strm.strm_logo_files_dir.clone();
        let cfg_c = (cfg_c.0, config_strm);

        let user1_id = data_c.0.get(0).unwrap().id;
        let stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
        let logo_name_full_path = stream_dto_res_logo.replacen(&alias_logo, &strm_logo_files_dir, 1);
        let path = path::Path::new(&logo_name_full_path);
        let receiver_ext = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap();
        let is_exists_logo_new = path.exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert_eq!(file_ext, receiver_ext);
        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, user1_id.to_string());
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_b_logo_old_1_new_1() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "put_circle5x5_b_old.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_logo = format!("/{}/{}", ALIAS_LOGO_FILES, name0_file);

        let name1_file = "put_circle5x5_b_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let user1_id = data_c.0.get(0).unwrap().id;
        let mut stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_logo);
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_logo_old);
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());
        let alias_logo = format!("/{}", ALIAS_LOGO_FILES);
        let logo_name_full_path = stream_dto_res_logo.replacen(&alias_logo, &strm_logo_files_dir, 1);

        let is_exists_logo_new = path::Path::new(&logo_name_full_path).exists();
        let _ = fs::remove_file(&logo_name_full_path);

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, user1_id.to_string());

        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = Utc::now().format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_c_logo_old_1_new_0() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "put_circle5x5_c_old.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&path_name0_file, 1).unwrap();
        let path_name0_logo = format!("/{}/{}", ALIAS_LOGO_FILES, name0_file);

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_text("title", "title1".to_string())
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let user1_id = data_c.0.get(0).unwrap().id;
        let mut stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_logo.clone());
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(path_name0_file.clone());
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_logo_old);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&format!("/{}", ALIAS_LOGO_FILES)));
        assert_eq!(&path_name0_logo, &stream_dto_res_logo);
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_d_logo_old_1_new_size0() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "put_circle5x5_d_old.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_logo = format!("/{}/{}", ALIAS_LOGO_FILES, name0_file);

        let name1_file = "put_circle5x5_d_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let user1_id = data_c.0.get(0).unwrap().id;
        let mut stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        stream.logo = Some(path_name0_logo);
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);
        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_logo_old);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert!(stream_dto_res.logo.is_none());
    }
    #[actix_web::test]
    async fn test_put_stream_valid_data_with_e_logo_old_0_new_size0() {
        let name1_file = "put_circle5x5_e_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_empty_file(&path_name1_file).unwrap();

        #[rustfmt::skip]
        let (header, body) = MultiPartFormDataBuilder::new()
            .with_file(path_name1_file.clone(), "logofile", "image/png", name1_file)
            .build();
        let (cfg_c, data_c, token) = get_cfg_data();

        let user1_id = data_c.0.get(0).unwrap().id;
        let stream = create_stream(0, user1_id, "title1", "tag11,tag12", Utc::now());
        let stream_id = stream.id;
        let stream_orm = StreamOrmApp::create(&[stream]);
        let data_c = (data_c.0, data_c.1, stream_orm.stream_info_vec.clone());

        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(put_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::put().uri(&format!("/api/streams/{}", stream_id))
            .insert_header(header_auth(&token))
            .insert_header(header).set_payload(body).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert!(stream_dto_res.logo.is_none());
    }

    // ** delete_stream **

    #[actix_web::test]
    async fn test_delete_stream_invalid_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id_bad = format!("{}a", data_c.2.get(0).unwrap().id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream_id_bad))
            .insert_header(header_auth(&token)).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, err::CD_UNSUPPORTED_TYPE);
        #[rustfmt::skip]
        let msg = format!("{}: `{}` - {}", err::MSG_PARSING_TYPE_NOT_SUPPORTED, "id", MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[actix_web::test]
    async fn test_delete_stream_non_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream_id = data_c.2.get(0).unwrap().id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream_id + 1))
            .insert_header(header_auth(&token)).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_stream_existent_id() {
        let (cfg_c, data_c, token) = get_cfg_data();
        let stream = data_c.2.get(0).unwrap().clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK); // 200
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_stream = serde_json::json!(stream).to_string();
        let stream_dto_org: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_stream_existent_id_with_logo_99() {
        let config_strm = config_strm::get_test_config();
        let strm_logo_files_dir = config_strm.strm_logo_files_dir;

        let name0_file = "delete_circle5x5_b_old.png";
        let path_name0_file = format!("{}/{}", &strm_logo_files_dir, name0_file);
        save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_logo = format!("/{}/{}", ALIAS_LOGO_FILES, name0_file);

        let (cfg_c, mut data_c, token) = get_cfg_data();
        let stream = data_c.2.get_mut(0).unwrap();
        stream.logo = Some(path_name0_logo);
        let stream2 = stream.clone();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_stream).configure(configure_stream(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/streams/{}", stream2.id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_logo_old);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json_stream = serde_json::json!(stream2).to_string();
        let stream_dto_org: StreamInfoDto = serde_json::from_slice(json_stream.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res, stream_dto_org);
    }
}

use std::borrow;
use std::{ops::Deref, time::Instant};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};

use crate::cdis::coding;
use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::file_upload::upload;
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{config_slp, stream_models, stream_orm::StreamOrm};
use crate::utils::parser::{parse_i32, CD_PARSE_INT_ERROR};
use crate::validators::{msg_validation, Validator};

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/streams/{id}
    cfg.service(get_stream_by_id)
        // GET api/streams
        .service(get_streams)
        // POST api/streams
        .service(post_stream)
        // PUT api/streams/{id}
        .service(put_stream)
        // DELETE api/streams/{id}
        .service(delete_stream);
}

fn err_parse_int(err: String) -> AppError {
    log::error!("{}: id: {}", CD_PARSE_INT_ERROR, err);
    AppError::new(CD_PARSE_INT_ERROR, &format!("id: {}", err)).set_status(400)
}
fn err_database(err: String) -> AppError {
    log::error!("{}: {}", err::CD_DATABASE, err);
    AppError::new(err::CD_DATABASE, &err).set_status(500)
}
fn err_blocking(err: String) -> AppError {
    log::error!("{}: {}", err::CD_BLOCKING, err);
    AppError::new(err::CD_BLOCKING, &err).set_status(500)
}
fn err_invalid_tags(err: String) -> AppError {
    let error = format!("{} {}", err::MSG_INVALID_TAGS_FIELD, err);
    log::error!("{}: {}", err::CD_INVALID_TAGS_FIELD, error);
    AppError::new(err::CD_INVALID_TAGS_FIELD, &error).set_status(400)
}
pub fn err_invalid_file_type(valie: String, valid_types: String) -> AppError {
    log::error!("{}: {}", err::CD_INVALID_FILE_TYPE, err::MSG_INVALID_IMAGE_FILE);
    let json = serde_json::json!({ "actualFileType": valie, "validFileType": valid_types });
    AppError::new(err::CD_INVALID_FILE_TYPE, err::MSG_INVALID_IMAGE_FILE)
        .add_param(borrow::Cow::Borrowed("invalidFileType"), &json)
        .set_status(400)
}
pub fn err_invalid_file_size(err_file_size: usize, max_size: usize) -> AppError {
    log::error!("{}: {}", err::CD_INVALID_FILE_SIZE, err::MSG_INVALID_FILE_SIZE);
    let json = serde_json::json!({ "actualFileSize": err_file_size, "maxFileSize": max_size });
    AppError::new(err::CD_INVALID_FILE_SIZE, err::MSG_INVALID_FILE_SIZE)
        .add_param(borrow::Cow::Borrowed("invalidFileSize"), &json)
        .set_status(400)
}
fn err_upload_file(err: String) -> AppError {
    let msg = format!("{} {}", err::MSG_INVALID_FILE_UPLOAD, err);
    log::error!("{}: {}", err::CD_INVALID_FILE_UPLOAD, msg);
    AppError::new(err::CD_INVALID_FILE_UPLOAD, &msg).set_status(400)
}

// GET api/streams/{id}
#[rustfmt::skip]
#[get("/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_stream_by_id(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    let res_data = web::block(move || {
        // Find 'stream' by id.
        let res_data =
            stream_orm.find_stream_by_id(id).map_err(|e| err_database(e.to_string()));
        res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let opt_data = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    let opt_stream_tag_dto = if let Some((stream, stream_tags)) = opt_data {
        let streams: Vec<stream_models::Stream> = vec![stream];
        // Merge a "stream" and a corresponding list of "tags".
        let list = stream_models::StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, curr_user_id);
        list.into_iter().nth(0)
    } else {
        None
    };

    log::info!("get_stream_by_id() elapsed time: {:.2?}", now.elapsed());
    if let Some(stream_tag_dto) = opt_stream_tag_dto {
        Ok(HttpResponse::Ok().json(stream_tag_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}

/* Name:
* @route streams
* @example streams?groupBy=date&userId=385e0469-7143-4915-88d0-f23f5b27ed28/9/2022&orderColumn=title&orderDirection=desc&live=true
* @type get
* @query pagination (optional):
* - userId (only for groupBy "date")
* - key (keyword by tag or date, the date should be YYYY-MM-DD)
* - live (false, true)
* - starttime (none, past, future)
* - groupBy (none / tag / date, none by default)
* - page (number, 1 by default)
* - limit (number, 10 by default)
* - orderColumn (starttime / title, starttime by default)
* - orderDirection (asc / desc, asc by default)
* @access public
*/
// GET api/streams
#[rustfmt::skip]
#[get("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn get_streams(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    query_params: web::Query<stream_models::SearchStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get search parameters.
    let search_stream_info_dto: stream_models::SearchStreamInfoDto = query_params.into_inner();

    let page: u32 = search_stream_info_dto.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
    let limit: u32 = search_stream_info_dto.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
    let search_stream = stream_models::SearchStream::convert(search_stream_info_dto);

    let res_data = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_data =
            stream_orm.find_streams(search_stream).map_err(|e| err_database(e.to_string()));
            res_data
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    let (count, streams, stream_tags) = match res_data { Ok(v) => v, Err(e) => return Err(e) };

    // Merge a "stream" and a corresponding list of "tags".
    let list = stream_models::StreamInfoDto::merge_streams_and_tags(&streams, &stream_tags, curr_user_id);

    let pages: u32 = count / limit + if (count % limit) > 0 { 1 } else { 0 };

    let result = stream_models::SearchStreamInfoResponseDto { list, limit, count, page, pages };

    log::info!("get_streams() elapsed time: {:.2?}", now.elapsed());
    Ok(HttpResponse::Ok().json(result)) // 200
}

fn remove_file_and_log(file_name: &str, msg: &str) {
    if file_name.len() > 0 {
        let res_remove = std::fs::remove_file(file_name);
        if let Err(err) = res_remove {
            log::error!("{} remove_file({}): error: {:?}", msg, file_name, err);
        }
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
    pub fn convert(
        create_stream_form: CreateStreamForm,
    ) -> Result<(stream_models::CreateStreamInfoDto, Option<TempFile>), String> {
        let val = create_stream_form.tags.into_inner();
        let res_tags: Result<Vec<String>, serde_json::error::Error> = serde_json::from_str(&val);
        if let Err(err) = res_tags {
            return Err(err.to_string());
        }
        let tags: Vec<String> = res_tags.unwrap();

        Ok((
            stream_models::CreateStreamInfoDto {
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
/* Name: 'Add stream'
* @route streams
* @type post
* @body title, description, starttime, tags (array stringify, 3 max)
* @files logo (jpg, png and gif only, 5MB)
* @required title, description
* @access protected
@Post()
@UseInterceptors(FileInterceptor('logo', UploadStreamLogoDTO))
async addStream (
    @Req() request: RequestSession,
    @Body() data: AddStreamDTO,
    @UploadedFile() logo: Express.Multer.File
): Promise<StreamDTO> {
    return await this.streamsService.addStream(request.user.getId(), data, logo);
}*/

// POST api/streams
#[rustfmt::skip]
#[post("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_stream(
    authenticated: Authenticated,
    config_slp: web::Data<config_slp::ConfigSLP>,
    stream_orm: web::Data<StreamOrmApp>,
    MultipartForm(create_stream_form): MultipartForm<CreateStreamForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from MultipartForm.
    let (create_stream_info_dto, logofile) = CreateStreamForm::convert(create_stream_form)
        .map_err(|err| err_invalid_tags(err))?;

    // Checking the validity of the data model.
    let validation_res = create_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }
    let mut create_file = "".to_string();
    while let Some(temp_file) = logofile {
        if temp_file.size == 0 {
            break;
        }
        let config_slp = config_slp.get_ref().clone();
        let valid_file_types = config_slp.slp_valid_types.clone();
        let valid_types: String = valid_file_types.join(",");
        // Check the mime file type for valid values.
        if let Err(err_file_type) = upload::file_validate_types(&temp_file, valid_file_types) {
            return Err(err_invalid_file_type(err_file_type, valid_types));
        }
        let max_size = config_slp.slp_max_size;
        // Check file size for maximum value.
        if let Err(err_file_size) = upload::file_validate_size(&temp_file, max_size) {
            return Err(err_invalid_file_size(err_file_size, max_size));
        }
        let date_time = Utc::now();
        // Get filename.
        let code_date = coding::encode(date_time, 1);
        // Upload the logo file.
        let file_name = format!("{}_{}", curr_user_id, code_date);
        let res_upload = upload::file_upload(temp_file, config_slp, file_name);
        if let Err(err) = res_upload {
            return Err(err_upload_file(err));
        }
        create_file = res_upload.unwrap();

        break;
    }

    let mut create_stream = stream_models::CreateStream::convert(create_stream_info_dto.clone(), curr_user_id);
    let tags = create_stream_info_dto.tags.clone();
    if create_file.len() > 0 {
        create_stream.logo = Some(create_file.clone());
    }

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data =
            stream_orm.create_stream(create_stream, &tags)
            .map_err(|e| err_database(e.to_string()));
            res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    if res_data.is_err() && create_file.len() > 0 {
        let res_remove = std::fs::remove_file(create_file.clone());
        if let Err(err) = res_remove {
            log::error!("post_stream() std::fs::remove_file({}): error: {:?}", create_file, err);
        }
    }
    let (stream, stream_tags) = res_data?;
    // Merge a "stream" and a corresponding list of "tags".
    let list = stream_models::StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
    let stream_info_dto = list[0].clone();
    log::info!("post_stream() elapsed time: {:.2?}", now.elapsed());

    Ok(HttpResponse::Created().json(stream_info_dto)) // 201
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

/* Name: 'Update stream'
* @route streams/:streamId
* @example streams/385e0469-7143-4915-88d0-f23f5b27ed36
* @type put
* @params streamId
* @body title, description, starttime, tags (array stringify, 3 max)
* @required streamId
* @access protected
@Put(':streamId')
@UseInterceptors(FileInterceptor('logo', UploadStreamLogoDTO))
async updateStream (
    @Req() request: RequestSession,
        @Param('streamId', new ParseUUIDPipe()) streamId: string,
        @Body() data: UpdateStreamDTO,
        @UploadedFile() logo: Express.Multer.File
): Promise<StreamDTO> {
    return await this.streamsService.updateStream(streamId, request.user.getId(), data, logo);
} */

// PUT api/streams/{id}
#[rustfmt::skip]
#[put("/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn put_stream(
    authenticated: Authenticated,
    config_slp: web::Data<config_slp::ConfigSLP>,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    MultipartForm(modify_stream_form): MultipartForm<ModifyStreamForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;
    
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;
    
    // Get data from MultipartForm.
    let (modify_stream_info_dto, logofile) = ModifyStreamForm::convert(modify_stream_form)
        .map_err(|err| err_invalid_tags(err))?;

    let res_check_required_fields = modify_stream_info_dto.check_required_fields();
    if logofile.is_none() && res_check_required_fields.is_err() {
        if let Err(errors) = res_check_required_fields {
            log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&errors));
            return Ok(AppError::validations_to_response(errors));
        }
    }
    
    // Checking the validity of the data model.
    let validation_res = modify_stream_info_dto.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let mut is_delete_logo = false;
    let mut new_logo_file = "".to_string();
    let mut logo: Option<Option<String>> = None;

    while let Some(temp_file) = logofile {
        // Delete the old version of the logo file.
        is_delete_logo = true;
        if temp_file.size == 0 {
            logo = Some(None); // Set the "logo" field to `NULL`.
            break;
        }
        let config_slp = config_slp.get_ref().clone();
        let valid_file_types = config_slp.slp_valid_types.clone();
        let valid_types: String = valid_file_types.join(",");
        // Check the mime file type for valid values.
        if let Err(err_file_type) = upload::file_validate_types(&temp_file, valid_file_types) {
            return Err(err_invalid_file_type(err_file_type, valid_types));
        }
        let max_size = config_slp.slp_max_size;
        // Check file size for maximum value.
        if let Err(err_file_size) = upload::file_validate_size(&temp_file, max_size) {
            return Err(err_invalid_file_size(err_file_size, max_size));
        }
        // Get filename.
        let code_date = coding::encode(Utc::now(), 1);
        // Upload the logo file.
        let file_name = format!("{}_{}", curr_user_id, code_date);
        let res_upload = upload::file_upload(temp_file, config_slp, file_name);
        if let Err(err) = res_upload {
            return Err(err_upload_file(err));
        }
        new_logo_file = res_upload.unwrap();
        logo = Some(Some(new_logo_file.clone()));

        break;
    }

    let tags = modify_stream_info_dto.tags.clone();
    let mut modify_stream = stream_models::ModifyStream::convert(modify_stream_info_dto);
    modify_stream.logo = logo;

    let res_data = web::block(move || {
        let mut old_logo_file = "".to_string();
        if is_delete_logo {
            // Get the logo file name for an entity (stream) by ID.
            let res_get_stream_logo =
            stream_orm.get_stream_logo_by_id(id, curr_user_id)
                .map_err(|e| err_database(e.to_string()));

            if let Ok(Some(old_logo)) = res_get_stream_logo {
                old_logo_file = old_logo;
            }
        }
        // Modify an entity (stream).
        let res_stream =
            stream_orm.modify_stream(id, curr_user_id, modify_stream, tags)
            .map_err(|e| err_database(e.to_string()));

        (old_logo_file, res_stream)
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let (old_logo_file, res_stream) = res_data;

    let mut opt_stream_info_dto: Option<stream_models::StreamInfoDto> = None;
    if let Ok(Some((stream, stream_tags)))= res_stream {
        // Merge a "stream" and a corresponding list of "tags".
        let list = stream_models::StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
        opt_stream_info_dto = Some(list[0].clone());
        remove_file_and_log(&old_logo_file, &"put_stream()");
    } else {
        remove_file_and_log(&new_logo_file, &"put_stream()");
    }
    log::info!("put_stream() elapsed time: {:.2?}", now.elapsed());

    if let Some(stream_info_dto) = opt_stream_info_dto {
        Ok(HttpResponse::Ok().json(stream_info_dto)) // 200
    } else {
        Err(AppError::new(err::CD_NOT_FOUND, err::MSG_STREAM_NOT_FOUND_BY_ID).set_status(404))
    }
}

/* Name: 'Delete stream'
* @route streams/:streamId
* @example streams/385e0469-7143-4915-88d0-f23f5b27ed36
* @type delete
* @params streamId
* @required streamId
* @access protected
@Delete(':streamId')
async deleteStream (
    @Req() request: RequestSession,
    @Param('streamId', new ParseUUIDPipe()) streamId: string
): Promise<StreamDTO> {
    return await this.streamsService.deleteStream(streamId, request.user.getId());
} */
// DELETE api/streams/{id}
#[rustfmt::skip]
#[delete("/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn delete_stream(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data =
            stream_orm.delete_stream(id, curr_user_id)
            .map_err(|e| err_database(e.to_string()));
            res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let result_count = res_data?;
    log::info!("delete_stream() elapsed time: {:.2?}", now.elapsed());
    // if result_count == 1 {
    //     stream.logo && await this.appFileService.remove(stream.logo, userId);  
    // }

    if 0 == result_count {
        Err(AppError::new(err::CD_NOT_FOUND, err::MSG_STREAM_NOT_FOUND_BY_ID).set_status(404))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{fs, io::Write, path};

    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Duration, SecondsFormat, Utc};
    // use serde_json::json;

    use crate::cdis::coding;
    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        config_slp,
        stream_models::{SearchStreamInfoResponseDto, Stream, StreamInfoDto, StreamModelsTest},
        stream_orm::tests::STREAM_ID,
    };
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };
    use crate::utils::parser::{CD_PARSE_INT_ERROR, MSG_PARSE_INT_ERROR};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";
    const MSG_CASTING_TO_TYPE: &str = "invalid digit found in string";
    const MSG_EXPECTED_VALUE_AT_LINE_COLUMN: &str = "expected value at line 1 column 1";
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

    async fn call_service1(
        config_jwt: config_jwt::ConfigJwt,
        vec: (Vec<User>, Vec<Session>, Vec<StreamInfoDto>),
        token: &str,
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(config_jwt);
        let data_config_slp = web::Data::new(config_slp::get_test_config());
        let data_user_orm = web::Data::new(UserOrmApp::create(&vec.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(vec.1));
        let data_stream_orm = web::Data::new(StreamOrmApp::create(&vec.2));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_config_slp))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .app_data(web::Data::clone(&data_stream_orm))
                .service(factory),
        )
        .await;
        let test_request = if token.len() > 0 {
            request.insert_header((http::header::AUTHORIZATION, format!("{}{}", BEARER, token)))
        } else {
            request
        };
        let req = test_request.to_request();

        test::call_service(&app, req).await
    }

    // ** get_stream_by_id **

    #[test]
    async fn test_get_stream_by_id_invalid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let stream_id_bad = format!("{}a", stream1b_dto.id);

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream_id_bad));
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_stream_by_id;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        #[rustfmt::skip]
        let msg = format!("id: {} `{}` - {}", MSG_PARSE_INT_ERROR, stream_id_bad, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[test]
    async fn test_get_stream_by_id_valid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream1b_dto.id));
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_stream_by_id;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_dto = serde_json::json!(stream1b_dto).to_string();
        let stream1b_dto_ser: StreamInfoDto =
            serde_json::from_slice(json_stream1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res, stream1b_dto_ser);
    }
    #[test]
    async fn test_get_stream_by_id_non_existent_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "title1", "tag11,tag12", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_id = stream_orm.stream_info_vec.get(0).unwrap().clone().id;

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream_id + 1));
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_stream_by_id;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT); // 204
    }

    // ** get_streams **

    #[test]
    async fn test_get_streams_search_by_user_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", now));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?userId={}&page={}&limit={}", user1.id, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_user_id_page2() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", now));
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));
        stream_vec.push(create_stream(4, user1.id, "demo31", "tag31,tag32", now));
        stream_vec.push(create_stream(5, user1.id, "demo32", "tag34,tag35", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info4 = stream_orm.stream_info_vec.get(4).unwrap().clone();
        let stream_info5 = stream_orm.stream_info_vec.get(5).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info4, stream_info5];
        let page = 2;
        let limit = 2;
        let uri = format!("/streams?userId={}&page={}&limit={}", user1.id, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list.len(), limit as usize);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 2);
    }
    #[test]
    async fn test_get_streams_search_by_live() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let live = true;
        let now = Utc::now();
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        let mut stream = create_stream(0, user1.id, "demo11", "tag11,tag12", now);
        stream.live = live;
        stream_vec.push(stream);
        let mut stream = create_stream(1, user1.id, "demo12", "tag14,tag15", now);
        stream.live = live;
        stream_vec.push(stream);
        stream_vec.push(create_stream(2, user1.id + 1, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id + 1, "demo22", "tag24,tag25", now));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?live={}&page={}&limit={}", live, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.list[0].live, live);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_is_future() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", now));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", tomorrow));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", yesterday));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", yesterday));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?isFuture={}&page={}&limit={}", true, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_is_not_future() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let yesterday = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", yesterday));
        stream_vec.push(create_stream(1, user1.id, "demo12", "tag14,tag15", yesterday));
        stream_vec.push(create_stream(2, user1.id, "demo21", "tag21,tag22", now));
        stream_vec.push(create_stream(3, user1.id, "demo22", "tag24,tag25", tomorrow));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream_info0 = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let stream_info1 = stream_orm.stream_info_vec.get(1).unwrap().clone();
        let stream1b_vec: Vec<StreamInfoDto> = vec![stream_info0, stream_info1];
        let limit = 2;
        let page = 1;
        let uri = format!("/streams?isFuture={}&page={}&limit={}", false, page, limit);

        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 2);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_asc() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let next_2_day = now + Duration::days(2);
        let next_1_day = now + Duration::days(1);
        let prev_2_day = now - Duration::days(2);
        let prev_1_day = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo11", "tag11,tag12", next_2_day));
        stream_vec.push(create_stream(1, user1.id, "demo13", "tag13,tag14", next_1_day));
        stream_vec.push(create_stream(2, user1.id, "demo15", "tag15,tag16", prev_1_day));
        stream_vec.push(create_stream(3, user1.id, "demo17", "tag17,tag18", prev_2_day));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_vec: Vec<StreamInfoDto> = vec![
            stream_orm.stream_info_vec.get(3).unwrap().clone(),
            stream_orm.stream_info_vec.get(2).unwrap().clone(),
            stream_orm.stream_info_vec.get(1).unwrap().clone(),
            stream_orm.stream_info_vec.get(0).unwrap().clone(),
        ];

        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Asc.to_string();
        let limit = 4;
        let page = 1;
        let uri = format!(
            "/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
            order_column, order_dir, page, limit
        );
        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }
    #[test]
    async fn test_get_streams_search_by_user_id_and_order_starttime_desc() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let next_2_day = now + Duration::days(2);
        let next_1_day = now + Duration::days(1);
        let prev_2_day = now - Duration::days(2);
        let prev_1_day = now - Duration::days(1);
        let mut stream_vec: Vec<StreamInfoDto> = Vec::new();
        stream_vec.push(create_stream(0, user1.id, "demo21", "tag21,tag22", prev_2_day));
        stream_vec.push(create_stream(1, user1.id, "demo23", "tag23,tag24", prev_1_day));
        stream_vec.push(create_stream(2, user1.id, "demo25", "tag25,tag26", next_1_day));
        stream_vec.push(create_stream(3, user1.id, "demo27", "tag27,tag28", next_2_day));

        let stream_orm = StreamOrmApp::create(&stream_vec);
        let stream1b_vec: Vec<StreamInfoDto> = vec![
            stream_orm.stream_info_vec.get(3).unwrap().clone(),
            stream_orm.stream_info_vec.get(2).unwrap().clone(),
            stream_orm.stream_info_vec.get(1).unwrap().clone(),
            stream_orm.stream_info_vec.get(0).unwrap().clone(),
        ];

        let order_column = stream_models::OrderColumn::Starttime.to_string();
        let order_dir = stream_models::OrderDirection::Desc.to_string();
        let limit = 4;
        let page = 1;
        let uri = format!(
            "/streams?orderColumn={}&orderDirection={}&page={}&limit={}",
            order_column, order_dir, page, limit
        );
        // GET api/streams
        let request = test::TestRequest::get().uri(&uri.to_string());
        let vec = (vec![user1], vec![session1], stream_vec);
        let factory = get_streams;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let resp_status = resp.status();
        assert_eq!(resp_status, http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let response: SearchStreamInfoResponseDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_vec = serde_json::json!(stream1b_vec).to_string();
        let stream1b_vec_ser: Vec<StreamInfoDto> =
            serde_json::from_slice(json_stream1b_vec.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.list, stream1b_vec_ser);
        assert_eq!(response.limit, limit);
        assert_eq!(response.count, 4);
        assert_eq!(response.page, page);
        assert_eq!(response.pages, 1);
    }

    fn save_empty_file(path_file: &str) -> Result<String, String> {
        let _ = fs::File::create(path_file).map_err(|e| e.to_string())?;
        Ok(path_file.to_string())
    }
    fn save_file_png(path_file: &str, code: u8) -> Result<String, String> {
        let header: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let footer: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
        #[rustfmt::skip]
        let buf1: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
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
        let buf2: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
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

        // if path::Path::new(&path_file).exists() {
        //     let _ = fs::remove_file(&path_file);
        // }

        let mut file = fs::File::create(path_file).map_err(|e| e.to_string())?;
        file.write_all(header.as_ref()).map_err(|e| e.to_string())?;
        #[rustfmt::skip]
        let buf: Vec<u8> = match code { 2 => buf2, _ => buf1 };
        file.write_all(buf.as_ref()).map_err(|e| e.to_string())?;
        file.write_all(footer.as_ref()).map_err(|e| e.to_string())?;

        Ok(path_file.to_string())
    }

    // ** post_stream **

    #[test]
    async fn test_post_stream_no_form() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // POST api/streams
        let request = test::TestRequest::post().uri("/streams");

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[test]
    async fn test_post_stream_epmty_form() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let form_builder = MultiPartFormDataBuilder::new();
        let (header, body) = form_builder.build();

        // POST api/streams
        let request = test::TestRequest::post()
            .uri("/streams")
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_MULTIPART_STREAM_INCOMPLETE));
    }

    async fn test_stream_validate(mode: u8, header: (String, String), body: Vec<u8>, code: &str, msgs: &[&str]) {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title0", "tag01,tag02", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let request = if mode == 1 {
            test::TestRequest::post().uri("/streams") // POST api/streams
        } else {
            // PUT api/streams/{id}
            test::TestRequest::put().uri(&format!("/streams/{}", stream_dto.id))
        };
        let request = request.insert_header(header).set_payload(body);

        let vec = (vec![user1], vec![session1], vec![stream_dto]);

        let resp = if mode == 1 {
            call_service1(config_jwt, vec, &token, post_stream, request).await
        } else {
            call_service1(config_jwt, vec, &token, put_stream, request).await
        };
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400
        let body = test::read_body(resp).await;
        let mut app_err_vec: Vec<AppError> = vec![];
        if err::CD_VALIDATION == code {
            let app_err_v: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
            app_err_vec.extend(app_err_v);
        } else {
            let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
            app_err_vec.extend(vec![app_err]);
        }
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }
    #[test]
    async fn test_post_stream_title_empty() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", "").with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_TITLE_REQUIRED, stream_models::MSG_TITLE_MIN_LENGTH];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_title_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_min())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_post_stream_title_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_max())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_post_stream_descript_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_min())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_DESCRIPT_MIN_LENGTH];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_descript_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("descript", StreamModelsTest::descript_max())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_DESCRIPT_MAX_LENGTH];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_starttime_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();
        let starttime = Utc::now();
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("starttime", starttime_s)
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_MIN_VALID_STARTTIME];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_source_min() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_min())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_SOURCE_MIN_LENGTH];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_source_max() {
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("source", StreamModelsTest::source_max())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        let msgs = [stream_models::MSG_SOURCE_MAX_LENGTH];
        test_stream_validate(1, header, body, err::CD_VALIDATION, &msgs).await;
    }
    #[test]
    async fn test_post_stream_tags_min_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_min();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_AMOUNT]).await;
    }
    #[test]
    async fn test_post_stream_tags_max_amount() {
        let tags: Vec<String> = StreamModelsTest::tag_names_max();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_AMOUNT]).await;
    }
    #[test]
    async fn test_post_stream_tag_name_min() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_min());
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_post_stream_tag_name_max() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_max());
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_post_stream_invalid_tag() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", "aaa");
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        let error = format!("{} {}", err::MSG_INVALID_TAGS_FIELD, MSG_EXPECTED_VALUE_AT_LINE_COLUMN);
        test_stream_validate(1, header, body, err::CD_INVALID_TAGS_FIELD, &[&error]).await;
    }

    #[test]
    async fn test_post_stream_invalid_file_type() {
        let name1_file = "post_ellipse5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/bmp", name1_file);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_INVALID_FILE_TYPE, &[err::MSG_INVALID_IMAGE_FILE]).await;
        let _ = fs::remove_file(&path_name1_file);
    }
    #[test]
    async fn test_post_stream_invalid_file_size() {
        let name1_file = "post_circuit5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 2).unwrap();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(1, header, body, err::CD_INVALID_FILE_SIZE, &[err::MSG_INVALID_FILE_SIZE]).await;
        let _ = fs::remove_file(&path_name1_file);
    }
    #[test]
    async fn test_post_stream_valid_data_without_logo_file() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let starttime = Utc::now() + Duration::minutes(2);
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", starttime);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();
        let tags: Vec<String> = stream.tags.clone();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", stream.title.to_string())
            .with_text("starttime", starttime_s)
            .with_text("tags", tags_s);
        let (header, body) = form_builder.build();

        // POST api/post_stream
        let request = test::TestRequest::post()
            .uri("/streams")
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream1b_dto = serde_json::json!(stream1b_dto).to_string();
        let stream1b_dto_ser: StreamInfoDto =
            serde_json::from_slice(json_stream1b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream1b_dto_ser.id);
        assert_eq!(stream_dto_res.user_id, stream1b_dto_ser.user_id);
        assert_eq!(stream_dto_res.title, stream1b_dto_ser.title);
        assert_eq!(stream_dto_res.descript, stream1b_dto_ser.descript);
        assert_eq!(stream_dto_res.logo, stream1b_dto_ser.logo);
        assert!(stream_dto_res.logo.is_none());
        assert_eq!(stream_dto_res.starttime, stream1b_dto_ser.starttime);
        assert_eq!(stream_dto_res.live, stream1b_dto_ser.live);
        assert_eq!(stream_dto_res.state, stream1b_dto_ser.state);
        assert_eq!(stream_dto_res.started, stream1b_dto_ser.started);
        assert_eq!(stream_dto_res.stopped, stream1b_dto_ser.stopped);
        assert_eq!(stream_dto_res.status, stream1b_dto_ser.status);
        assert_eq!(stream_dto_res.source, stream1b_dto_ser.source);
        assert_eq!(stream_dto_res.tags, stream1b_dto_ser.tags);
        assert_eq!(stream_dto_res.is_my_stream, stream1b_dto_ser.is_my_stream);
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z" "2024-02-06 09:55:41"
        let res_created_at = stream_dto_res.created_at.format(date_format).to_string();
        let ser_created_at = stream1b_dto_ser.created_at.format(date_format).to_string();
        assert_eq!(res_created_at, ser_created_at);
        let res_updated_at = stream_dto_res.updated_at.format(date_format).to_string();
        let ser_updated_at = stream1b_dto_ser.updated_at.format(date_format).to_string();
        assert_eq!(res_updated_at, ser_updated_at);
        assert_eq!(stream_dto_res.id, stream1b_dto_ser.id);
    }
    #[test]
    async fn test_post_stream_valid_data_with_logo_file_new() {
        let name1_file = "post_circle5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", StreamModelsTest::title_enough())
            .with_text("tags", tags_s);
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);

        let (header, body) = form_builder.build();
        let now = Utc::now();

        // POST api/post_stream
        let request = test::TestRequest::post()
            .uri("/streams")
            .insert_header(header)
            .set_payload(body);

        let user1_id = user1.id.to_string();
        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let config_slp = config_slp::get_test_config();
        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&config_slp.slp_dir));

        let is_exists_logo_new = path::Path::new(&stream_dto_res_logo).exists();
        let _ = fs::remove_file(&stream_dto_res_logo);
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string();
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string();
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string();
        assert_eq!(file_stem_part1, user1_id);
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[test]
    async fn test_post_stream_valid_data_with_empty_file() {
        let name1_file = "post_circle_empty.png";
        let path_name1_file = format!("./{}", name1_file);
        eprintln!("path_name1_file: {}", path_name1_file);
        save_empty_file(&path_name1_file).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let title1 = StreamModelsTest::title_enough().to_string();
        let tags: Vec<String> = StreamModelsTest::tag_names_enough();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", title1.to_string()).with_text("tags", tags_s);

        form_builder.with_file(path_name1_file.clone(), "logofile", "image/png", name1_file);

        let (header, body) = form_builder.build();

        // POST api/post_stream
        let request = test::TestRequest::post()
            .uri("/streams")
            .insert_header(header)
            .set_payload(body);

        let user1_id = user1.id;
        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        let _ = fs::remove_file(path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::CREATED); // 201
        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(stream_dto_res.user_id, user1_id);
        assert_eq!(stream_dto_res.title, title1.to_string());
        assert_eq!(stream_dto_res.descript, "");
        assert_eq!(stream_dto_res.logo, None);
        assert_eq!(stream_dto_res.tags.len(), tags.len());
        assert_eq!(stream_dto_res.tags, tags);
    }

    // ** put_stream **

    #[test]
    async fn test_put_stream_no_form() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // PUT api/streams/{id}
        let request = test::TestRequest::put().uri(&format!("/streams/1"));

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[test]
    async fn test_put_stream_epmty_form() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let form_builder = MultiPartFormDataBuilder::new();
        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri("/streams/1")
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_MULTIPART_STREAM_INCOMPLETE));
    }
    #[test]
    async fn test_put_stream_invalid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        let stream_id_bad = "100a".to_string();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", "".to_string());
        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_id_bad))
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        #[rustfmt::skip]
        let msg = format!("id: {} `{}` - {}", MSG_PARSE_INT_ERROR, stream_id_bad, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }

    #[test]
    async fn test_put_stream_title_min_amount() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", StreamModelsTest::title_min());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_title_max_amount() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", StreamModelsTest::title_max());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TITLE_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_descript_min() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("descript", StreamModelsTest::descript_min());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_descript_max() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("descript", StreamModelsTest::descript_max());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_DESCRIPT_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_starttime_now() {
        let starttime = Utc::now();
        let starttime_s = starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("starttime", starttime_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_MIN_VALID_STARTTIME]).await;
    }
    #[test]
    async fn test_put_stream_source_min() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("source", StreamModelsTest::source_min());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_source_max() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("source", StreamModelsTest::source_max());
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_SOURCE_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_tags_min_amount() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        let tag_names_min = StreamModelsTest::tag_names_min();
        if tag_names_min.len() > 0 {
            let tags_s = serde_json::to_string(&tag_names_min).unwrap();
            form_builder.with_text("tags", tags_s);
            let (header, body) = form_builder.build();
            #[rustfmt::skip]
            test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_AMOUNT]).await;
        }
    }
    #[test]
    async fn test_put_stream_tags_max_amount() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        let tags: Vec<String> = StreamModelsTest::tag_names_max();
        let tags_s = serde_json::to_string(&tags).unwrap();
        form_builder.with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_AMOUNT]).await;
    }
    #[test]
    async fn test_put_stream_tag_name_min() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_min()];
        let tags_s = serde_json::to_string(&tags).unwrap();
        form_builder.with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MIN_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_tag_name_max() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        let tags: Vec<String> = vec![StreamModelsTest::tag_name_max()];
        let tags_s = serde_json::to_string(&tags).unwrap();
        form_builder.with_text("tags", tags_s);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_VALIDATION, &[stream_models::MSG_TAG_MAX_LENGTH]).await;
    }
    #[test]
    async fn test_put_stream_invalid_tag() {
        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("tags", "aaa");
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        let error = format!("{} {}", err::MSG_INVALID_TAGS_FIELD, MSG_EXPECTED_VALUE_AT_LINE_COLUMN);
        test_stream_validate(2, header, body, err::CD_INVALID_TAGS_FIELD, &[&error]).await;
    }

    #[test]
    async fn test_put_stream_invalid_file_type() {
        let name1_file = "put_ellipse5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/bmp", name1_file);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_INVALID_FILE_TYPE, &[err::MSG_INVALID_IMAGE_FILE]).await;
        let _ = fs::remove_file(path_name1_file);
    }
    #[test]
    async fn test_put_stream_invalid_file_size() {
        let name1_file = "put_circuit5x5.png";
        let path_name1_file = format!("./{}", &name1_file);
        save_file_png(&path_name1_file, 2).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);
        let (header, body) = form_builder.build();
        #[rustfmt::skip]
        test_stream_validate(2, header, body, err::CD_INVALID_FILE_SIZE, &[err::MSG_INVALID_FILE_SIZE]).await;
        let _ = fs::remove_file(path_name1_file);
    }

    #[test]
    async fn test_put_stream_non_existent_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", format!("{}a", StreamModelsTest::title_min()));

        let (header, body) = form_builder.build();
        let user1_id = user1.id;

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", user1_id + 1))
            .insert_header(header)
            .set_payload(body);
        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_STREAM_NOT_FOUND_BY_ID);
    }
    #[test]
    async fn test_put_stream_valid_data_without_file() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut stream2_dto = stream_dto.clone();
        stream2_dto.title = "title2".to_string();
        stream2_dto.descript = "descript2".to_string();
        stream2_dto.starttime = now + Duration::days(1);
        stream2_dto.source = format!("{}_2", stream_models::STREAM_SOURCE_DEF.to_string());
        stream2_dto.tags.clear();
        stream2_dto.tags.push("tag11".to_string());
        stream2_dto.tags.push("tag14".to_string());
        stream2_dto.updated_at = Utc::now();
        let stream2b_dto = stream2_dto.clone();

        let starttime_s = stream2_dto.starttime.to_rfc3339_opts(SecondsFormat::Millis, true);
        let tags: Vec<String> = stream2_dto.tags.clone();
        let tags_s = serde_json::to_string(&tags).unwrap();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder
            .with_text("title", stream2_dto.title.to_string())
            .with_text("descript", stream2_dto.descript.to_string())
            .with_text("starttime", starttime_s)
            .with_text("source", stream2_dto.source.to_string())
            .with_text("tags", tags_s);

        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);
        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json_stream2b_dto = serde_json::json!(stream2b_dto).to_string();
        let stream2b_dto_ser: StreamInfoDto =
            serde_json::from_slice(json_stream2b_dto.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(stream_dto_res.id, stream2b_dto_ser.id);
        assert_eq!(stream_dto_res.user_id, stream2b_dto_ser.user_id);
        assert_eq!(stream_dto_res.title, stream2b_dto_ser.title);
        assert_eq!(stream_dto_res.descript, stream2b_dto_ser.descript);
        assert_eq!(stream_dto_res.logo, stream2b_dto_ser.logo);
        assert_eq!(stream_dto_res.starttime, stream2b_dto_ser.starttime);
        assert_eq!(stream_dto_res.live, stream2b_dto_ser.live);
        assert_eq!(stream_dto_res.state, stream2b_dto_ser.state);
        assert_eq!(stream_dto_res.started, stream2b_dto_ser.started);
        assert_eq!(stream_dto_res.stopped, stream2b_dto_ser.stopped);
        assert_eq!(stream_dto_res.status, stream2b_dto_ser.status);
        assert_eq!(stream_dto_res.source, stream2b_dto_ser.source);
        assert_eq!(stream_dto_res.tags, stream2b_dto_ser.tags);
        assert_eq!(stream_dto_res.is_my_stream, stream2b_dto_ser.is_my_stream);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = stream_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let ser_created_at = stream2b_dto_ser.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, ser_created_at);
        let res_updated_at = stream_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let ser_updated_at = stream2b_dto_ser.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, ser_updated_at);
        assert_eq!(stream_dto_res.id, stream2b_dto_ser.id);
    }
    #[test]
    async fn test_put_stream_valid_data_with_a_logo_old_0_new_1() {
        let name1_file = "put_circle5x5_a_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();

        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);
        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);

        let user1_id = user1.id.to_string();
        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let config_slp = config_slp::get_test_config();
        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&config_slp.slp_dir));

        let is_exists_logo_new = path::Path::new(&stream_dto_res_logo).exists();
        let _ = fs::remove_file(&stream_dto_res_logo);
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, user1_id);
        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[test]
    async fn test_put_stream_valid_data_with_b_logo_old_1_new_1() {
        let config_slp = config_slp::get_test_config();

        let name0_file = "put_circle5x5_b_old.png";
        let mut path = path::PathBuf::from(&config_slp.slp_dir);
        path.push(name0_file);
        let path_name0_file = path.to_str().unwrap().to_string();
        save_file_png(&(path_name0_file.clone()), 1).unwrap();

        let name1_file = "put_circle5x5_b_new.png";
        let path_name1_file = format!("./{}", name1_file);
        save_file_png(&path_name1_file, 1).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);
        stream.logo = Some(path_name0_file.clone());

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);

        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);

        let user1_id = user1.id.to_string();
        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        assert!(!is_exists_logo_old);

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let config_slp = config_slp::get_test_config();
        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&config_slp.slp_dir));

        let is_exists_logo_new = path::Path::new(&stream_dto_res_logo).exists();
        let _ = fs::remove_file(&stream_dto_res_logo);
        assert!(is_exists_logo_new);

        let path_logo = path::PathBuf::from(stream_dto_res_logo);
        let file_stem = path_logo.file_stem().unwrap().to_str().unwrap().to_string(); // file_stem: "1100_3226061294TF"
        let file_stem_parts: Vec<&str> = file_stem.split('_').collect();
        let file_stem_part1 = file_stem_parts.get(0).unwrap_or(&"").to_string(); // file_stem_part1: "1100"
        let file_stem_part2 = file_stem_parts.get(1).unwrap_or(&"").to_string(); // file_stem_part2: "3226061294TF"
        assert_eq!(file_stem_part1, user1_id);

        let date_time2 = coding::decode(&file_stem_part2, 1).unwrap();
        let date_format = "%Y-%m-%d %H:%M:%S"; // "%Y-%m-%d %H:%M:%S%.9f %z"
        let date_time2_s = date_time2.format(date_format).to_string(); // : 2024-02-06 09:55:41
        let now_s = now.format(date_format).to_string(); // : 2024-02-06 09:55:41
        assert_eq!(now_s, date_time2_s);
    }
    #[test]
    async fn test_put_stream_valid_data_with_c_logo_old_1_new_0() {
        let config_slp = config_slp::get_test_config();

        let name0_file = "put_circle5x5_c_old.png";
        let mut path = path::PathBuf::from(&config_slp.slp_dir);
        path.push(name0_file);
        let path_name0_file = path.to_str().unwrap().to_string();
        save_file_png(&(path_name0_file.clone()), 1).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);
        stream.logo = Some(path_name0_file.clone());

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_text("title", "title1".to_string());

        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(path_name0_file.clone());

        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        assert!(is_exists_logo_old);

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let config_slp = config_slp::get_test_config();
        let stream_dto_res_logo = stream_dto_res.logo.unwrap_or("".to_string());

        assert!(stream_dto_res_logo.len() > 0);
        assert!(stream_dto_res_logo.starts_with(&config_slp.slp_dir));

        assert_eq!(&path_name0_file, &stream_dto_res_logo);
    }
    #[test]
    async fn test_put_stream_valid_data_with_d_logo_old_1_new_size0() {
        let config_slp = config_slp::get_test_config();

        let name0_file = "put_circle5x5_d_old.png";
        let mut path = path::PathBuf::from(&config_slp.slp_dir);
        path.push(name0_file);
        let path_name0_file = path.to_str().unwrap().to_string();
        save_file_png(&(path_name0_file.clone()), 1).unwrap();

        let name1_file = "put_circle5x5_d_new.png";
        let path_name1_file = format!("./{}", name1_file);
        eprintln!("path_name1_file: {}", path_name1_file);
        save_empty_file(&path_name1_file).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let mut stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);
        stream.logo = Some(path_name0_file.clone());

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);

        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        let is_exists_logo_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);
        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::OK); // 200
        assert!(!is_exists_logo_old);

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert!(stream_dto_res.logo.is_none());
    }
    #[test]
    async fn test_put_stream_valid_data_with_e_logo_old_0_new_size0() {
        // let config_slp = config_slp::get_test_config();

        let name1_file = "put_circle5x5_e_new.png";
        let path_name1_file = format!("./{}", name1_file);
        eprintln!("path_name1_file: {}", path_name1_file);
        save_empty_file(&path_name1_file).unwrap();

        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        let mut form_builder = MultiPartFormDataBuilder::new();
        form_builder.with_file(path_name1_file.to_string(), "logofile", "image/png", name1_file);

        let (header, body) = form_builder.build();

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .insert_header(header)
            .set_payload(body);

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;

        let _ = fs::remove_file(&path_name1_file);

        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert!(stream_dto_res.logo.is_none());
    }

    // ** delete_stream **

    #[test]
    async fn test_delete_stream_invalid_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let stream_id_bad = "100a".to_string();

        // DELETE api/streams/{id}
        let request = test::TestRequest::delete().uri(&format!("/streams/{}", stream_id_bad));

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = delete_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, CD_PARSE_INT_ERROR);
        #[rustfmt::skip]
        let msg = format!("id: {} `{}` - {}", MSG_PARSE_INT_ERROR, stream_id_bad, MSG_CASTING_TO_TYPE);
        assert!(app_err.message.starts_with(&msg));
    }
    #[test]
    async fn test_delete_stream_non_existent_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        // DELETE api/streams/{id}
        let request = test::TestRequest::delete().uri(&format!("/streams/{}", stream_dto.id + 1));

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = delete_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND); // 404

        let body = test::read_body(resp).await;
        let app_err: AppError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        assert_eq!(app_err.code, err::CD_NOT_FOUND);
        assert_eq!(app_err.message, err::MSG_STREAM_NOT_FOUND_BY_ID);
    }
    #[test]
    async fn test_delete_stream_existent_id() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let now = Utc::now();
        let stream = create_stream(0, user1.id, "title1", "tag11,tag12", now);

        let stream_orm = StreamOrmApp::create(&[stream.clone()]);
        let stream_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        // DELETE api/streams/{id}
        let request = test::TestRequest::delete().uri(&format!("/streams/{}", stream_dto.id));

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = delete_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200
    }
}

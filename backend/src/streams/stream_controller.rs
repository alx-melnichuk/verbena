use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::borrow;
use std::{ops::Deref, time::Instant};

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::file_upload::upload;
use crate::settings::err;
use crate::streams::config_avatar_files;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{stream_models, stream_orm::StreamOrm};
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

#[derive(Debug, MultipartForm)]
pub struct CreateStreamForm {
    #[multipart(limit = "255")]
    pub title: Text<String>,
    #[multipart(limit = "2048")]
    pub descript: Option<Text<String>>,
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub logo: Option<String>,
    #[multipart(limit = "24")]
    pub starttime: Option<Text<DateTime<Utc>>>,
    #[multipart(limit = "255")]
    pub source: Option<Text<String>>,
    #[multipart(rename = "tags[]")]
    pub tags: Vec<Text<String>>,
    #[multipart(limit = "7 MiB")]
    pub avatar: Option<TempFile>,
}

impl CreateStreamForm {
    pub fn convert(create_stream_form: CreateStreamForm) -> (stream_models::CreateStreamInfoDto, Option<TempFile>) {
        let starttime: DateTime<Utc> = match create_stream_form.starttime {
            Some(v) => v.clone(),
            None => Utc::now(),
        };
        let tags: Vec<String> = create_stream_form.tags.iter().map(|v| v.to_string()).collect();

        (
            stream_models::CreateStreamInfoDto {
                title: create_stream_form.title.to_string(),
                descript: create_stream_form.descript.map(|v| v.to_string()),
                logo: None,
                starttime,
                live: None,
                state: None,
                started: None,
                stopped: None,
                status: None,
                source: create_stream_form.source.map(|v| v.to_string()),
                tags,
            },
            create_stream_form.avatar,
        )
    }
}

// POST api/streams
#[rustfmt::skip]
#[post("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_stream(
    _authenticated: Authenticated,
    config_avatar: web::Data<config_avatar_files::ConfigAvatarFiles>,
    // stream_orm: web::Data<StreamOrmApp>,
    // json_body: web::Json<stream_models::CreateStreamInfoDto>,
    MultipartForm(create_stream_form): MultipartForm<CreateStreamForm>,
) -> actix_web::Result<HttpResponse, AppError> {
    //
    let (create_stream_info_dto, opt_avatar) = CreateStreamForm::convert(create_stream_form);
    

    let title = create_stream_info_dto.title.to_string();
    // let mut file: NamedTempFile<File>,
    // let mut content_type: Option<Mime>,
    // let mut file_name: String = "".to_string();
    
    if let Some(temp_file) = opt_avatar {
        if 0 == temp_file.size {
            eprintln!("Delete old avatar file.");
        } else {
            let config_af = config_avatar.get_ref().clone();
            let valid_file_types = config_af.avatar_valid_types;
            let valid_types: String = valid_file_types.join(",");
            // Check the mime file type for valid values.
            let res_validate_types = upload::file_validate_types(&temp_file, valid_file_types);
            if let Err(err_file_type) = res_validate_types {
                return Err(err_invalid_file_type(err_file_type, valid_types));
            }
            let max_size = config_af.avatar_max_size;
            // Check file size for maximum value.
            let res_validate_size = upload::file_validate_size(&temp_file, max_size);
            if let Err(err_file_size) = res_validate_size {
                return Err(err_invalid_file_size(err_file_size, max_size));
            }
            // Upload the avatar file.
            let file_name = "file_name_demo".to_string();
            // let config_af2 = config_avatar.get_ref().clone();
            // upload::file_upload(temp_file, config_af2, file_name);

            let avatar_dir = config_af.avatar_dir.to_string();
            let path_file = format!("{}{}", avatar_dir, file_name);
            let res_upload = temp_file.file.persist(path_file);
            
            if let Err(err) = res_upload {
                return Err(AppError::new("InvalidFileUpload", err.to_string().as_str())
                    // .add_param(borrow::Cow::Borrowed("invalidFileType"), &json)
                    .set_status(400))
            }
        }
    }
    eprintln!("create_stream_info_dto: {:?}", &create_stream_info_dto);


    Ok(HttpResponse::Ok().json(json!({ "title": title }))) // 200
}

// POST api/streams
#[rustfmt::skip]
#[post("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
pub async fn post_stream0(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    json_body: web::Json<stream_models::CreateStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let create_stream_info: stream_models::CreateStreamInfoDto = json_body.into_inner();
    let create_stream = stream_models::CreateStream::convert(create_stream_info.clone(), curr_user_id);
    let tags = create_stream_info.tags.clone();

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data =
            stream_orm.create_stream(create_stream, &tags)
            .map_err(|e| err_database(e.to_string()));
            res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let (stream, stream_tags) = res_data?;
    // Merge a "stream" and a corresponding list of "tags".
    let list = stream_models::StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
    let stream_info_dto = list[0].clone();
    log::info!("post_stream() elapsed time: {:.2?}", now.elapsed());

    Ok(HttpResponse::Created().json(stream_info_dto)) // 201
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
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<stream_models::ModifyStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    // Get current user details.
    let curr_user = authenticated.deref();
    let curr_user_id = curr_user.id;
    
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let modify_stream_info: stream_models::ModifyStreamInfoDto = json_body.into_inner();
    let modify_stream = stream_models::ModifyStream::convert(modify_stream_info.clone(), curr_user_id);
    let opt_tags = modify_stream_info.tags.clone();

    let res_data = web::block(move || {
        // Add a new entity (stream).
        let res_data =
            stream_orm.modify_stream(id, modify_stream, opt_tags)
            .map_err(|e| err_database(e.to_string()));
            res_data
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    log::info!("put_stream() elapsed time: {:.2?}", now.elapsed());
    let opt_data = res_data?;
    
    if let Some((stream, stream_tags)) = opt_data {
        // Merge a "stream" and a corresponding list of "tags".
        let list = stream_models::StreamInfoDto::merge_streams_and_tags(&[stream], &stream_tags, curr_user_id);
        let stream_info_dto = list[0].clone();
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
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Duration, SecondsFormat, Utc};
    use serde_json::json;

    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        stream_models::{
            CreateStreamInfoDto, ModifyStreamInfoDto, SearchStreamInfoResponseDto, Stream, StreamInfoDto,
            StreamModelsTest,
        },
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
    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";

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
        let data_user_orm = web::Data::new(UserOrmApp::create(&vec.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(vec.1));
        let data_stream_orm = web::Data::new(StreamOrmApp::create(&vec.2));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
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

    // ** post_stream **

    #[test]
    async fn test_post_stream_no_data() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // POST api/post_stream
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
    async fn test_post_stream_empty_json_object() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // POST api/post_stream
        let request = test::TestRequest::post().uri("/streams").set_json(json!({}));

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }

    async fn test_post_stream_validate(create_stream: CreateStreamInfoDto, err_msg: &str) {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));
        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        // POST api/post_stream
        let request = test::TestRequest::post().uri("/streams").set_json(create_stream);

        let vec = (vec![user1], vec![session1], vec![]);
        let factory = post_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, err_msg);
    }
    #[test]
    async fn test_post_stream_title_empty() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TITLE_REQUIRED).await;
    }
    #[test]
    async fn test_post_stream_title_min() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: StreamModelsTest::title_min(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TITLE_MIN_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_title_max() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: StreamModelsTest::title_max(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TITLE_MAX_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_descript_min() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: Some(StreamModelsTest::descript_min()),
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_DESCRIPT_MIN_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_descript_max() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: Some(StreamModelsTest::descript_max()),
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_DESCRIPT_MAX_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_logo_min() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: Some(StreamModelsTest::logo_min()),
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_LOGO_MIN_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_logo_max() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: Some(StreamModelsTest::logo_max()),
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_LOGO_MAX_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_source_min() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: Some(StreamModelsTest::source_min()),
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_SOURCE_MIN_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_source_max() {
        let starttime = Utc::now();
        let tags: Vec<String> = vec!["tag11".to_string(), "tag12".to_string()];
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: Some(StreamModelsTest::source_max()),
            tags: tags.clone(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_SOURCE_MAX_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_tags_min_amount() {
        let starttime = Utc::now();
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: StreamModelsTest::tag_names_min(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TAG_NAME_MIN_AMOUNT).await;
    }
    #[test]
    async fn test_post_stream_tags_max_amount() {
        let starttime = Utc::now();
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: StreamModelsTest::tag_names_max(),
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TAG_NAME_MAX_AMOUNT).await;
    }
    #[test]
    async fn test_post_stream_tag_name_empty() {
        let starttime = Utc::now();
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push("".to_string());
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags,
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TAG_NAME_REQUIRED).await;
    }
    #[test]
    async fn test_post_stream_tag_name_min() {
        let starttime = Utc::now();
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_min());
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags,
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TAG_NAME_MIN_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_tag_name_max() {
        let starttime = Utc::now();
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_max());
        let create_stream = CreateStreamInfoDto {
            title: "title".to_string(),
            descript: None,
            logo: None,
            starttime,
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags,
        };
        test_post_stream_validate(create_stream, stream_models::MSG_TAG_NAME_MAX_LENGTH).await;
    }
    #[test]
    async fn test_post_stream_valid_data() {
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
        let stream1b_dto = stream_orm.stream_info_vec.get(0).unwrap().clone();

        // POST api/post_stream
        let request = test::TestRequest::post().uri("/streams").set_json(CreateStreamInfoDto {
            title: stream.title.to_string(),
            descript: None,
            logo: None,
            starttime: stream.starttime.clone(),
            live: None,
            state: None,
            started: None,
            stopped: None,
            status: None,
            source: None,
            tags: stream.tags.clone(),
        });

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
        assert_eq!(stream_dto_res.starttime, stream1b_dto_ser.starttime);
        assert_eq!(stream_dto_res.live, stream1b_dto_ser.live);
        assert_eq!(stream_dto_res.state, stream1b_dto_ser.state);
        assert_eq!(stream_dto_res.started, stream1b_dto_ser.started);
        assert_eq!(stream_dto_res.stopped, stream1b_dto_ser.stopped);
        assert_eq!(stream_dto_res.status, stream1b_dto_ser.status);
        assert_eq!(stream_dto_res.source, stream1b_dto_ser.source);
        assert_eq!(stream_dto_res.tags, stream1b_dto_ser.tags);
        assert_eq!(stream_dto_res.is_my_stream, stream1b_dto_ser.is_my_stream);
        // DateTime.to_rfc3339_opts(SecondsFormat::Secs, true) => "2018-01-26T18:30:09Z"
        let res_created_at = stream_dto_res.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let ser_created_at = stream1b_dto_ser.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_created_at, ser_created_at);
        let res_updated_at = stream_dto_res.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let ser_updated_at = stream1b_dto_ser.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        assert_eq!(res_updated_at, ser_updated_at);
        assert_eq!(stream_dto_res.id, stream1b_dto_ser.id);
    }

    // ** put_stream **

    #[test]
    async fn test_put_stream_no_data() {
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

        // PUT api/streams/{id}
        let request = test::TestRequest::put().uri(&format!("/streams/{}", stream_dto.id));

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
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

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_id_bad))
            .set_json(json!({}));

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

    async fn test_put_stream_validate(modify_stream: ModifyStreamInfoDto, err_msg: &str) {
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

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id))
            .set_json(modify_stream);

        let vec = (vec![user1], vec![session1], vec![stream_dto]);
        let factory = put_stream;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST); // 400

        let body = test::read_body(resp).await;
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err_vec.len(), 1);
        let app_err = app_err_vec.get(0).unwrap();
        assert_eq!(app_err.code, err::CD_VALIDATION);
        assert_eq!(app_err.message, err_msg);
    }
    #[test]
    async fn test_put_stream_no_required_fields() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_NO_REQUIRED_FIELDS).await;
    }
    #[test]
    async fn test_put_stream_title_empty() {
        let modify_stream = ModifyStreamInfoDto {
            title: Some("".to_string()),
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TITLE_REQUIRED).await;
    }
    #[test]
    async fn test_put_stream_title_min() {
        let modify_stream = ModifyStreamInfoDto {
            title: Some(StreamModelsTest::title_min()),
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TITLE_MIN_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_title_max() {
        // let starttime = Utc::now();
        // let tags: Vec<String> = vec!["tag11".to_string(), "tag14".to_string()];
        let modify_stream = ModifyStreamInfoDto {
            title: Some(StreamModelsTest::title_max()),
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TITLE_MAX_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_descript_min() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: Some(StreamModelsTest::descript_min()),
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_DESCRIPT_MIN_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_descript_max() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: Some(StreamModelsTest::descript_max()),
            logo: None,
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_DESCRIPT_MAX_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_logo_min() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: Some(StreamModelsTest::logo_min()),
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_LOGO_MIN_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_logo_max() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: Some(StreamModelsTest::logo_max()),
            starttime: None,
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_LOGO_MAX_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_starttime_past() {
        let dt = Utc::now() - Duration::days(1);
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: Some(dt),
            source: None,
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_CANNOT_SET_PAST_START_DATE).await;
    }
    #[test]
    async fn test_put_stream_source_min() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: Some(StreamModelsTest::source_min()),
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_SOURCE_MIN_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_source_max() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: Some(StreamModelsTest::source_max()),
            tags: None,
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_SOURCE_MAX_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_tags_min_amount() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: Some(StreamModelsTest::tag_names_min()),
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TAG_NAME_MIN_AMOUNT).await;
    }
    #[test]
    async fn test_put_stream_tags_max_amount() {
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: Some(StreamModelsTest::tag_names_max()),
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TAG_NAME_MAX_AMOUNT).await;
    }
    #[test]
    async fn test_put_stream_tag_name_empty() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push("".to_string());
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: Some(tags),
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TAG_NAME_REQUIRED).await;
    }
    #[test]
    async fn test_put_stream_tag_name_min() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_min());
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: Some(tags),
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TAG_NAME_MIN_LENGTH).await;
    }
    #[test]
    async fn test_put_stream_tag_name_max() {
        let mut tags: Vec<String> = StreamModelsTest::tag_names_min();
        tags.push(StreamModelsTest::tag_name_max());
        let modify_stream = ModifyStreamInfoDto {
            title: None,
            descript: None,
            logo: None,
            starttime: None,
            source: None,
            tags: Some(tags),
        };
        test_put_stream_validate(modify_stream, stream_models::MSG_TAG_NAME_MAX_LENGTH).await;
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

        // PUT api/streams/{id}
        let request = test::TestRequest::put()
            .uri(&format!("/streams/{}", stream_dto.id + 1))
            .set_json(ModifyStreamInfoDto {
                title: Some("title2".to_string()),
                descript: Some("descript2".to_string()),
                logo: None,
                starttime: Some(now + Duration::days(1)),
                source: Some(format!("{}_2", stream_models::STREAM_SOURCE_DEF.to_string())),
                tags: Some(vec!["tag11".to_string(), "tag14".to_string()]),
            });

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
    async fn test_put_stream_valid_data() {
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

        // PUT api/streams/{id}
        let request =
            test::TestRequest::put()
                .uri(&format!("/streams/{}", stream_dto.id))
                .set_json(ModifyStreamInfoDto {
                    title: Some(stream2_dto.title.to_string()),
                    descript: Some(stream2_dto.descript.to_string()),
                    logo: None,
                    starttime: Some(stream2_dto.starttime.clone()),
                    source: Some(stream2_dto.source.to_string()),
                    tags: Some(stream2_dto.tags.clone()),
                });

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

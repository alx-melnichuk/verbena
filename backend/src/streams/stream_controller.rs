use actix_web::{get, web, HttpResponse};
// use actix_web::{get, post, put, web, HttpResponse};
use std::{ops::Deref, time::Instant};

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{stream_models, stream_orm::StreamOrm};
use crate::utils::parser::{parse_i32, CD_PARSE_INT_ERROR};
// use crate::validators::{msg_validation, Validator};

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/streams/{id}
    cfg.service(get_stream_by_id)
        // GET api/streams
        .service(get_streams)
        // // POST api/streams
        // .service(create_stream)
        // // PUT api/streams/{id}
        // .service(update_stream)
        ;
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

// GET api/streams/{id}
#[rustfmt::skip]
#[get("/streams/{id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())" )]
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

    let stream_orm2 = stream_orm.clone();
    let res_stream = web::block(move || {
        // Find 'stream' by id.
        let stream_opt =
            stream_orm2.find_stream_by_id(id, curr_user_id).map_err(|e| err_database(e.to_string()));
        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let stream_tag_dto_opt = match res_stream { Ok(v) => v, Err(e) => return Err(e) };

    log::info!("get_stream_by_id() elapsed time: {:.2?}", now.elapsed());
    if let Some(stream_tag_dto) = stream_tag_dto_opt {
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
#[get("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())" )]
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
    // dbg!(&search_stream_info_dto);

    let stream_orm2 = stream_orm.clone();
    let res_streams = web::block(move || {
        // A query to obtain a list of "streams" based on the specified search parameters.
        let res_streams =
            stream_orm2.find_streams(search_stream_info_dto, curr_user_id).map_err(|e| err_database(e.to_string()));
            res_streams
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;

    log::info!("get_streams() elapsed time: {:.2?}", now.elapsed());
    let stream_dto_vec = match res_streams { Ok(v) => v, Err(e) => return Err(e) };

    Ok(HttpResponse::Ok().json(stream_dto_vec)) // 200
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
// #[rustfmt::skip]
// #[post("/streams", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())" )]
/*pub async fn post_stream(
    authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    json_body: web::Json<stream_models::CreateStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    let now = Instant::now();
    let user = authenticated.deref();
    let user_id = user.id;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let create_stream_info: stream_models::CreateStreamInfoDto = json_body.0.clone();
    let create_stream = stream_models::CreateStream::convert(create_stream_info.clone(), user_id);
    let tags = create_stream_info.tags.clone();

    let stream_orm2 = stream_orm.clone();
    let res_stream = web::block(move || {
        // Add a new entity (stream).
        let stream_opt =
            stream_orm2.create_stream(create_stream)
            .map_err(|e| err_database(e.to_string()));
        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let stream = res_stream?;
    let id = stream.id;

    let stream_orm2 = stream_orm.clone();
    // Update a list of "stream_tags" for the entity (stream).
    let res_tags = stream_orm2.update_stream_tags(id, user_id, tags.clone())
        .map_err(|e| err_database(e.to_string()));

    if let Err(err) = res_tags {
        // If an error occurred when adding "stream tags", then delete the "stream".
        let _ = stream_orm.delete_stream(id).map_err(|e| err_database(e.to_string()));
        return Err(err);
    }
    let result = stream_models::StreamInfoDto::convert(stream, user_id, tags);
    log::info!("post_stream() elapsed time: {:.2?}", now.elapsed());

    Ok(HttpResponse::Ok().json(result)) // 200
}*/

// PUT api/streams/{id}
// #[rustfmt::skip]
// #[put("/streams/{id}", /*wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())"*/ )]
/*pub async fn update_stream(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
    json_body: web::Json<stream_models::ModifyStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    // let user = authenticated.deref();
    // let user_id = user.id;
    let user_id = 182;

    // Checking the validity of the data model.
    let validation_res = json_body.validate();
    if let Err(validation_errors) = validation_res {
        log::error!("{}: {}", err::CD_VALIDATION, msg_validation(&validation_errors));
        return Ok(AppError::validations_to_response(validation_errors));
    }

    let modify_stream_info: stream_models::ModifyStreamInfoDto = json_body.0.clone();
    let modify_stream = stream_models::ModifyStream::convert(modify_stream_info.clone(), user_id);
    // let tags = modify_stream_info.tags.clone();

    let stream_orm2 = stream_orm.clone();
    let res_stream = web::block(move || {
        // Start transaction
        // let mut is_commit = false;
        // eprintln!("Start transaction");

        // Modify an entity (stream).
        let res_stream_opt =
            stream_orm2.modify_stream(id, modify_stream)
            .map_err(|e| err_database(e.to_string()));

        /*let res_stream_opt2 = res_stream_opt.clone();
        if let Ok(opt_stream) = res_stream_opt2 {
            if opt_stream.is_some() {
                let stream_orm2 = stream_orm.clone();
                // Update a list of "stream_tags" for the entity (stream).
                let res_tags = stream_orm2.update_stream_tags(id, user_id, tags.clone())
                    .map_err(|e| err_database(e.to_string()));
                if res_tags.is_ok() {
                    // Commit transaction
                    is_commit = true;
                    eprintln!("Commit transaction");
                }
            }
        }*/
        // if !is_commit {
        //     // Rollback transaction
        //     eprintln!("Rollback transaction");
        // }
        res_stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let opt_stream = res_stream?;

    if let Some(stream) = opt_stream {

        // let stream_orm2 = stream_orm.clone();
        // // Update a list of "stream_tags" for the entity (stream).
        // let res_tags = stream_orm2.update_stream_tags(id, user_id, tags.clone())
        //     .map_err(|e| err_database(e.to_string()));
        let tags = modify_stream_info.tags.clone();
        let stream_tag_dto = stream_models::StreamInfoDto::convert(stream, user_id, tags);

        Ok(HttpResponse::Ok().json(stream_tag_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
    }
}*/

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{dev, http, test, test::TestRequest, web, App};
    use chrono::{DateTime, Duration, Utc};

    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::streams::{
        stream_models::{SearchStreamInfoResponseDto, Stream, StreamInfoDto},
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
        let tags1: Vec<&str> = tags.split(',').collect();
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
}

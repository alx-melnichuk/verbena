use actix_web::{get, web, HttpResponse};
// use actix_web::{get, post, put, web, HttpResponse};
use std::ops::Deref;

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
    // Get data from request.
    let id_str = request.match_info().query("id").to_string();
    let id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    let user = authenticated.deref();
    let user_id = user.id;

    let stream_orm2 = stream_orm.clone();
    let res_stream = web::block(move || {
        // Find 'stream' by id.
        let stream_opt =
            stream_orm2.find_stream_by_id(id, user_id).map_err(|e| err_database(e.to_string()));
        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let stream_tag_dto_opt = match res_stream { Ok(v) => v, Err(e) => return Err(e) };

    if let Some(stream_tag_dto) = stream_tag_dto_opt {

        /*let res_stream_tags= web::block(move || {
            // Find 'stream_tag' by stream_id.
            let stream_tags =
                stream_orm.find_stream_tags(user_id, stream_id)
                .map_err(|e| err_database(e.to_string()));
            stream_tags
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;
        
        let stream_tags = match res_stream_tags { Ok(v) => v, Err(e) => return Err(e) };
        */
        // let tags = vec![];
        // let stream_tag_dto = stream_models::StreamInfoDto::convert(stream, user_id, &tags);

        Ok(HttpResponse::Ok().json(stream_tag_dto)) // 200
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204
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
// #[rustfmt::skip]
// #[post("/streams", /*wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())"*/ )]
/*pub async fn create_stream(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    json_body: web::Json<stream_models::CreateStreamInfoDto>,
) -> actix_web::Result<HttpResponse, AppError> {
    // let user = authenticated.deref();
    // let user_id = user.id;
    let user_id = 182;

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
    use chrono::Utc;

    // use crate::errors::AppError;
    use crate::extractors::authentication::BEARER;
    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp,
        tokens::encode_token,
    };
    use crate::streams::stream_models::{Stream, StreamInfoDto};
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };
    // use crate::utils::parser::{CD_PARSE_INT_ERROR, MSG_PARSE_INT_ERROR};

    use super::*;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    fn create_user() -> User {
        let mut user =
            UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwdT1R1");
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
    // fn create_stream(user_id: i32, title: &str) -> Stream {
    //     StreamOrmApp::new_stream(1, user_id, title, Utc::now())
    // }
    fn create_stream_with_id(stream: Stream, tags: &str) -> Stream {
        let stream_orm = StreamOrmApp::create(&vec![(stream, tags)]);
        stream_orm.stream_vec.get(0).unwrap().clone()
    }

    async fn call_service1(
        config_jwt: config_jwt::ConfigJwt,
        vec: (Vec<User>, Vec<Session>, Vec<(Stream, &str)>),
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
    async fn test_get_stream_by_id_valid_id() {
        let user1: User = user_with_id(create_user());
        // let user1b_dto = UserDto::from(user1.clone());

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Create token values.
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let title = "title1";
        let tags = "tag11,tag12";
        let stream0 = StreamOrmApp::new_stream(1, user1.id, title, Utc::now());
        let stream1 = create_stream_with_id(stream0, tags);
        let stream1_id = stream1.id;
        let tag1: Vec<&str> = tags.split(',').collect();
        let stream1b_dto = StreamInfoDto::convert(stream1.clone(), user1.id, &tag1);

        // GET api/streams/{id}
        let request = test::TestRequest::get().uri(&format!("/streams/{}", stream1_id));
        let vec = (vec![user1], vec![session1], vec![(stream1, tags)]);
        let factory = get_stream_by_id;
        let resp = call_service1(config_jwt, vec, &token, factory, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK); // 200

        let body = test::read_body(resp).await;
        eprintln!("\n###### body: {:?}\n", &body);
        // ###### body: b"{\"errCode\":\"InternalServerError\",\"errMsg\":\"user_not_received_from_request\"}"

        let stream_dto_res: StreamInfoDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        eprintln!("\n stream_dto_res: {:?}\n", &stream_dto_res);

        let json_stream1b_dto = serde_json::json!(stream1b_dto).to_string();
        eprintln!("\n json_stream1b_dto: {:?}\n", &json_stream1b_dto);
        let stream1b_dto_ser: StreamInfoDto =
            serde_json::from_slice(json_stream1b_dto.as_bytes()).expect(MSG_FAILED_DESER);
        eprintln!("\n stream1b_dto_ser: {:?}\n", &stream1b_dto_ser);

        assert_eq!(stream_dto_res, stream1b_dto_ser);
    }
}

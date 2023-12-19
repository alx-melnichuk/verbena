// use actix_web::{delete, get, put, web, HttpResponse};
use actix_web::{get, post, web, HttpResponse};
// use std::ops::Deref;

use crate::errors::AppError;
// use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
// #[cfg(not(feature = "mockdata"))]
// use crate::streams::stream_tag_orm::inst::StreamTagOrmApp;
// #[cfg(feature = "mockdata")]
// use crate::streams::stream_tag_orm::tests::StreamTagOrmApp;
use crate::streams::{stream_models, stream_orm::StreamOrm};
use crate::utils::parser::{parse_i32, CD_PARSE_INT_ERROR};
use crate::validators::{msg_validation, Validator};

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/streams/{stream_id}
    cfg.service(get_stream_by_stream_id)
        // POST api/streams
        .service(create_stream);
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

// GET api/streams/{stream_id}
#[rustfmt::skip]
#[get("/streams/{stream_id}", /*wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())"*/ )]
pub async fn get_stream_by_stream_id(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
    let id_str = request.match_info().query("stream_id").to_string();
    let stream_id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    // let user = authenticated.deref();
    // let user_id = user.id;
    let user_id = 182;

    let stream_orm2 = stream_orm.clone();
    let result_stream = web::block(move || {
        // Find 'stream' by id.
        let stream_opt =
            stream_orm2.find_stream(stream_id).map_err(|e| err_database(e.to_string())).ok()?;

        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    if let Some(stream) = result_stream {

        let stream_tags_opt = web::block(move || {
            // Find 'stream_tag' by stream_id.
            let stream_tags_opt =
                stream_orm.find_stream_tag_names(user_id, stream_id)
                .map_err(|e| err_database(e.to_string())).ok();
    
            stream_tags_opt
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;
        
        let mut stream_tag_dto = stream_models::StreamInfoDto::convert(stream, user_id, vec![]);

        if let Some(stream_tags) = stream_tags_opt {
            stream_tag_dto.tags.extend(stream_tags);
        }

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
#[rustfmt::skip]
#[post("/streams", /*wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())"*/ )]
pub async fn create_stream(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    // request: actix_web::HttpRequest,
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
    let stream_tags = create_stream_info.tags.clone();

    let create_stream = stream_models::CreateStream::convert(create_stream_info, user_id);
    
    let stream_orm2 = stream_orm.clone();
    let result_stream = web::block(move || {
        // Add a new entity (stream).
        let stream_opt =
            stream_orm2.create_stream(create_stream)
            .map_err(|e| err_database(e.to_string())).ok();

        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    let stream = result_stream.unwrap();
    let stream_id = stream.id;

    // Add a list of "stream_tags" for the entity (stream).
    let _ = stream_orm.add_stream_tag(user_id, stream_id, stream_tags.clone())
        .map_err(|e| err_database(e.to_string()));
    
    let result = stream_models::StreamInfoDto::convert(stream, user_id, stream_tags);

    Ok(HttpResponse::Ok().json(result)) // 200
}
/*
// GET api/streams/{stream_id}
#[rustfmt::skip]
// #[get("/streams/{stream_id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())" )]
#[get("/streams/{stream_id}")]
pub async fn get_stream_by_stream_id2(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
    let id_str = request.match_info().query("stream_id").to_string();
    let stream_id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    // let user = authenticated.deref();
    // let user_id = user.id;
    let user_id = 182;
}*/

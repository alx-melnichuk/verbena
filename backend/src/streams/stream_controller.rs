// use actix_web::{delete, get, put, web, HttpResponse};
use actix_web::{get, web, HttpResponse};
use std::ops::Deref;

use crate::errors::AppError;
use crate::extractors::authentication::{Authenticated, RequireAuth};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::inst::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_tag_orm::inst::StreamTagOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_tag_orm::tests::StreamTagOrmApp;
use crate::streams::{stream_models, stream_orm::StreamOrm, stream_tag_orm::StreamTagOrm};
use crate::utils::parser::{parse_i32, CD_PARSE_INT_ERROR};

pub fn configure(cfg: &mut web::ServiceConfig) {
    //     GET api/streams/{stream_id}
    cfg.service(get_stream_by_id_user_id);
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

/* Name: 'Get stream'
    * @route streams/:streamId
    * @example streams/385e0469-7143-4915-88d0-f23f5b27ed36
    * @type get
    * @params streamId
    * @required streamId
    * @access public
@Get(':streamId')
@Public()
async getStream (@Req() request: RequestSession, @Param('streamId', new ParseUUIDPipe()) streamId: string): Promise<StreamDTO> {
    try {
        if (request.get('auth')) {
            const userData = await this.jwtService.verify(request.get('auth'));
            request.user = new UserSession(userData);
        }
    } catch (e) {
        request.user = null;
    }
    return await this.streamsService.getStream(request.user, streamId);
}*/
// GET api/streams/{stream_id}
#[rustfmt::skip]
// #[get("/streams/{stream_id}", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())" )]
#[get("/streams/{stream_id}")]
pub async fn get_stream_by_id_user_id(
    // authenticated: Authenticated,
    stream_orm: web::Data<StreamOrmApp>,
    stream_tag_orm: web::Data<StreamTagOrmApp>,
    request: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse, AppError> {
    // Get data from request.
    let id_str = request.match_info().query("stream_id").to_string();
    let stream_id = parse_i32(&id_str).map_err(|e| err_parse_int(e.to_string()))?;

    // let user = authenticated.deref();
    // let user_id = user.id;
    let user_id = 162;

    let result_stream = web::block(move || {
        // Find 'stream' by id.
        let stream_opt =
            stream_orm.find_stream_by_id(stream_id).map_err(|e| err_database(e.to_string())).ok()?;

        stream_opt
    })
    .await
    .map_err(|e| err_blocking(e.to_string()))?;

    if let Some(stream) = result_stream {

        let result_stream_tags = web::block(move || {
            // Find 'stream_tag' by stream_id.
            let stream_tags_opt =
                stream_tag_orm.find_stream_tag_by_user_id_stream_id(user_id, stream_id)
                // ) -> Result<Vec<StreamTag>, String> {
                // find_stream_tag_by_user_id_stream_id(user_id, stream_id)
                .map_err(|e| err_database(e.to_string())).ok();
    
            stream_tags_opt
        })
        .await
        .map_err(|e| err_blocking(e.to_string()))?;
        
        let mut stream_tag_dto = stream_models::StreamTagDto::from(stream);

        if let Some(stream_tags) = result_stream_tags {
            let tags: Vec<String> = stream_tags.iter().map(|stream_tag| stream_tag.name.to_owned()).collect();
            stream_tag_dto.tags.extend(tags);
        }

        Ok(HttpResponse::Ok().json(stream_tag_dto))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

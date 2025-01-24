use actix_web::web;
use log;

use crate::errors::AppError;
use crate::settings::err;
use crate::stream_controller::{remove_image_file, ALIAS_LOGO_FILES_DIR};
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{config_strm, stream_orm::StreamOrm};

/** Get a list of logo file names for streams of the user with the specified user_id. */
pub async fn get_stream_logo_files(stream_orm: web::Data<StreamOrmApp>, user_id: i32) -> Result<Vec<String>, AppError> {
    let opt_id: Option<i32> = None;
    let opt_user_id: Option<i32> = Some(user_id);
    let opt_is_logo: Option<bool> = Some(true);
    let opt_live: Option<bool> = None;
    // Get a list of streams for a user (user_id) that have logo files (is_logo = true).
    let result_data = web::block(move || {
        // Filter entities (streams) by specified parameters.
        let res_data = stream_orm
            .filter_streams_by_params(opt_id, opt_user_id, opt_is_logo, opt_live, false, &[])
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_data
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        AppError::blocking506(&e.to_string())
    })?;

    // Extract data from the result.
    let (streams, _stream_tags) = match result_data {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    // Get a list of logo file names from a list of streams.
    let path_file_img_list: Vec<String> = streams
        .into_iter()
        .filter(|stream| stream.logo.is_some() && stream.logo.clone().unwrap_or(String::new()).len() > 0)
        .map(|stream| stream.logo.unwrap_or(String::new()))
        .collect();

    Ok(path_file_img_list)
}

/** Delete all specified logo files in the given list. */
pub fn remove_stream_logo_files(path_file_img_list: Vec<String>, config_strm: config_strm::ConfigStrm) {
    let img_file_dir = config_strm.strm_logo_files_dir;
    // Remove files from the resulting list of stream logo files.
    for path_file_img in path_file_img_list {
        // Delete a file if its path starts with the specified alias.
        #[rustfmt::skip]
        remove_image_file(&path_file_img, ALIAS_LOGO_FILES_DIR, &img_file_dir, &"remove_stream_logo_files()");
    }
}

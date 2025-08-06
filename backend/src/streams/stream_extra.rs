use actix_web::{http::StatusCode, web};
use log::error;
use vrb_common::api_error::{code_to_str, ApiError};
use vrb_tools::err;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm,
    stream_controller::{remove_image_file, ALIAS_LOGO_FILES_DIR},
    stream_orm::StreamOrm,
};

/** Get a list of logo file names for streams of the user with the specified user_id. */
pub async fn get_stream_logo_files(stream_orm: web::Data<StreamOrmApp>, user_id: i32) -> Result<Vec<String>, ApiError> {
    let opt_id: Option<i32> = None;
    let opt_user_id: Option<i32> = Some(user_id);
    let opt_is_logo: Option<bool> = Some(true);
    let opt_live: Option<bool> = None;
    // Get a list of streams for a user (user_id) that have logo files (is_logo = true).
    let result_data = web::block(move || {
        // Filter entities (streams) by specified parameters.
        let res_data = stream_orm
            .filter_streams_by_params(opt_id, opt_user_id, opt_is_logo, opt_live, false)
            .map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            });
        res_data
    })
    .await
    .map_err(|e| {
        #[rustfmt::skip]
        error!("{}-{}; {}", code_to_str(StatusCode::VARIANT_ALSO_NEGOTIATES), err::MSG_BLOCKING, &e.to_string());
        ApiError::create(506, err::MSG_BLOCKING, &e.to_string()) // 506
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
pub fn remove_stream_logo_files(path_file_img_list: Vec<String>, config_strm: config_strm::ConfigStrm) -> usize {
    let result = path_file_img_list.len();
    let img_file_dir = config_strm.strm_logo_files_dir;
    // Remove files from the resulting list of stream logo files.
    for path_file_img in path_file_img_list {
        // Delete a file if its path starts with the specified alias.
        #[rustfmt::skip]
        remove_image_file(&path_file_img, ALIAS_LOGO_FILES_DIR, &img_file_dir, &"remove_stream_logo_files()");
    }
    result
}

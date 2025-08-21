use actix_web::{http::StatusCode, web};
use log::error;
use vrb_common::{alias_path::alias_path_stream, api_error::{code_to_str, ApiError}};
use vrb_tools::err;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
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
pub fn remove_stream_logo_files(path_file_img_list: &[String], strm_logo_files_dir: &str) -> usize {
    let mut result = 0;
    let alias_path_strm = alias_path_stream::AliasStrm::new(strm_logo_files_dir);
    let alias_strm = alias_path_strm.as_ref();

    // Remove files from the resulting list of stream logo files.
    for path_file_img in path_file_img_list {
        // If the file path starts with alice, then the file corresponds to the entity type.
        // And only then can the file be deleted.
        if !alias_strm.starts_with_alias(path_file_img) {
            continue;
        }
        // If the image file name starts with the specified alias, then delete the file.
        // Return file path prefix instead of alias.
        let full_path_file_img = alias_strm.alias_to_path(&path_file_img);
        let res_remove = std::fs::remove_file(&full_path_file_img);
        if let Err(err) = res_remove {
            error!("remove_stream_logo_files() remove_file({}): error: {:?}", &full_path_file_img, err);
        } else {
            result += 1;
        }
    }
    result
}

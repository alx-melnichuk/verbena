use actix_web::web;
use log::error;

use crate::errors::AppError;
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::streams::stream_orm::impls::StreamOrmApp;
#[cfg(feature = "mockdata")]
use crate::streams::stream_orm::tests::StreamOrmApp;
use crate::streams::{
    config_strm,
    stream_controller::{remove_image_file, ALIAS_LOGO_FILES_DIR},
    stream_orm::StreamOrm,
};

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
                error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                AppError::database507(&e)
            });
        res_data
    })
    .await
    .map_err(|e| {
        error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
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

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{fs, path};

    use actix_web::web;
    use chrono::Utc;

    use crate::streams::{
        config_strm,
        stream_controller::{tests::create_stream, ALIAS_LOGO_FILES_DIR},
        stream_orm::tests::StreamOrmApp,
    };

    use super::{get_stream_logo_files, remove_stream_logo_files};

    pub fn save_empty_file(path_file: &str) -> Result<String, String> {
        let _ = fs::File::create(path_file).map_err(|e| e.to_string())?;
        Ok(path_file.to_string())
    }

    // ** get_stream_logo_files **

    #[actix_web::test]
    async fn test_get_stream_logo_files_another_user() {
        let name0_file = "test_get_stream_logo_files_another_user.png";
        let profile0_id = 0;
        let mut stream = create_stream(0, profile0_id, "title0", "tag01,tag02", Utc::now());
        stream.logo = Some(format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file));

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&[stream]));
        let result = get_stream_logo_files(data_stream_orm, profile0_id + 1).await;

        assert!(result.is_ok());
        let path_files = result.unwrap();
        assert_eq!(path_files.len(), 0);
    }
    #[actix_web::test]
    async fn test_get_stream_logo_files_without_files() {
        let profile0_id = 0;
        let stream = create_stream(0, profile0_id, "title0", "tag01,tag02", Utc::now());

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&[stream]));
        let result = get_stream_logo_files(data_stream_orm, profile0_id).await;

        assert!(result.is_ok());
        let path_files = result.unwrap();
        assert_eq!(path_files.len(), 0);
    }
    #[actix_web::test]
    async fn test_get_stream_logo_files_with_files() {
        let name0_file = "test_get_stream_logo_files_with_files.png";
        let path_name0_alias = format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file);

        let profile0_id = 0;

        let mut stream = create_stream(0, profile0_id, "title0", "tag01,tag02", Utc::now());
        stream.logo = Some(path_name0_alias.clone());

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&[stream]));
        let result = get_stream_logo_files(data_stream_orm, profile0_id).await;

        assert!(result.is_ok());
        let path_files = result.unwrap();
        assert_eq!(path_files.len(), 1);
        assert_eq!(path_files.get(0), Some(&path_name0_alias));
    }

    // ** remove_stream_logo_files **

    #[actix_web::test]
    async fn test_remove_stream_logo_files_empty_list() {
        let config_strm = config_strm::get_test_config();
        let path_file_img_list: Vec<String> = vec![];

        let res = remove_stream_logo_files(path_file_img_list, config_strm);
        assert_eq!(res, 0);
    }
    #[actix_web::test]
    async fn test_remove_stream_logo_files_not_empty_list() {
        let config_strm = config_strm::get_test_config();

        let name0_file = "test_remove_stream_logo_files_not_empty_list.png";
        let path_name0_file = format!("{}/{}", &config_strm.strm_logo_files_dir, name0_file);
        save_empty_file(&path_name0_file).unwrap();

        let path_name0_alias = path_name0_file.replace(&config_strm.strm_logo_files_dir, ALIAS_LOGO_FILES_DIR);

        let path_file_img_list: Vec<String> = vec![path_name0_alias];

        let res = remove_stream_logo_files(path_file_img_list, config_strm);
        assert_eq!(res, 1);
        let is_exists_logo_file = path::Path::new(&path_name0_file).exists();
        assert!(!is_exists_logo_file);
    }
}

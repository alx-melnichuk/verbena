#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{fs, path};

    use actix_web::web;

    use vrb_dbase::user_auth::user_auth_orm::tests::{UserAuthOrmTest as User_Test, USER, USER1};

    use crate::streams::{
        config_strm,
        stream_controller::ALIAS_LOGO_FILES_DIR,
        stream_extra::{get_stream_logo_files, remove_stream_logo_files},
        stream_orm::tests::{StreamOrmApp, StreamOrmTest as Strm_Test},
    };

    pub fn save_empty_file(path_file: &str) -> Result<String, String> {
        let _ = fs::File::create(path_file).map_err(|e| e.to_string())?;
        Ok(path_file.to_string())
    }

    // ** get_stream_logo_files **

    #[actix_web::test]
    async fn test_get_stream_logo_files_another_user() {
        let name0_file = "test_get_stream_logo_files_another_user.png";
        let data_u = User_Test::users(&[USER]);
        let user0_id = data_u.0.get(0).unwrap().id;

        let mut streams = Strm_Test::streams(&[USER1]);
        let stream = streams.get_mut(0).unwrap();
        stream.logo = Some(format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file));

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&streams));
        let result = get_stream_logo_files(data_stream_orm, user0_id + 1).await;

        assert!(result.is_ok());
        let path_files = result.unwrap();
        assert_eq!(path_files.len(), 0);
    }
    #[actix_web::test]
    async fn test_get_stream_logo_files_without_files() {
        let data_u = User_Test::users(&[USER]);
        let user0_id = data_u.0.get(0).unwrap().id;
        let streams = Strm_Test::streams(&[USER1]);

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&streams));
        let result = get_stream_logo_files(data_stream_orm, user0_id).await;

        assert!(result.is_ok());
        let path_files = result.unwrap();
        assert_eq!(path_files.len(), 0);
    }
    #[actix_web::test]
    async fn test_get_stream_logo_files_with_files() {
        let name0_file = "test_get_stream_logo_files_with_files.png";
        let path_name0_alias = format!("{}/{}", ALIAS_LOGO_FILES_DIR, name0_file);

        let data_u = User_Test::users(&[USER]);
        let user0_id = data_u.0.get(0).unwrap().id;

        let mut streams = Strm_Test::streams(&[USER1]);
        let stream = streams.get_mut(0).unwrap();
        stream.logo = Some(path_name0_alias.clone());

        let data_stream_orm: web::Data<StreamOrmApp> = web::Data::new(StreamOrmApp::create(&streams));
        let result = get_stream_logo_files(data_stream_orm, user0_id).await;

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

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::{fs, path};

    use actix_web::{
        body, dev,
        http::header::{HeaderValue, CONTENT_TYPE},
        http::StatusCode,
        test, App,
    };
    use serde_json;
    use vrb_tools::{
        api_error::{code_to_str, ApiError},
        err, png_files,
        token_data::header_auth,
    };

    use crate::profiles::{
        config_prfl,
        profile_controller::{delete_profile, delete_profile_current, ALIAS_AVATAR_FILES_DIR},
        profile_models::ProfileDto,
        profile_orm::tests::{ProfileOrmTest as PrfTest, ADMIN, USER},
    };
    use crate::users::user_registr_orm::tests::UserRegistrOrmTest as RegTest;

    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // ** delete_profile **

    #[actix_web::test]
    async fn test_delete_profile_invalid_id() {
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile_id_bad = format!("{}a", data_p.0.get(0).unwrap().user_id);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id_bad))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE); // 416

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err: ApiError = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        assert_eq!(app_err.code, code_to_str(StatusCode::RANGE_NOT_SATISFIABLE));
        #[rustfmt::skip]
        let msg = format!("{}; `id` - invalid digit found in string ({})", err::MSG_PARSING_TYPE_NOT_SUPPORTED, profile_id_bad);
        assert_eq!(app_err.message, msg);
    }
    #[actix_web::test]
    async fn test_delete_profile_non_existent_id() {
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile_id = data_p.0.get(0).unwrap().user_id;
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile_id + 1))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT); // 204
    }
    #[actix_web::test]
    async fn test_delete_profile_existent_id() {
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get(0).unwrap().clone();
        let profile1_id = profile1.user_id;
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir; // "./tmp"

        let name0_file = "test_delete_profile_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir; // "./tmp"

        let name0_file = "test_delete_profile_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_with_stream_img() {
        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        let path_logo0_file = PrfTest::stream_logo_path(profile1_id).unwrap();
        // Create a logo file for this user's stream.
        png_files::save_file_png(&(path_logo0_file.clone()), 1).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri(&format!("/api/profiles/{}", profile1_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_logo0_file).exists();
        let _ = fs::remove_file(&path_logo0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200

        // After deleting a user, the stream logo file should be deleted.
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }

    // ** delete_profile_current **

    #[actix_web::test]
    async fn test_delete_profile_current_without_img() {
        let token = PrfTest::token1();
        let (cfg_p, data_p) = (PrfTest::cfg(), PrfTest::profiles(&[USER]));
        let profile1 = data_p.0.get(0).unwrap().clone();
        let profile1_dto = ProfileDto::from(profile1);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile1_dto).to_string();
        let profile1_dto_ser: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile1_dto_ser);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir; // "./tmp"

        let name0_file = "test_delete_profile_current_with_img.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_img_not_alias() {
        let config_prfl = config_prfl::get_test_config();
        let prfl_avatar_files_dir = config_prfl.prfl_avatar_files_dir; // "./tmp"

        let name0_file = "test_delete_profile_current_with_img_not_alias.png";
        let path_name0_file = format!("{}/{}", &prfl_avatar_files_dir, name0_file);
        png_files::save_file_png(&(path_name0_file.clone()), 1).unwrap();
        let path_name0_alias = format!("/1{}/{}", ALIAS_AVATAR_FILES_DIR, name0_file);

        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        profile1.avatar = Some(path_name0_alias);
        let profile_dto = ProfileDto::from(profile1.clone());
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_name0_file).exists();
        let _ = fs::remove_file(&path_name0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200
        assert!(is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
    #[actix_web::test]
    async fn test_delete_profile_current_with_stream_img() {
        let token = PrfTest::token1();
        let (cfg_p, mut data_p) = (PrfTest::cfg(), PrfTest::profiles(&[ADMIN]));
        let profile1 = data_p.0.get_mut(0).unwrap();
        let profile1_id = profile1.user_id;
        let profile_dto = ProfileDto::from(profile1.clone());
        let path_logo0_file = PrfTest::stream_logo_path(profile1_id).unwrap();
        // Create a logo file for this user's stream.
        png_files::save_file_png(&(path_logo0_file.clone()), 1).unwrap();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(delete_profile_current)
                .configure(PrfTest::config(cfg_p, data_p))
                .configure(RegTest::config(RegTest::registrs(false)))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::delete().uri("/api/profiles_current")
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;

        let is_exists_img_old = path::Path::new(&path_logo0_file).exists();
        let _ = fs::remove_file(&path_logo0_file);

        assert_eq!(resp.status(), StatusCode::OK); // 200

        // After deleting a user, the stream logo file should be deleted.
        assert!(!is_exists_img_old);
        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let profile_dto_res: ProfileDto = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        let json = serde_json::json!(profile_dto).to_string();
        let profile_dto_org: ProfileDto = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(profile_dto_res, profile_dto_org);
    }
}

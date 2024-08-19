use std::{env, io};

use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{
    http,
    web::{self, Data},
};

use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use profiles::{
    config_prfl, profile_auth_controller, profile_controller, profile_orm::cfg::get_profile_orm_app,
    profile_registr_controller,
};
use send_email::{config_smtp, mailer};
use sessions::{config_jwt, session_orm::cfg::get_session_orm_app};
use streams::{config_strm, stream_controller, stream_get_controller, stream_orm::cfg::get_stream_orm_app};
use tools::evn_data::{check_params_env, update_params_env};
use users::{user_recovery_orm::cfg::get_user_recovery_orm_app, user_registr_orm::cfg::get_user_registr_orm_app};
use utils::parser;
use utoipa::OpenApi;

pub mod cdis;
pub(crate) mod dbase;
pub(crate) mod errors;
pub(crate) mod extractors;
pub mod file_upload;
pub(crate) mod hash_tools;
pub mod profiles;
pub(crate) mod schema;
pub(crate) mod send_email;
pub(crate) mod sessions;
pub mod settings;
pub(crate) mod static_controller;
pub mod streams;
pub mod swagger_docs;
pub(crate) mod tools;
pub(crate) mod users;
pub mod utils;
pub mod validators;

pub fn configure_server() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");

        let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
        dbase::run_migration(&mut pool.get().unwrap());

        // Adding various configs.
        let config_app0 = settings::config_app::ConfigApp::init_by_env();
        let temp_file_config0 = TempFileConfig::default().clone().directory(config_app0.app_dir_tmp.clone());

        // used: profile_registr_controller, static_controller
        let config_app = Data::new(config_app0);
        // used: profile_auth_controller, profile_registr_controller
        let config_jwt = Data::new(config_jwt::ConfigJwt::init_by_env());
        // Used "actix-multipart" to upload files. TempFileConfig.from_req()
        let temp_file_config = Data::new(temp_file_config0);
        // used: stream_get_controller, stream_controller
        let config_strm = Data::new(config_strm::ConfigStrm::init_by_env());
        //
        let config_smtp0 = config_smtp::ConfigSmtp::init_by_env();
        // used: stream_controller
        let config_smtp = Data::new(config_smtp0.clone());
        // used: Profile_controller
        let config_prfl = Data::new(config_prfl::ConfigPrfl::init_by_env());

        // Adding various entities.
        // used: profile_registr_controller
        let mailer = Data::new(mailer::cfg::get_mailer_app(config_smtp0));
        // used: profile_registr_controller
        let user_registr_orm = Data::new(get_user_registr_orm_app(pool.clone()));
        // used: profile_registr_controller
        let user_recovery_orm = Data::new(get_user_recovery_orm_app(pool.clone()));
        // used: profile_auth_controller, profile_registr_controller
        let session_orm = Data::new(get_session_orm_app(pool.clone()));
        // used: stream_get_controller, stream_controller
        let stream_orm = Data::new(get_stream_orm_app(pool.clone()));
        // used: profile_controller
        let profile_orm = Data::new(get_profile_orm_app(pool.clone()));

        // Make instance variable of ApiDoc so all worker threads gets the same instance.
        let openapi = swagger_docs::ApiDoc::openapi();

        config
            .app_data(Data::clone(&config_app))
            .app_data(Data::clone(&config_jwt))
            .app_data(Data::clone(&temp_file_config))
            .app_data(Data::clone(&config_strm))
            .app_data(Data::clone(&config_smtp))
            .app_data(Data::clone(&config_prfl))
            .app_data(Data::clone(&mailer))
            .app_data(Data::clone(&user_registr_orm))
            .app_data(Data::clone(&session_orm))
            .app_data(Data::clone(&user_recovery_orm))
            .app_data(Data::clone(&stream_orm))
            .app_data(Data::clone(&profile_orm))
            // Add documentation service "Redoc" and "RapiDoc".
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Add documentation service "SwaggerUi".
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            // Add configuration of internal services.
            .configure(profile_registr_controller::configure())
            .configure(profile_auth_controller::configure())
            .configure(stream_get_controller::configure())
            .configure(stream_controller::configure())
            .configure(profile_controller::configure())
            .configure(static_controller::configure());
    }
}

pub fn create_cors(config_app: settings::config_app::ConfigApp) -> Cors {
    let app_domain = config_app.app_domain;
    // Maximum number of seconds the results can be cached.
    let app_max_age = config_app.app_max_age;

    let mut cors = Cors::default()
        .allowed_origin(&app_domain.to_string())
        // .allowed_origin("https://fonts.googleapis.com")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(app_max_age);

    cors = cors.allowed_origin(&app_domain.clone());
    let cors_allowed_origin: Vec<&str> = config_app.app_allowed_origin.split(',').collect();
    if cors_allowed_origin.len() > 0 {
        for allowed_origin in cors_allowed_origin.into_iter() {
            cors = cors.allowed_origin(allowed_origin.trim());
        }
    }
    cors
}
// List of parameters that are encrypted.
fn get_list_params<'a>() -> &'a [&'a str] {
    &["DATABASE_URL", "SMTP_HOST_PORT", "SMTP_USER_PASS", "JWT_SECRET_KEY"]
}
// Checking the configuration and encrypting the specified parameters.
pub fn check_env() -> Result<usize, String> {
    // SSL private key
    let app_private_key = std::env::var("APP_PRIVATE_KEY").unwrap_or("".to_string());

    if app_private_key.len() > 0 {
        // Checking the configuration and encrypting the specified parameters.
        let result = check_params_env(&"./.env", get_list_params(), &app_private_key, 500);
        let params = result.clone().unwrap_or(vec![]);
        for param in params.iter() {
            std::env::remove_var(param);
        }
        result.map(|v| v.len())
    } else {
        Ok(0)
    }
}
// Update configurations and decryption of specified parameters.
pub fn update_env() -> Result<usize, String> {
    // SSL private key
    let app_private_key = std::env::var("APP_PRIVATE_KEY").unwrap_or("".to_string());

    if app_private_key.len() > 0 {
        // Update configurations and decryption of specified parameters.
        let result = update_params_env(get_list_params(), &app_private_key, 500);
        let params = result.clone().unwrap_or(vec![]);
        let is_show_prm = std::env::var("IS_SHOW_DECRYPTED_PRMS").unwrap_or("false".to_string());
        let is_show_prm = parser::parse_bool(&is_show_prm).unwrap_or(false);
        if is_show_prm && params.len() > 0 {
            println!("Decrypted parameters:");
            for param in params.iter() {
                println!("{}={}", param, std::env::var(param).unwrap_or("".to_string()));
            }
        }
        result.map(|v| v.len())
    } else {
        Ok(0)
    }
}
// Checking the presence of required directories
pub fn check_presence_required_directories() -> Result<(), io::Error> {
    // Check the correctness of "STRM_LOGO_VALID_TYPES"
    let logo_valid_types = config_strm::ConfigStrm::get_logo_valid_types();
    config_strm::ConfigStrm::get_logo_valid_types_by_str(&logo_valid_types)?;

    // Check the correctness of "PRFL_AVATAR_VALID_TYPES"
    let avatar_valid_types = config_prfl::ConfigPrfl::get_avatar_valid_types();
    config_prfl::ConfigPrfl::get_avatar_valid_types_by_str(&avatar_valid_types)?;

    Ok(())
}

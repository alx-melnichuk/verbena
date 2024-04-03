use std::env;

use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{http, web};

use send_email::{config_smtp, mailer};
use sessions::{config_jwt, session_orm::cfg::get_session_orm_app};
use streams::{config_strm, stream_controller, stream_get_controller, stream_orm::cfg::get_stream_orm_app};
use tools::evn_data::{check_params_env, update_params_env};
use users::{
    config_usr, user_auth_controller, user_controller, user_orm::cfg::get_user_orm_app,
    user_recovery_orm::cfg::get_user_recovery_orm_app, user_registr_controller,
    user_registr_orm::cfg::get_user_registr_orm_app,
};
use utils::parser;

pub mod cdis;
pub(crate) mod dbase;
pub(crate) mod errors;
pub(crate) mod extractors;
pub mod file_upload;
pub(crate) mod hash_tools;
pub(crate) mod schema;
pub(crate) mod send_email;
pub(crate) mod sessions;
pub mod settings;
pub(crate) mod static_controller;
pub mod streams;
pub(crate) mod tools;
pub(crate) mod users;
pub mod utils;
pub mod validators;

pub fn configure_server() -> Box<dyn Fn(&mut web::ServiceConfig)> {
    Box::new(move |cfg: &mut web::ServiceConfig| {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");

        let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
        dbase::run_migration(&mut pool.get().unwrap());

        // Adding various configs.
        let config_app = settings::config_app::ConfigApp::init_by_env();
        let temp_file_config = TempFileConfig::default().clone().directory(config_app.app_dir_tmp.clone());

        let data_config_app = web::Data::new(config_app);
        let data_config_jwt = web::Data::new(config_jwt::ConfigJwt::init_by_env());
        let config_smtp = config_smtp::ConfigSmtp::init_by_env();
        let data_config_smtp = web::Data::new(config_smtp.clone());
        // data_config_smtp.get_ref().clone()
        let data_temp_file_config = web::Data::new(temp_file_config);
        let data_config_usr = web::Data::new(config_usr::ConfigUsr::init_by_env());
        let data_config_strm = web::Data::new(config_strm::ConfigStrm::init_by_env());

        // Adding various entities.
        let data_mailer = web::Data::new(mailer::cfg::get_mailer_app(config_smtp));
        let data_user_orm = web::Data::new(get_user_orm_app(pool.clone()));
        let data_user_registr_orm = web::Data::new(get_user_registr_orm_app(pool.clone()));
        let data_session_orm = web::Data::new(get_session_orm_app(pool.clone()));
        let data_user_recovery_orm = web::Data::new(get_user_recovery_orm_app(pool.clone()));
        let data_stream_orm = web::Data::new(get_stream_orm_app(pool.clone()));

        cfg.app_data(web::Data::clone(&data_config_app))
            .app_data(web::Data::clone(&data_config_jwt))
            .app_data(web::Data::clone(&data_config_smtp))
            .app_data(web::Data::clone(&data_temp_file_config))
            .app_data(web::Data::clone(&data_config_usr))
            .app_data(web::Data::clone(&data_config_strm))
            .app_data(web::Data::clone(&data_mailer))
            .app_data(web::Data::clone(&data_user_orm))
            .app_data(web::Data::clone(&data_user_registr_orm))
            .app_data(web::Data::clone(&data_session_orm))
            .app_data(web::Data::clone(&data_user_recovery_orm))
            .app_data(web::Data::clone(&data_stream_orm))
            .configure(static_controller::configure)
            .service(
                web::scope("/api")
                    .configure(user_registr_controller::configure)
                    .configure(user_auth_controller::configure)
                    .configure(user_controller::configure)
                    .configure(stream_get_controller::configure)
                    .configure(stream_controller::configure),
            );
    })
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

    let cors_allowed_origin: Vec<&str> = config_app.app_allowed_origin.split(',').collect();
    if cors_allowed_origin.len() > 0 {
        for allowed_origin in cors_allowed_origin.into_iter() {
            cors = cors.allowed_origin(allowed_origin.trim())
        }
    }
    cors
}
// List of parameters that are encrypted.
fn get_list_params<'a>() -> &'a [&'a str] {
    &["DATABASE_URL", "SMTP_HOST_PORT", "SMTP_USER_PASS"]
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

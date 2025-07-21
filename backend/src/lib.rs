use std::env;

use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{http, middleware, web, App, HttpServer};
use dotenv;
use env_logger;
use log::{info, log_enabled, Level::Info};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use vrb_tools::ssl_acceptor;
#[cfg(not(feature = "mockdata"))]
use vrb_tools::send_email::mailer::impls::MailerApp;
#[cfg(feature = "mockdata")]
use vrb_tools::send_email::mailer::tests::MailerApp;
use vrb_tools::send_email::config_smtp;

use chats::{chat_message_controller, chat_message_orm::cfg::get_chat_message_orm_app, chat_ws_controller};
use profiles::{
    config_prfl, profile_auth_controller, profile_controller, profile_orm::cfg::get_profile_orm_app,
    profile_registr_controller,
};
use sessions::{config_jwt, session_orm::cfg::get_session_orm_app};
use settings::config_app;
use streams::{config_strm, stream_controller, stream_orm::cfg::get_stream_orm_app};
use users::{user_recovery_orm::cfg::get_user_recovery_orm_app, user_registr_orm::cfg::get_user_registr_orm_app};
use utoipa::OpenApi;

pub mod chats;
pub(crate) mod dbase;
pub(crate) mod errors;
pub(crate) mod extractors;
pub mod loading;
pub mod profiles;
pub(crate) mod schema;
pub(crate) mod sessions;
pub mod settings;
pub(crate) mod static_controller;
pub mod streams;
pub mod swagger_docs;
pub(crate) mod users;
pub mod utils;
pub mod validators;

pub async fn server_run() -> std::io::Result<()> {
    #[cfg(feature = "mockdata")]
    #[rustfmt::skip]
    assert!(false, "Launch in \"mockdata\" mode! Disable \"default=[test, mockdata]\" in Cargo.toml.");

    dotenv::dotenv().expect("Failed to read .env file");

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn,actix_web=info,verbena=info");
    }

    env_logger::init();

    let config_app = config_app::ConfigApp::init_by_env();

    let app_protocol = config_app.app_protocol.clone();
    let app_host = config_app.app_host.clone();
    let app_port = config_app.app_port.clone();
    let app_url = format!("{}:{}", &app_host, &app_port);

    // Creating temporary directory.
    std::fs::create_dir_all(config_app.app_dir_tmp.clone())?;

    // Creating a directory for upload the logo.
    let config_strm = config_strm::ConfigStrm::init_by_env();
    std::fs::create_dir_all(&config_strm.strm_logo_files_dir)?;

    // Creating a directory for upload the avatar.
    let config_prfl = config_prfl::ConfigPrfl::init_by_env();
    std::fs::create_dir_all(&config_prfl.prfl_avatar_files_dir)?;

    let app_domain = config_app.app_domain.clone();
    app_log(&format!("Starting server {}", &app_domain));

    let config_app2 = config_app.clone();

    let mut srv = HttpServer::new(move || {
        let cors = create_cors(config_app2.clone());
        App::new()
            .configure(configure_server())
            .wrap(cors)
            .wrap(middleware::Logger::default())
    });

    if config_app::PROTOCOL_HTTP == app_protocol {
        srv = srv.bind(&app_url)?;
    } else {
        let builder =
            ssl_acceptor::create_ssl_acceptor_builder(&config_app.app_certificate, &config_app.app_private_key);
        srv = srv.bind_openssl(&app_url, builder)?;
    }

    if let Some(num_workers) = config_app.app_num_workers {
        let worker_count = std::thread::available_parallelism()?.get();
        #[rustfmt::skip]
        let workers = if num_workers > worker_count { worker_count } else { num_workers };
        srv = srv.workers(workers);
    }

    srv.run().await
}

pub fn configure_server() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        let db_url = env::var("DATABASE_URL").expect("Env \"DATABASE_URL\" not found.");

        app_log("Configuring database.");
        let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
        dbase::run_migration(&mut pool.get().unwrap());

        // Adding various configs.
        let config_app0 = settings::config_app::ConfigApp::init_by_env();
        let temp_file_config0 = TempFileConfig::default().clone().directory(config_app0.app_dir_tmp.clone());

        // used: profile_registr_controller, static_controller
        let config_app = web::Data::new(config_app0);
        // used: profile_auth_controller, profile_registr_controller
        let config_jwt = web::Data::new(config_jwt::ConfigJwt::init_by_env());
        // Used "actix-multipart" to upload files. TempFileConfig.from_req()
        let temp_file_config = web::Data::new(temp_file_config0);
        // used: stream_controller
        let config_strm = web::Data::new(config_strm::ConfigStrm::init_by_env());
        //
        let config_smtp0 = config_smtp::ConfigSmtp::init_by_env();
        // used: stream_controller, profile_controller
        let config_smtp = web::Data::new(config_smtp0.clone());
        // used: profile_controller
        let config_prfl = web::Data::new(config_prfl::ConfigPrfl::init_by_env());

        // Adding various entities.
        // used: profile_registr_controller
        let mailer = web::Data::new(MailerApp::new(config_smtp0));
        // used: profile_registr_controller
        let user_registr_orm = web::Data::new(get_user_registr_orm_app(pool.clone()));
        // used: profile_registr_controller
        let user_recovery_orm = web::Data::new(get_user_recovery_orm_app(pool.clone()));
        // used: profile_auth_controller, profile_registr_controller
        let session_orm = web::Data::new(get_session_orm_app(pool.clone()));
        // used: stream_controller, profile_controller
        let stream_orm = web::Data::new(get_stream_orm_app(pool.clone()));
        // used: profile_controller
        let profile_orm = web::Data::new(get_profile_orm_app(pool.clone()));
        // used: chat_message_controller, chat_ws_controller
        let chat_message_orm = web::Data::new(get_chat_message_orm_app(pool.clone()));

        // Make instance variable of ApiDoc so all worker threads gets the same instance.
        let openapi = swagger_docs::ApiDoc::openapi();

        config
            .app_data(web::Data::clone(&config_app))
            .app_data(web::Data::clone(&config_jwt))
            .app_data(web::Data::clone(&temp_file_config))
            .app_data(web::Data::clone(&config_strm))
            .app_data(web::Data::clone(&config_smtp))
            .app_data(web::Data::clone(&config_prfl))
            .app_data(web::Data::clone(&mailer))
            .app_data(web::Data::clone(&user_registr_orm))
            .app_data(web::Data::clone(&session_orm))
            .app_data(web::Data::clone(&user_recovery_orm))
            .app_data(web::Data::clone(&stream_orm))
            .app_data(web::Data::clone(&profile_orm))
            .app_data(web::Data::clone(&chat_message_orm))
            // Add documentation service "Redoc" and "RapiDoc".
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Add documentation service "SwaggerUi".
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            // Add configuration of internal services.
            .configure(profile_registr_controller::configure())
            .configure(profile_auth_controller::configure())
            .configure(stream_controller::configure())
            .configure(profile_controller::configure())
            .configure(static_controller::configure())
            .configure(chat_message_controller::configure())
            .configure(chat_ws_controller::configure());
    }
}

pub fn create_cors(config_app: settings::config_app::ConfigApp) -> Cors {
    let app_domain = config_app.app_domain;
    // Maximum number of seconds the results can be cached.
    let app_max_age = config_app.app_max_age;

    let mut cors = Cors::default()
        // Add primary domain.
        .allowed_origin(&app_domain)
        // .allowed_origin("https://fonts.googleapis.com")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(app_max_age);

    // Add additional domains.
    let cors_allowed_origin: Vec<&str> = config_app.app_allowed_origin.split(',').collect();
    if cors_allowed_origin.len() > 0 {
        for allowed_origin in cors_allowed_origin.into_iter() {
            let allowed_origin_val = allowed_origin.trim();
            if allowed_origin_val.len() > 0 {
                cors = cors.allowed_origin(allowed_origin_val);
            }
        }
    }
    cors
}

fn app_log(text: &str) {
    if log_enabled!(Info) {
        info!("{}", text);
    } else {
        eprintln!("{}", text);
    }
}

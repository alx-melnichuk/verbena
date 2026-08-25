use std::{env, fs};

use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{
    App, HttpServer, http, middleware,
    web::{Data, ServiceConfig},
};
use dotenv;
use env_logger;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use vrb_authent;
use vrb_chats;
use vrb_common::env_var;
use vrb_db::db;
use vrb_profiles;
use vrb_streams;
#[cfg(not(feature = "mockdata"))]
use vrb_tools::send_email::mailer::impls::MailerApp;
#[cfg(feature = "mockdata")]
use vrb_tools::send_email::mailer::tests::MailerApp;
use vrb_tools::ssl_acceptor;
use vrb_tools::{config_app, send_email::config_smtp};

pub(crate) mod static_controller;
pub mod swagger_docs;

pub async fn server_run() -> std::io::Result<()> {
    #[cfg(feature = "mockdata")]
    #[rustfmt::skip]
    assert!(false, "Launch in \"mockdata\" mode! Disable \"default=[test, mockdata]\" in Cargo.toml.");

    dotenv::dotenv().expect("Failed to read .env file");

    if env::var("RUST_LOG").is_err() {
        env_var::env_set_var("RUST_LOG", "warn,actix_web=info,verbena=info");
    }

    env_logger::init();

    let config_app = config_app::ConfigApp::init_by_env();

    let app_protocol = config_app.app_protocol.clone();
    let app_host = config_app.app_host.clone();
    let app_port = config_app.app_port.clone();
    let app_url = format!("{}:{}", &app_host, &app_port);

    let app_domain = config_app.app_domain.clone();
    eprintln!("Starting server {}", &app_domain);

    eprintln!("Configuring database.");

    let db_url = env::var("DATABASE_URL").expect("Env \"DATABASE_URL\" not found.");
    let max_conn: u32 = env::var("DB_POOL_MAX_CONN").unwrap_or("0".to_owned()).trim().parse().unwrap();
    let min_conn: u32 = env::var("DB_POOL_MIN_CONN").unwrap_or("0".to_owned()).trim().parse().unwrap();
    let max_lifetime_sec: u64 = env::var("DB_POOL_MAX_LIFETIME").unwrap_or("0".to_owned()).trim().parse().unwrap();
    let idle_sec: u64 = env::var("DB_POOL_IDLE_TIMEOUT").unwrap_or("0".to_owned()).trim().parse().unwrap();

    let db_pool = db::init_db_pool(&db_url, max_conn, min_conn, max_lifetime_sec, idle_sec)
        .await
        .expect("Failed to create pool_db");

    eprintln!("db_pool_db.max_conn: {}", max_conn);

    // Execute all unapplied migrations for a given migration source
    let _ = db::run_migration_db(&db_pool).await;

    let config_app2 = config_app.clone();
    #[rustfmt::skip]
    let mut srv = HttpServer::new(move || {
        let cors = create_cors(config_app2.clone());
        let mut app = App::new().wrap(cors).wrap(middleware::Logger::default());
        app = app.app_data(db_pool.clone());

        //   Addendum: ConfigApp, TempFileConfig, ConfigSmtp, MailerApp.
        app = app.configure(configure_data_and_server());

        // Adding data for the controller of "vrb_authent" library.
        //   Addendum: ConfigJwt, UserDb, UserRegistrDb, UserRecoveryDb.
        //   Use: ConfigApp, ConfigSmtp, mailer.
        app = app.configure(vrb_authent::configure_data_and_server(&db_pool));

        // Adding data for the controller of "vrb_profiles" library.
        //   Addendum: ConfigPrfl, ProfileDb.
        //   Use: UserDb, UserRegistrDb.
        app = app.configure(vrb_profiles::configure_data_and_server(&db_pool));

        // Adding data for the controller of "vrb_streams" library.
        //   Addendum: ConfigStrm, StreamDb.
        app = app.configure(vrb_streams::configure_data_and_server(&db_pool));

        // Adding data for the controller of "vrb_chats" library.
        //   Addendum: ConfigJwt, ChatMessageDb.
        //   Use: ConfigJwt, UserDb.
        app = app.configure(vrb_chats::configure_data_and_server(&db_pool));

        app
    });

    if config_app::PROTOCOL_HTTP == app_protocol {
        srv = srv.bind(&app_url)?;
    } else {
        #[rustfmt::skip]
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

pub fn configure_data_and_server() -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        let config_app = config_app::ConfigApp::init_by_env();
        // Creating temporary directory.
        let dir_tmp = config_app.app_dir_tmp.clone();
        fs::create_dir_all(&dir_tmp).expect(&format!("Error when creating the \"{}\" directory:", &dir_tmp));

        // Make instance variable of ApiDoc so all worker threads gets the same instance.
        let openapi = swagger_docs::ApiDoc::openapi();

        // Add documentation service "Redoc" and "RapiDoc".
        config
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Add documentation service "SwaggerUi".
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));

        let config_app1 = config_app.clone();
        let app_dir_tmp = config_app.app_dir_tmp.clone();
        // used: user_recovery_controller, user_registr_controller, static_controller
        let config_app = Data::new(config_app);
        config.app_data(Data::clone(&config_app));

        let temp_file_config = TempFileConfig::default().clone().directory(app_dir_tmp);
        // Used "actix-multipart" to upload files. TempFileConfig.from_req()
        let temp_file_config = Data::new(temp_file_config);
        config.app_data(Data::clone(&temp_file_config));

        let config_smtp0 = config_smtp::ConfigSmtp::init_by_env();
        // used: stream_controller
        let config_smtp = Data::new(config_smtp0.clone());
        config.app_data(Data::clone(&config_smtp));

        // used: user_recovery_controller, user_registr_controller
        let mailer = Data::new(MailerApp::new(config_smtp0));
        config.app_data(Data::clone(&mailer));

        // Add configuration of internal services.
        config.configure(static_controller::configure(config_app1));
    }
}

pub fn create_cors(config_app: config_app::ConfigApp) -> Cors {
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

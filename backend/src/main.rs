use std::{env, io};

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{http, web, App, HttpServer};
use dotenv;
use env_logger;
use log;

mod dbase;
mod email;
mod errors;
mod extractors;
mod schema;
mod sessions;
mod static_controller;
mod users;
mod utils;

use crate::email::config_smtp;
use crate::sessions::config_jwt;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::SessionOrmApp;

#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::{user_auth_controller, user_controller};

// ** Funcion Main **
#[actix_web::main]
async fn main() -> io::Result<()> {
    #[cfg(feature = "mockdata")]
    #[rustfmt::skip]
    assert!(false, "Launch in `mockdata` mode! Disable `default=[test, mockdata]` in Cargo.toml.");

    dotenv::dotenv().expect("Failed to read .env file");

    if std::env::var_os("RUST_LOG").is_none() {
        let log = "info,actix_web=info,actix_server=info,verbena_backend=info";
        std::env::set_var("RUST_LOG", log);
    }
    env_logger::init();

    let app_host = env::var("APP_HOST").expect("APP_HOST not found.");
    let app_port = env::var("APP_PORT").expect("APP_PORT not found.");
    let app_url = format!("{}:{}", &app_host, &app_port);
    let max_age_str = env::var("APP_MAX_AGE").unwrap_or("".to_string()); // default=3600
    let app_max_age: usize = max_age_str.parse::<usize>().unwrap_or(3600);
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");

    // let domain: String = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
    dbase::run_migration(&mut pool.get().unwrap());

    let config_jwt = web::Data::new(config_jwt::ConfigJwt::init_by_env());

    let config_smtp = web::Data::new(config_smtp::ConfigSmtp::init_by_env());

    #[cfg(feature = "mockdata")]
    let user_orm: UserOrmApp = UserOrmApp::new();
    #[cfg(not(feature = "mockdata"))]
    let user_orm: UserOrmApp = UserOrmApp::new(pool.clone());
    let data_user_orm = web::Data::new(user_orm);

    #[cfg(feature = "mockdata")]
    let session_orm: SessionOrmApp = SessionOrmApp::new();
    #[cfg(not(feature = "mockdata"))]
    let session_orm: SessionOrmApp = SessionOrmApp::new(pool.clone());
    let data_session_orm = web::Data::new(session_orm);

    println!("🚀 Server started successfully {}", &app_url);
    log::info!("starting HTTP server at http://{}", &app_url);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4250")
            .allowed_origin("http://127.0.0.1:8080")
            // .allowed_origin("https://fonts.googleapis.com")
            // "HEAD", "CONNECT", "PATCH", "TRACE",
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
            .max_age(app_max_age);
        // let cors = Cors::permissive();
        // let cors = cors.allow_any_method().allow_any_header()

        App::new()
            .app_data(web::Data::clone(&config_jwt))
            .app_data(web::Data::clone(&config_smtp))
            .app_data(web::Data::clone(&data_user_orm))
            .app_data(web::Data::clone(&data_session_orm))
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/assets", "static/assets").show_files_listing())
            .configure(static_controller::configure)
            .service(
                web::scope("/api")
                    .configure(user_auth_controller::configure)
                    .configure(user_controller::configure),
            )
    })
    .bind(&app_url)?
    .run()
    .await
}

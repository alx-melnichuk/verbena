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
mod hash_tools;
mod schema;
mod sessions;
mod static_controller;
mod tools;
mod users;
mod utils;

use crate::email::{config_smtp, mailer};
use crate::sessions::{config_jwt, session_orm::cfg::get_session_orm_app};
use crate::users::{user_auth_controller, user_controller, user_registr_controller};
use crate::users::{
    user_orm::cfg::get_user_orm_app, user_recovery_orm::cfg::get_user_recovery_orm_app,
    user_registr_orm::cfg::get_user_registr_orm_app,
};

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

    let config_app = utils::config_app::ConfigApp::init_by_env();

    let app_host: String = config_app.app_host.clone();
    let app_port: usize = config_app.app_port.clone();
    let app_domain: String = config_app.app_domain.clone();
    // let app_url = format!("{}:{}", &app_host, &app_port);
    let app_max_age: usize = config_app.app_max_age.clone();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
    // let domain: String = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
    dbase::run_migration(&mut pool.get().unwrap());

    let data_config_app = web::Data::new(utils::config_app::ConfigApp::init_by_env());
    let data_config_jwt = web::Data::new(config_jwt::ConfigJwt::init_by_env());
    let config_smtp = config_smtp::ConfigSmtp::init_by_env();
    let data_config_smtp = web::Data::new(config_smtp.clone());
    // data_config_smtp.get_ref().clone()
    let data_mailer = web::Data::new(mailer::cfg::get_mailer_app(config_smtp));
    let data_user_orm = web::Data::new(get_user_orm_app(pool.clone()));
    let data_user_registr_orm = web::Data::new(get_user_registr_orm_app(pool.clone()));
    let data_session_orm = web::Data::new(get_session_orm_app(pool.clone()));
    let data_user_recovery_orm = web::Data::new(get_user_recovery_orm_app(pool.clone()));

    println!("🚀 Server started successfully {}", &app_domain);
    log::info!("starting HTTP server at {}", &app_domain);

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
            .app_data(web::Data::clone(&data_config_app))
            .app_data(web::Data::clone(&data_config_jwt))
            .app_data(web::Data::clone(&data_config_smtp))
            .app_data(web::Data::clone(&data_mailer))
            .app_data(web::Data::clone(&data_user_orm))
            .app_data(web::Data::clone(&data_user_registr_orm))
            .app_data(web::Data::clone(&data_session_orm))
            .app_data(web::Data::clone(&data_user_recovery_orm))
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/assets", "static/assets").show_files_listing())
            .configure(static_controller::configure)
            .service(
                web::scope("/api")
                    .configure(user_registr_controller::configure)
                    .configure(user_auth_controller::configure)
                    .configure(user_controller::configure),
            )
    })
    // .bind(&app_url)?
    .bind(&format!("{}:{}", &app_host, &app_port))?
    .run()
    .await
}

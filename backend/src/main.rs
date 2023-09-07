use std::{env, io};

use actix_files::Files;
use actix_web::web;
use actix_web::{App, HttpServer};
use dotenv;
use env_logger;
use log;

mod dbase;
mod schema;
mod static_controller;
mod users;
mod utils;

use crate::dbase::db;
use crate::users::users_controller;

// ** Funcion Main **
#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().expect("Failed to read .env file");
    env::set_var(
        "RUST_LOG",
        "verbena_backend=debug,actix_web=info,actix_server=info",
    );

    env_logger::init();

    let app_host = env::var("APP_HOST").expect("APP_HOST not found.");
    let app_port = env::var("APP_PORT").expect("APP_PORT not found.");
    let app_url = format!("{}:{}", &app_host, &app_port);
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");

    // let domain: String = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    let pool: db::DbPool = db::init_db_pool(&db_url);
    db::run_migration(&mut pool.get().unwrap());

    println!("🚀 Server started successfully {}", &app_url);
    log::info!("starting HTTP server at http://{}", &app_url);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // enable logger
            .wrap(actix_web::middleware::Logger::default())
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/assets", "static/assets").show_files_listing())
            .configure(static_controller::configure)
            .service(web::scope("/api").configure(users_controller::configure))
    })
    .bind(&app_url)?
    .run()
    .await
}

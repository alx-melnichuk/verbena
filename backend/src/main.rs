use actix_cors::Cors;
use actix_web::{http, App, HttpServer};
use dotenv;
use env_logger;
use log;

use verbena::{configure_server, utils};

// ** Funcion Main **
#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            .configure(configure_server())
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(&format!("{}:{}", &app_host, &app_port))?
    .run()
    .await
}

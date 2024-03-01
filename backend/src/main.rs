use actix_web::{App, HttpServer};
use dotenv;
use env_logger;
use log;

use verbena::{configure_server, create_cors, settings::config_app, streams::config_strm, utils::ssl_acceptor};

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

    // Check the correctness of "STRM_LOGO_VALID_TYPES"
    config_strm::ConfigStrm::init_strm_valid_types_by_env()?;

    let config_app = config_app::ConfigApp::init_by_env();

    let app_host = config_app.app_host.clone();
    let app_protocol = config_app.app_protocol.clone();
    let app_port = config_app.app_port.clone();
    let app_domain = config_app.app_domain.clone();
    let app_url = format!("{}:{}", &app_host, &app_port);
    log::info!("creating temporary directory");
    std::fs::create_dir_all(config_app.app_dir_tmp.clone())?;

    log::info!("creating a directory for upload the logo");
    let config_strm = config_strm::ConfigStrm::init_by_env();
    std::fs::create_dir_all(&config_strm.strm_logo_files_dir)?;

    log::info!("Starting server {}", &app_domain);

    if config_app::PROTOCOL_HTTP == app_protocol {
        HttpServer::new(move || {
            let cors = create_cors(config_app.clone());
            App::new()
                .configure(configure_server())
                .wrap(cors)
                .wrap(actix_web::middleware::Logger::default())
        })
        .bind(&app_url)?
        .run()
        .await
    } else {
        let builder =
            ssl_acceptor::create_ssl_acceptor_builder(&config_app.app_certificate, &config_app.app_private_key);

        HttpServer::new(move || {
            let cors = create_cors(config_app.clone());
            App::new()
                .configure(configure_server())
                .wrap(cors)
                .wrap(actix_web::middleware::Logger::default())
        })
        .bind_openssl(&app_url, builder)?
        .run()
        .await
    }
}

use actix_web::{App, HttpServer};
use dotenv;
use env_logger;
use log;

use verbena::{check_env, check_presence_required_directories, configure_server, create_cors};
use verbena::{profiles::config_prfl, settings::config_app, streams::config_strm, update_env, utils::ssl_acceptor};

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

    // Checking the configuration and encrypting the specified parameters.
    if check_env().unwrap_or(0) > 0 {
        // Load the `.env` file again.
        dotenv::dotenv().expect("Failed to read .env file");
    }
    // Update configurations and decryption of specified parameters.
    update_env().ok();

    env_logger::init();

    // Checking the presence of required directories
    check_presence_required_directories()?;

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

    log::info!("creating a directory for upload the avatar");
    let config_prfl = config_prfl::ConfigPrfl::init_by_env();
    std::fs::create_dir_all(&config_prfl.prfl_avatar_files_dir)?;

    let version = env!("CARGO_PKG_VERSION");
    log::info!("Starting server v.{} {}", version, &app_domain);
    let config_app2 = config_app.clone();

    let mut srv = HttpServer::new(move || {
        let cors = create_cors(config_app2.clone());
        App::new()
            .configure(configure_server())
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
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

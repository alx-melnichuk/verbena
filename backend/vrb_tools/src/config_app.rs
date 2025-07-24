use std::env;

pub const HOST: &str = "127.0.0.1";
pub const PROTOCOL_HTTP: &str = "http";
pub const PROTOCOL_HTTPS: &str = "https";
pub const PORT_HTTP: &str = "80";
pub const PORT_HTTPS: &str = "443";
pub const MAX_AGE: &str = "600";
pub const REGISTR_DURATION: &str = "900";
pub const RECOVERY_DURATION: &str = "600";
pub const NAME: &str = "Verbéna";
pub const CERTIFICATE: &str = "ssl.crt.pem";
pub const PRIVATE_KEY: &str = "ssl.key.pem";
pub const ALLOWED_ORIGIN: &str = "http://localhost:4250,http://127.0.0.1:4250";
pub const DIR_TMP: &str = "./tmp";

#[derive(Debug, Clone)]
pub struct ConfigApp {
    pub app_host: String,
    pub app_protocol: String,
    pub app_port: usize,
    pub app_max_age: usize,
    pub app_domain: String,
    pub app_registr_duration: usize,
    pub app_recovery_duration: usize,
    pub app_name: String,
    pub app_certificate: String,
    pub app_private_key: String,
    pub app_allowed_origin: String,
    pub app_dir_tmp: String,
    pub app_num_workers: Option<usize>,
}

impl ConfigApp {
    pub fn init_by_env() -> Self {
        let app_host = env::var("APP_HOST").unwrap_or(HOST.to_string());
        let protocol = env::var("APP_PROTOCOL").unwrap_or("".to_string()).to_lowercase();
        #[rustfmt::skip]
        let app_protocol = if protocol == PROTOCOL_HTTPS { PROTOCOL_HTTPS.to_string() } else { PROTOCOL_HTTP.to_string() };
        #[rustfmt::skip]
        let port_default = if PROTOCOL_HTTPS == app_protocol { PORT_HTTPS } else { PORT_HTTP };
        let app_port = env::var("APP_PORT").unwrap_or(port_default.to_string());

        // Maximum number of seconds the results can be cached.
        let app_max_age = env::var("APP_MAX_AGE").unwrap_or(MAX_AGE.to_string());

        let app_domain = Self::get_domain(&app_protocol, &app_host, &app_port);
        // Waiting time for registration confirmation (in seconds).
        let app_registr_duration = env::var("APP_REGISTR_DURATION").unwrap_or(REGISTR_DURATION.to_string());
        // Waiting time for password recovery confirmation (in seconds).
        let app_recovery_duration = env::var("APP_RECOVERY_DURATION").unwrap_or(RECOVERY_DURATION.to_string());

        let app_name = env::var("APP_NAME").unwrap_or(NAME.to_string());

        // SSL certificate and private key
        let certificate = if protocol == PROTOCOL_HTTPS { CERTIFICATE } else { "" };
        let app_certificate = env::var("APP_CERTIFICATE").unwrap_or(certificate.to_string());
        // SSL private key
        let private_key = if protocol == PROTOCOL_HTTPS { PRIVATE_KEY } else { "" };
        let app_private_key = env::var("APP_PRIVATE_KEY").unwrap_or(private_key.to_string());

        #[rustfmt::skip]
        let allowed_origin = if app_host == HOST.to_string() { ALLOWED_ORIGIN } else { "" };
        // Cors permissions "allowed_origin" (array of values, comma delimited)
        let app_allowed_origin = env::var("APP_ALLOWED_ORIGIN").unwrap_or(allowed_origin.to_string());
        // Directory for temporary files when uploading user files.
        let app_dir_tmp = env::var("APP_DIR_TMP").unwrap_or(DIR_TMP.to_string());

        // Number of worker services (this is the number of available physical CPU cores for parallel computing).
        // By default, it is detected automatically.
        let num_workers = env::var("APP_NUM_WORKERS").unwrap_or("".to_string());
        #[rustfmt::skip]
        let app_num_workers = if num_workers.len() > 0 { Some(num_workers.parse::<usize>().unwrap()) } else { None };

        ConfigApp {
            app_host,
            app_protocol,
            app_port: app_port.parse::<usize>().unwrap(),
            app_max_age: app_max_age.parse::<usize>().unwrap(),
            app_domain,
            app_registr_duration: app_registr_duration.parse::<usize>().unwrap(),
            app_recovery_duration: app_recovery_duration.parse::<usize>().unwrap(),
            app_name,
            app_certificate,
            app_private_key,
            app_allowed_origin,
            app_dir_tmp,
            app_num_workers,
        }
    }
    fn get_domain(protocol: &str, host: &str, port: &str) -> String {
        format!("{}://{}:{}", protocol, host, port)
    }
}

pub fn get_test_config() -> ConfigApp {
    let app_host = HOST.to_string();
    let app_protocol = PROTOCOL_HTTP.to_string();
    let app_port = 8080;
    let app_domain = ConfigApp::get_domain(&app_protocol, &app_host, &(app_port.to_string())); // "http://127.0.0.1:8080"

    ConfigApp {
        app_host,
        app_port: 8080,
        app_protocol,
        app_max_age: 120,
        app_domain,
        app_registr_duration: 240,
        app_recovery_duration: 120,
        app_name: "app_name".to_string(),
        app_certificate: "demo.crt.pem".to_string(),
        app_private_key: "demo.key.pem".to_string(),
        app_allowed_origin: "".to_string(),
        app_dir_tmp: "./".to_string(),
        app_num_workers: None,
    }
}

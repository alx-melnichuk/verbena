use std::env;

pub const PROTOCOL_HTTP: &str = "http";
pub const PROTOCOL_HTTPS: &str = "https";
const PORT_HTTP: &str = "80";
const PORT_HTTPS: &str = "443";

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
}

impl ConfigApp {
    pub fn init_by_env() -> Self {
        let app_host = env::var("APP_HOST").expect("APP_HOST must be set");
        let protocol = env::var("APP_PROTOCOL").unwrap_or("".to_string());
        let app_protocol = if protocol.to_lowercase() == PROTOCOL_HTTPS {
            PROTOCOL_HTTPS.to_string()
        } else {
            PROTOCOL_HTTP.to_string()
        };
        let port_default = if PROTOCOL_HTTPS == app_protocol {
            PORT_HTTPS
        } else {
            PORT_HTTP
        };
        let app_port = env::var("APP_PORT").unwrap_or(port_default.to_string());
        // Maximum number of seconds the results can be cached.
        let app_max_age = env::var("APP_MAX_AGE").expect("APP_MAX_AGE must be set");
        let app_domain = Self::get_domain(&app_protocol, &app_host, &app_port);
        // Waiting time for registration confirmation (in seconds).
        let app_registr_duration =
            env::var("APP_REGISTR_DURATION").expect("APP_REGISTR_DURATION must be set");
        // Waiting time for password recovery confirmation (in seconds).
        let app_recovery_duration =
            env::var("APP_RECOVERY_DURATION").expect("APP_RECOVERY_DURATION must be set");
        let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
        // SSL certificate and private key
        let app_certificate = env::var("APP_CERTIFICATE").unwrap_or("".to_string());
        // SSL private key
        let app_private_key = env::var("APP_PRIVATE_KEY").unwrap_or("".to_string());
        if protocol.to_lowercase() == PROTOCOL_HTTPS {
            #[rustfmt::skip]
            assert_ne!(0, app_certificate.len(), "For the {} protocol, the value APP_CERTIFICATE must be set.", PROTOCOL_HTTPS);
            #[rustfmt::skip]
            assert_ne!(0, app_private_key.len(), "For the {} protocol, the value APP_PRIVATE_KEY must be set.", PROTOCOL_HTTPS);
        }
        // Cors permissions "allowed_origin" (array of values, comma delimited)
        let app_allowed_origin = env::var("APP_ALLOWED_ORIGIN").unwrap_or("".to_string());

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
        }
    }
    fn get_domain(protocol: &str, host: &str, port: &str) -> String {
        format!("{}://{}:{}", protocol, host, port)
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigApp {
    let app_host = "127.0.0.1".to_string();
    let app_protocol = "http".to_string();
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
    }
}

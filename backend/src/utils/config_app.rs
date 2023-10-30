use std::env;

#[derive(Debug, Clone)]
pub struct ConfigApp {
    pub app_host: String,
    pub app_port: usize,
    pub app_protocol: String,
    pub app_max_age: usize,
    pub app_domain: String,
    pub app_registr_duration: usize,
    pub app_recovery_duration: usize,
}

impl ConfigApp {
    pub fn init_by_env() -> Self {
        let app_host = env::var("APP_HOST").expect("APP_HOST must be set");
        let app_port = env::var("APP_PORT").expect("APP_PORT must be set");
        let app_protocol = env::var("APP_PROTOCOL").expect("APP_PROTOCOL must be set");
        let app_max_age = env::var("APP_MAX_AGE").expect("APP_MAX_AGE must be set");
        let app_domain = format!("{}://{}:{}", &app_protocol, &app_host, &app_port);
        let app_registr_duration =
            env::var("APP_REGISTR_DURATION").expect("APP_REGISTR_DURATION must be set");
        let app_recovery_duration =
            env::var("APP_RECOVERY_DURATION").expect("APP_RECOVERY_DURATION must be set");

        ConfigApp {
            app_host,
            app_port: app_port.parse::<usize>().unwrap(),
            app_protocol,
            app_max_age: app_max_age.parse::<usize>().unwrap(),
            app_domain,
            app_registr_duration: app_registr_duration.parse::<usize>().unwrap(),
            app_recovery_duration: app_recovery_duration.parse::<usize>().unwrap(),
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigApp {
    let app_host = "127.0.0.1".to_string();
    let app_port = 8080;
    let app_protocol = "http".to_string();
    let app_domain = format!("{}://{}:{}", &app_protocol, &app_host, &app_port); // "http://127.0.0.1:8080"

    ConfigApp {
        app_host,
        app_port: 8080,
        app_protocol,
        app_max_age: 60,
        app_domain,
        app_registr_duration: 4,
        app_recovery_duration: 2,
    }
}

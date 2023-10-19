#[derive(Debug, Clone)]
pub struct ConfigApp {
    pub app_host: String,
    pub app_port: i64,
    pub app_max_age: i64,
    pub app_domain: String,
}

impl ConfigApp {
    pub fn init_by_env() -> Self {
        let app_host = std::env::var("APP_HOST").expect("APP_HOST must be set");
        let app_port = std::env::var("APP_PORT").expect("APP_PORT must be set");
        let app_max_age = std::env::var("APP_MAX_AGE").expect("APP_MAX_AGE must be set");
        let app_domain = format!("http://{}:{}", &app_host, &app_port); // "http://127.0.0.1:8080"

        ConfigApp {
            app_host,
            app_port: app_port.parse::<i64>().unwrap(),
            app_max_age: app_max_age.parse::<i64>().unwrap(),
            app_domain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigSmtp {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_sender: String,
}

impl ConfigSmtp {
    pub fn init_by_env() -> Self {
        let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_port = std::env::var("SMTP_PORT").expect("SMTP_PORT must be set");
        let smtp_user = std::env::var("SMTP_USER").expect("SMTP_USER must be set");
        let smtp_pass = std::env::var("SMTP_PASS").expect("SMTP_PASS must be set");
        let smtp_sender = std::env::var("SMTP_SENDER").expect("SMTP_SENDER must be set");

        ConfigSmtp {
            smtp_host,
            smtp_port: smtp_port.parse::<u16>().unwrap(),
            smtp_user,
            smtp_pass,
            smtp_sender,
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
#[allow(dead_code)]
pub fn get_test_config() -> ConfigSmtp {
    ConfigSmtp {
        smtp_host: "127.0.0.1".to_string(),
        smtp_pass: "pass".to_string(),
        smtp_user: "user".to_string(),
        smtp_port: 465,
        smtp_sender: "sender".to_string(),
    }
}

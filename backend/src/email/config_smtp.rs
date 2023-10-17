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
            smtp_pass,
            smtp_user,
            smtp_port: smtp_port.parse::<u16>().unwrap(),
            smtp_sender,
        }
    }
}

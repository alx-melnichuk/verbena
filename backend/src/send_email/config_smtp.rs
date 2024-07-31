use crate::utils::parser::parse_bool;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigSmtp {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_sender: String,
    pub smtp_not_send_letter: bool,
    pub smtp_save_letter: bool,
}

impl ConfigSmtp {
    pub fn init_by_env() -> Self {
        let smtp_host_port = std::env::var("SMTP_HOST_PORT").expect("SMTP_HOST_PORT must be set");
        let (host, port) = smtp_host_port.split_once(':').unwrap_or(("", ""));
        let smtp_host = host.to_string();
        let smtp_port = port.to_string();
        let smtp_user_pass = std::env::var("SMTP_USER_PASS").expect("SMTP_USER_PASS must be set");
        let (user, pass) = smtp_user_pass.split_once(':').unwrap_or(("", ""));
        let smtp_user = user.to_string();
        let smtp_pass = pass.to_string();
        let smtp_sender = smtp_user.clone();
        let not_send_letter = std::env::var("SMTP_NOT_SEND_LETTER").unwrap_or("false".to_string());
        let smtp_not_send_letter = parse_bool(&not_send_letter).unwrap();
        let save_letter = std::env::var("SMTP_SAVE_LETTER").unwrap_or("false".to_string());
        let smtp_save_letter = parse_bool(&save_letter).unwrap();

        ConfigSmtp {
            smtp_host,
            smtp_port: smtp_port.parse::<u16>().unwrap(),
            smtp_user,
            smtp_pass,
            smtp_sender,
            smtp_not_send_letter,
            smtp_save_letter,
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
        smtp_sender: "user".to_string(),
        smtp_not_send_letter: false,
        smtp_save_letter: false,
    }
}

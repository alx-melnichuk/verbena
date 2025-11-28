use std::env;

use vrb_common::{consts, parser};

const NOT_SEND_LETTER: &str = "false";
const SAVE_LETTER: &str = "false";
const PATH_TEMPLATE: &str = "./templates";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigSmtp {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_sender: String,
    pub smtp_path_template: String,
    pub smtp_not_send_letter: bool,
    pub smtp_save_letter: bool,
}

impl ConfigSmtp {
    pub fn init_by_env() -> Self {
        let smtp_host_port = env::var(consts::SMTP_HOST_PORT).expect("Env \"SMTP_HOST_PORT\" not found.");
        let (host, port) = smtp_host_port.split_once(':').unwrap_or(("", ""));
        let smtp_host = host.to_string();
        let smtp_port = port.to_string();
        let smtp_user_pass = env::var(consts::SMTP_USER_PASS).expect("Env \"SMTP_USER_PASS\" not found.");
        let (user, pass) = smtp_user_pass.split_once(':').unwrap_or(("", ""));
        let smtp_user = user.to_string();
        let smtp_pass = pass.to_string();
        let smtp_sender = smtp_user.clone();
        // Path to letter text templates.
        let smtp_path_template = env::var(consts::SMTP_PATH_TEMPLATE).unwrap_or(PATH_TEMPLATE.to_owned());
        let not_send_letter = env::var(consts::SMTP_NOT_SEND_LETTER).unwrap_or(NOT_SEND_LETTER.to_string());
        let smtp_not_send_letter = parser::parse_bool(&not_send_letter).unwrap();
        let save_letter = env::var(consts::SMTP_SAVE_LETTER).unwrap_or(SAVE_LETTER.to_string());
        let smtp_save_letter = parser::parse_bool(&save_letter).unwrap();

        ConfigSmtp {
            smtp_host,
            smtp_port: smtp_port.parse::<u16>().unwrap(),
            smtp_user,
            smtp_pass,
            smtp_sender,
            smtp_path_template,
            smtp_not_send_letter,
            smtp_save_letter,
        }
    }
}

pub fn get_test_config() -> ConfigSmtp {
    // Path to letter text templates.
    let smtp_path_template = env::var(consts::SMTP_PATH_TEMPLATE).unwrap_or(PATH_TEMPLATE.to_owned());
    ConfigSmtp {
        smtp_host: "127.0.0.1".to_string(),
        smtp_pass: "pass".to_string(),
        smtp_user: "user".to_string(),
        smtp_port: 465,
        smtp_sender: "user".to_string(),
        smtp_path_template,
        smtp_not_send_letter: false,
        smtp_save_letter: false,
    }
}

pub trait Mailer {
    /// Send an email to confirm registration.
    fn send_verification_code(
        &self,
        receiver: &str,
        domain: &str,
        nickname: &str,
        target: &str,
    ) -> Result<(), String>;
    /// Send an email to confirm the password change.
    fn send_password_recovery(
        &self,
        receiver: &str,
        domain: &str,
        nickname: &str,
        target: &str,
    ) -> Result<(), String>;
}

pub mod cfg {
    use crate::email::config_smtp::ConfigSmtp;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::MailerApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_mailer_app(config_smtp: ConfigSmtp) -> MailerApp {
        MailerApp::new(config_smtp)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::MailerApp;
    #[cfg(feature = "mockdata")]
    pub fn get_mailer_app(config_smtp: ConfigSmtp) -> MailerApp {
        MailerApp::new(config_smtp)
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {
    use lettre::{
        message::header::ContentType, transport::smtp, Message, SmtpTransport, Transport,
    };
    use std::collections::HashMap;

    use crate::email::config_smtp::ConfigSmtp;
    use crate::tools::template_rendering::render_template;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct MailerApp {
        pub config_smtp: ConfigSmtp,
    }

    impl MailerApp {
        pub fn new(config_smtp: ConfigSmtp) -> Self {
            MailerApp { config_smtp }
        }
        // Create an instance of Credentials.
        pub fn get_credentials(&self) -> smtp::authentication::Credentials {
            smtp::authentication::Credentials::new(
                self.config_smtp.smtp_user.to_owned(),
                self.config_smtp.smtp_pass.to_owned(),
            )
        }
        // Create an instance of SmtpTransport.
        fn new_smtp_transport(&self) -> Result<SmtpTransport, lettre::transport::smtp::Error> {
            let smtp_host = self.config_smtp.smtp_host.to_string();
            let smtp_port = self.config_smtp.smtp_port;

            let transport = SmtpTransport::relay(&smtp_host.to_owned())?
                .port(smtp_port)
                .credentials(self.get_credentials())
                .build();

            Ok(transport)
        }
        // Create a message to send.
        fn new_message(&self, to_whom: &str, subject: &str, body: &str) -> Result<Message, String> {
            if to_whom.len() == 0 {
                return Err("Recipient not specified.".to_string());
            }
            if body.len() == 0 {
                return Err("The contents of the letter are not specified.".to_string());
            }
            let smtp_from = self.config_smtp.smtp_sender.to_string();

            let message = Message::builder()
                .from(smtp_from.parse().unwrap())
                // .reply_to("Yuin <yuin@domain.tld>".parse()?)
                .to(to_whom.parse().unwrap())
                .subject(subject.to_string())
                .header(ContentType::TEXT_HTML)
                .body(body.to_owned())
                .map_err(|e| e.to_string())?;

            Ok(message)
        }
        // Sending mail (synchronous)
        fn sending(&self, message: Message) -> Result<(), String> {
            // Open a remote connection to the SMTP relay server
            let transport = self.new_smtp_transport().map_err(|e| e.to_string())?;
            // Send the email.
            transport.send(&message).map(|_| ()).map_err(|e| e.to_string())?;
            Ok(())
        }
    }

    impl Mailer for MailerApp {
        /// Send an email to confirm registration.
        fn send_verification_code(
            &self,
            receiver: &str,
            domain: &str,
            nickname: &str,
            target: &str,
        ) -> Result<(), String> {
            if receiver.len() == 0 {
                return Err("Recipient not specified.".to_string());
            }
            let subject = "Your account verification code";

            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("subject", subject);
            params.insert("domain", domain);
            params.insert("nickname", nickname);
            params.insert("target", target);
            // Create a html_template to send.
            let html_template = render_template("verification_code", params)?;
            eprintln!("html_template: {:?}", html_template);
            // #- temp
            // let receiver = "lg2aam@gmail.com";
            // // Create a message to send.
            // let message = self.new_message(receiver, subject, &html_template)?;
            // // Sending mail (synchronous)
            // self.sending(message)
            if html_template.len() > 0 {
                Ok(())
            } else {
                Ok(())
            }
        }
        /// Send an email to confirm the password change.
        fn send_password_recovery(
            &self,
            receiver: &str,
            domain: &str,
            nickname: &str,
            target: &str,
        ) -> Result<(), String> {
            if receiver.len() == 0 {
                return Err("Recipient not specified.".to_string());
            }
            let subject = "Your password reset token (valid for only 10 minutes)";

            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("subject", subject);
            params.insert("domain", domain);
            params.insert("nickname", nickname);
            params.insert("target", target);
            // Create a html_template to send.
            let html_template = render_template("password_recovery", params)?;
            eprintln!("html_template: {:?}", html_template);
            // // Create a message to send.
            // let message = self.new_message(receiver, subject, &html_template)?;
            // // Sending mail (synchronous)
            // self.sending(message)
            if html_template.len() > 0 {
                Ok(())
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use crate::email::config_smtp::ConfigSmtp;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct MailerApp {
        pub config_smtp: ConfigSmtp,
    }

    impl MailerApp {
        /// Create a new instance.
        pub fn new(config_smtp: ConfigSmtp) -> Self {
            MailerApp { config_smtp }
        }
    }

    impl Mailer for MailerApp {
        /// Send an email to confirm registration.
        fn send_verification_code(
            &self,
            receiver: &str,
            domain: &str,
            nickname: &str,
            target: &str,
        ) -> Result<(), String> {
            if receiver.len() == 0 {
                return Err("Recipient not specified.".to_string());
            }
            if domain.len() == 0 || nickname.len() == 0 || target.len() == 0 {
                return Err("Recipient params: domain, nickname, target.".to_string());
            }
            /*
            let subject = "Your account verification code";

            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("subject", subject);
            params.insert("domain", domain);
            params.insert("nickname", nickname);
            params.insert("target", target);
            // Create a html_template to send.
            let html_template = render_template("verification_code", params)?;
            // Create a message to send.
            let message = self.new_message(receiver, subject, &html_template)?;
            // Sending mail (synchronous)
            self.sending(message)
            */
            Ok(())
        }
        /// Send an email to confirm the password change.
        fn send_password_recovery(
            &self,
            receiver: &str,
            domain: &str,
            nickname: &str,
            target: &str,
        ) -> Result<(), String> {
            if receiver.len() == 0 {
                return Err("Recipient not specified.".to_string());
            }
            if domain.len() == 0 || nickname.len() == 0 || target.len() == 0 {
                return Err("Recipient params: domain, nickname, target.".to_string());
            }
            /*
            let subject = "Your password reset token (valid for only 10 minutes)";

            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("subject", subject);
            params.insert("domain", domain);
            params.insert("nickname", nickname);
            params.insert("target", target);
            // Create a html_template to send.
            let html_template = render_template("password_recovery", params)?;
            // Create a message to send.
            let message = self.new_message(receiver, subject, &html_template)?;
            // Sending mail (synchronous)
            self.sending(message)
            */
            Ok(())
        }
    }
}

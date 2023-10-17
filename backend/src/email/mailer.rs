use std::collections::HashMap;

use handlebars::Handlebars;
use lettre::{message::header::ContentType, transport::smtp, Message, SmtpTransport, Transport};

use super::config_smtp::ConfigSmtp;

pub fn render_template(template: &str, params: HashMap<&str, &str>) -> Result<String, String> {
    if template.len() == 0 {
        return Err("The template name is not defined.".to_string());
    }
    let template_path = format!("./templates/{}.hbs", template);
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file(template, &template_path)
        .map_err(|e| e.to_string())?;

    handlebars
        .register_template_file("base", "./templates/basic_layout.hbs")
        .map_err(|e| e.to_string())?;

    let mut data = serde_json::json!({});
    for (key, value) in params {
        data[key] = serde_json::Value::String(value.to_string());
    }

    let content_template = handlebars.render(template, &data).map_err(|e| e.to_string())?;

    Ok(content_template)
}

pub struct Mailer {
    config_smtp: ConfigSmtp,
}
// render by template
impl Mailer {
    pub fn new(config_smtp: ConfigSmtp) -> Self {
        Mailer { config_smtp }
    }

    pub fn get_credentials(&self) -> smtp::authentication::Credentials {
        smtp::authentication::Credentials::new(
            self.config_smtp.smtp_user.to_owned(),
            self.config_smtp.smtp_pass.to_owned(),
        )
    }

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
    pub fn new_message(&self, to_whom: &str, subject: &str, body: &str) -> Result<Message, String> {
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
    pub fn sending(&self, message: Message) -> Result<(), String> {
        // Open a remote connection to the SMTP relay server
        let transport = self.new_smtp_transport().map_err(|e| e.to_string())?;
        // Send the email.
        transport.send(&message).map(|_| ()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn send_verification_code(
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
        // Create a message to send.
        let message = self.new_message(receiver, subject, &html_template)?;
        // Sending mail (synchronous)
        self.sending(message)
    }

    pub fn send_password_recovery(
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
        // Create a message to send.
        let message = self.new_message(receiver, subject, &html_template)?;
        // Sending mail (synchronous)
        self.sending(message)
    }
}

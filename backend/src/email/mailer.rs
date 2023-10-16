use handlebars::Handlebars;
use lettre::{
    message::header::ContentType, transport::smtp, AsyncSmtpTransport, AsyncTransport,
    SmtpTransport, Tokio1Executor, Transport,
};

use super::config_smtp::ConfigSmtp;

pub fn render_template(template_name: &str) -> Result<String, handlebars::RenderError> {
    let template_path = format!("./templates/{}.hbs", template_name);
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file(template_name, &template_path)?;
    handlebars.register_template_file("styles", "./templates/partials/styles.hbs")?;
    handlebars.register_template_file("base", "./templates/layouts/base.hbs")?;

    let first_name = "demo first_name";
    let url = "url";

    let data = serde_json::json!({
        "first_name": &first_name,
        "subject": &template_name,
        "url": &url
    });

    let content_template = handlebars.render(template_name, &data)?;

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

    fn new_async_smtp_transport(
        &self,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
        let smtp_host = self.config_smtp.smtp_host.to_string();
        let smtp_port = self.config_smtp.smtp_port;

        let transport =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host.to_owned())?
                .port(smtp_port)
                .credentials(self.get_credentials())
                .build();

        Ok(transport)
    }

    pub fn new_message(
        &self,
        recipient: &str,
        subject: &str,
        letter: &str,
    ) -> Result<lettre::Message, String> {
        if recipient.len() == 0 {
            return Err("Recipient not specified.".to_string());
        }
        if letter.len() == 0 {
            return Err("The contents of the letter are not specified.".to_string());
        }

        let smtp_from = self.config_smtp.smtp_sender.to_string();

        let message = lettre::Message::builder()
            .from(smtp_from.parse().unwrap())
            // .reply_to("Yuin <yuin@domain.tld>".parse()?)
            .to(recipient.parse().unwrap())
            .subject(subject.to_string())
            // .header(ContentType::html())
            .body(letter.to_owned())
            .map_err(|e| e.to_string())?;

        Ok(message)
    }
    // Sending mail (synchronous)
    pub fn sending(&self, message: lettre::Message) -> Result<(), String> {
        // Open a remote connection to the SMTP relay server
        let transport = self.new_smtp_transport().map_err(|e| e.to_string())?;
        // Send the email.
        transport.send(&message).map(|_| ()).map_err(|e| e.to_string())?;
        Ok(())
    }

    // Send mail (asynchronous)
    pub async fn send(&self, message: lettre::Message) -> Result<(), String> {
        // Open a remote connection to the SMTP relay server
        let transport = self.new_async_smtp_transport().map_err(|e| e.to_string())?;
        // Send the email.
        transport.send(message).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn send_verification_code(&self, recipient: &str) -> Result<(), String> {
        if recipient.len() == 0 {
            return Err("Recipient not specified.".to_string());
        }

        let template_name = "verification_code";
        let subject = "Your account verification code";
        let html_template = render_template(template_name).map_err(|e| e.to_string())?;
        if html_template.len() == 0 {
            return Err("The contents of the letter are not specified.".to_string());
        }

        let smtp_from = self.config_smtp.smtp_sender.to_string();

        let message = lettre::Message::builder()
            .from(smtp_from.parse().unwrap())
            // .reply_to("Yuin <yuin@domain.tld>".parse()?)
            .to(recipient.parse().unwrap())
            .subject(subject.to_string())
            .header(ContentType::TEXT_HTML)
            .body(html_template)
            .map_err(|e| e.to_string())?;

        self.sending(message)
    }
}

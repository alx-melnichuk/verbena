use handlebars::Handlebars;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

use lettre::{
    //     message::header::ContentType, transport::smtp::authentication::Credentials,
    AsyncSmtpTransport,
    AsyncTransport,
    // Message,
    Tokio1Executor,
};

use crate::email::config_smtp::{Config, ConfigSmtp};

pub struct SendEmail {
    config_smtp: ConfigSmtp,
}

impl SendEmail {
    pub fn new(config_smtp: ConfigSmtp) -> Self {
        SendEmail { config_smtp }
    }
}

pub struct Email {
    user_name: String,
    user_email: String,
    url: String,
    from: String,
    config: Config,
}

impl Email {
    pub fn new(user_name: String, user_email: String, url: String, config: Config) -> Self {
        let from = format!("Codevo <{}>", config.smtp_from.to_owned());

        Email {
            user_name,
            user_email,
            url,
            from,
            config,
        }
    }

    fn new_transport(
        &self,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
        let creds = Credentials::new(
            self.config.smtp_user.to_owned(),
            self.config.smtp_pass.to_owned(),
        );
        let smtp_host = self.config.smtp_host.to_string();
        let smtp_port = self.config.smtp_port.to_string();
        eprintln!("smtp_host: `{smtp_host}`"); // #-
        eprintln!("smtp_port: `{smtp_port}`"); // #-
        let smtp_user = self.config.smtp_user.to_string();
        let smtp_pass = self.config.smtp_pass.to_string();
        eprintln!("smtp_user: `{smtp_user}`"); // #-
        eprintln!("smtp_pass: `{smtp_pass}`"); // #-
        let smtp_from = self.config.smtp_from.to_string();
        let smtp_to = self.config.smtp_to.to_string();
        eprintln!("smtp_from: `{smtp_from}`"); // #-
        eprintln!("smtp_to: `{smtp_to}`"); // #-

        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
            &self.config.smtp_host.to_owned(),
        )?
        .port(self.config.smtp_port)
        .credentials(creds)
        .build();

        Ok(transport)
    }

    fn render_template(&self, template_name: &str) -> Result<String, handlebars::RenderError> {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file(template_name, &format!("./templates/{}.hbs", template_name))?;
        handlebars.register_template_file("styles", "./templates/partials/styles.hbs")?;
        handlebars.register_template_file("base", "./templates/layouts/base.hbs")?;

        let data = serde_json::json!({
            "first_name": &self.user_name.split_whitespace().next().unwrap(),
            "subject": &template_name,
            "url": &self.url
        });

        let content_template = handlebars.render(template_name, &data)?;

        Ok(content_template)
    }

    async fn send_email(
        &self,
        template_name: &str,
        subject: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let html_template = self.render_template(template_name)?;
        // eprintln!("html_template: `{}`", html_template); // #-

        // let message = Message::builder()
        //  .from("Alice <alice@example.org>".parse().unwrap())
        //  .to("Carla <carla@example.net>".parse().unwrap())
        //  .subject("Hello")
        //  .body("Hi there, it's a test email, with utf-8 chars ë!\n\n\n".to_owned())
        //  .unwrap();
        // pub struct Message { headers: Headers, body: MessageBody, envelope: Envelope }

        // let address_to = format!("{} <{}>", self.user_name.as_str(), self.user_email.as_str())
        //     .parse()
        //     .unwrap();
        // eprintln!("address_to: `{address_to}`"); // #-
        // let address_from = self.from.as_str().parse().unwrap();
        // eprintln!("address_from: `{address_from}`"); // #-

        let smtp_from = self.config.smtp_from.as_str().parse().unwrap();
        eprintln!("smtp-from: `{smtp_from}`"); // #-
        let smtp_to = self.config.smtp_to.as_str().parse().unwrap();
        eprintln!("smtp-to: `{smtp_to}`"); // #-

        let message = Message::builder()
            .to(smtp_to)
            // .reply_to(self.from.as_str().parse().unwrap())
            .from(smtp_from)
            .subject(subject)
            // .header(ContentType::TEXT_HTML)
            // .body(html_template)?;
            // .header(ContentType::TEXT_PLAIN)
            .body("Hi there, it's a test email, with utf-8 chars ë!\n\n\n".to_owned())?;

        let transport = self.new_transport()?;

        transport.send(message).await?;
        Ok(())
    }

    pub async fn send_verification_code(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_email("verification_code", "Your account verification code").await
    }

    pub async fn send_password_reset_token(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_email(
            "reset_password",
            "Your password reset token (valid for only 10 minutes)",
        )
        .await
    }

    /*pub fn send(&self) -> Result<(), String> {
        let smtp_to = self.config.smtp_to.to_string();
        let smtp_from = self.config.smtp_from.to_string();
        // let recipient:String,
        let subject = "Demo test";
        let body = "Hi there, it's a test email, with utf-8 chars ë!\n\n\n";

        let email = Message::builder()
            .from(smtp_from.parse().unwrap())
            // .reply_to("Yuin <yuin@domain.tld>".parse()?)
            .to(smtp_to.parse().unwrap())
            .subject(subject.to_string())
            // .header(ContentType::html())
            .body(body.to_owned())
            .unwrap();

        let credentials = Credentials::new(
            self.config.smtp_user.to_owned(),
            self.config.smtp_pass.to_owned(),
        );
        let smtp_host = self.config.smtp_host.to_string();
        // Open a remote connection to the SMTP relay server
        let mailer = SmtpTransport::relay(&smtp_host.to_string())
            .unwrap()
            .credentials(credentials)
            .build();
        // Send the email
        match mailer.send(&email) {
            Ok(_) => {
                println!("Email sent successfully!");
                Ok(())
            }
            Err(e) => {
                eprintln!("mailer.send() {:#?}", e);
                Err(e.to_string())
            }
        }
    }*/
}

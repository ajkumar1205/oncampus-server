use actix_web::web;
use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParametersBuilder},
    },
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

#[derive(Debug, Clone)]
pub struct Email {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    email: String,
}

impl Email {
    pub fn init(username: String, password: String) -> Result<Email, Box<dyn std::error::Error>> {
        let cred = Credentials::new(username.clone(), password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")?
            .credentials(cred)
            .build();

        Ok(Self {
            mailer,
            email: username,
        })
    }

    pub async fn send(&self, to: String, otp: String) -> Result<(), Box<dyn std::error::Error>> {
        let email = Message::builder()
            .from(Mailbox::new(
                Some("OnCampus".to_owned()),
                Address::new("oncampus.chat", "gmail.com")?,
            ))
            .to(to.parse()?)
            .subject("OnCampus Email Verification")
            .body(format!("The otp for OnCampus is {}", otp))?;

        self.mailer.send(email).await?;

        Ok(())
    }

    pub fn generate_otp() -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect()
    }
}

use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

#[derive(Debug, Clone)]
pub struct Email {
    mailer: SmtpTransport,
    email: String,
}

impl Email {
    pub fn init(username: String, password: String) -> Result<Email, Box<dyn std::error::Error>> {
        let cred = Credentials::new(username.clone(), password);
        let mailer = SmtpTransport::relay("smtp.gmail.com")?
            .credentials(cred)
            .build();

        Ok(Self {
            mailer,
            email: username,
        })
    }

    pub fn send(&self, to: String, otp: String) -> Result<(), Box<dyn std::error::Error>> {
        let email = Message::builder()
            .from(format!("OnCampus {}", &self.email).parse()?)
            .to(to.parse()?)
            .subject("OnCampus Email Verification")
            .body(format!("The otp for OnCampus is {}", otp))?;

        self.mailer.send(&email)?;

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

use crate::models::user::User;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, fs};

pub struct JWT {
    private: EncodingKey,
    public: DecodingKey,
}

impl JWT {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let private = EncodingKey::from_rsa_pem(fs::read_to_string("private.pem")?.as_bytes())?;
        let public = DecodingKey::from_rsa_pem(fs::read_to_string("public.pem")?.as_bytes())?;

        Ok(Self { private, public })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub token: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(user: &User, exp: usize) -> Self {
        Self {
            sub: user.email.clone(),
            token: user.password.clone(),
            exp,
        }
    }

    pub fn encode(&self, jwt: &JWT) -> Result<String, Box<dyn std::error::Error>> {
        Ok(encode(&Header::new(Algorithm::RS256), self, &jwt.private)?)
    }

    pub fn decode(token: &str, jwt: &JWT) -> Result<Self, Box<dyn std::error::Error>> {
        let data = decode::<Self>(token, &jwt.public, &Validation::new(Algorithm::RS256))?;

        Ok(data.claims)
    }
}

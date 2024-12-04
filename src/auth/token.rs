use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use libsql::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub token: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(uid: String) -> Self {
        Self {
            sub: uid,
            token: "refresh".to_string(),
            exp: 0,
        }
    }

    pub fn get_access(&mut self, jwt: &JWT) -> Result<String, Box<dyn std::error::Error>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .unwrap()
            .timestamp() as usize;
        self.exp = expiration;
        self.token = "access".to_string();

        Ok(encode(&Header::new(Algorithm::RS256), self, &jwt.private)?)
    }

    pub fn get_refresh(&mut self, jwt: &JWT) -> Result<String, Box<dyn std::error::Error>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .unwrap()
            .timestamp() as usize;
        self.exp = expiration;
        self.token = "refresh".to_string();

        Ok(encode(&Header::new(Algorithm::RS256), self, &jwt.private)?)
    }

    pub fn decode(token: &str, jwt: &JWT) -> Result<Self, Box<dyn std::error::Error>> {
        let data = decode::<Self>(token, &jwt.public, &Validation::new(Algorithm::RS256))?;
        Ok(data.claims)
    }

    pub fn is_expired(&self) -> bool {
        self.exp < chrono::Utc::now().timestamp() as usize
    }

    pub async fn blacklist(
        token: &str,
        conn: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute("INSERT INTO tokens (token) VALUES (?1)", params![token])
            .await?;
        Ok(())
    }

    pub async fn is_valid(
        token: &str,
        conn: &Connection,
        jwt: &JWT,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut row = conn
            .query("SELECT * FROM tokens WHERE token = ?1", params![token])
            .await?;
        if row.next().await.unwrap().is_some() {
            return Ok(false);
        }

        let claim = Claims::decode(token, jwt)?;

        return Ok(!claim.is_expired());
    }
}

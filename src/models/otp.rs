use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

use libsql::{Row, de};
use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Otp {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 6, max = 6, message = "OTP must be 6 characters long"))]
    pub otp: String,
    pub created_at: NaiveDateTime,
}

impl TryFrom<Row> for Otp {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        // Extract values from the row
        let email: String = row.get(0)?;
        let otp: String = row.get(1)?;
        
        // Convert timestamp to NaiveDateTime
        let created_at_str: String = row.get(2)?;
        log::info!("created_at_str: {}", created_at_str);
        let created_at = NaiveDateTime::parse_from_str(
            &created_at_str, 
            "%Y-%m-%d %H:%M:%S"
        )?;

        Ok(Otp {
            email,
            otp,
            created_at,
        })
    }
}

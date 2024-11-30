use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator_derive::Validate;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Otp {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 6, max = 6, message = "OTP must be 6 characters long"))]
    pub otp: String,
    pub created_at: NaiveDateTime,
}
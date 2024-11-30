use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Debug, Serialize, Validate, Deserialize)]
pub struct User {
    pub id: Uuid,
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = 4, message = "Username must be at least 4 characters long"))]
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub roll: String,
    pub posts: i32,
    pub followers: i32,
    pub following: i32,
    pub dob: NaiveDate,
    pub is_active: bool,
    pub is_superuser: bool,
    #[validate(url(message = "Invalid URL"))]
    pub profile_url: Option<String>,
}

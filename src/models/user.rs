use std::sync::Arc;

use chrono::NaiveDate;
use libsql::{params, Connection, Row};
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InsertUser {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = 4, message = "Username must be at least 4 characters long"))]
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub roll: String,
    pub dob: NaiveDate,
}

impl InsertUser {
    pub async fn insert_into_db(
        &self,
        uuid: Uuid,
        conn: Arc<Connection>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let email = self.email.clone();
        let password = bcrypt::hash(self.password.clone(), bcrypt::DEFAULT_COST)?;
        let username = self.username.clone();
        let first_name = self.first_name.clone();
        let last_name = self.last_name.clone();
        let roll = self.roll.clone();
        let dob = self.dob.clone();

        conn.execute(
            r#"
            INSERT INTO users (
                id, email, password, username, first_name, last_name, roll, dob
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
            )
            "#,
            params!(
                uuid.to_string(),
                email,
                password,
                username,
                first_name,
                last_name,
                roll,
                dob.to_string()
            ),
        )
        .await?;

        Ok(())
    }
}

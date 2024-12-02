use actix_web::{
    error,
    web::{Data, Json},
    HttpResponse,
};
use chrono::Duration;
use futures::{FutureExt, TryFutureExt};
use libsql::{params, Connection};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Timestamp;

use validator::Validate;
use validator_derive::Validate;

use crate::{email, models::otp::Otp};
use crate::{email::Email, models::user::InsertUser};

mod token;

// ======================================== REGISTER USER FOR FURTHER VERIFICATION ==========================================

#[actix_web::post("/auth/register")]
pub async fn register_user(
    user: Json<InsertUser>,
    conn: Data<Connection>,
) -> Result<HttpResponse, actix_web::error::Error> {
    user.validate().map_err(|validation_errors| {
        info!("User validation failed: {:?}", validation_errors);
        error::ErrorBadRequest(serde_json::to_string(&validation_errors).unwrap_or_default())
    })?;

    if user.username.contains(" ") || user.username.contains("@") || user.username.contains("/") {
        return Ok(HttpResponse::BadRequest().body("Username cannot contain spaces or @"));
    }

    if !user.email.ends_with("dcrustm.org") {
        return Ok(HttpResponse::BadRequest().body("Only DCRUSTM email addresses are allowed"));
    }

    info!("User data is correct: {:?}", user);
    let user = user.into_inner();
    let conn = conn.into_inner();

    let context = uuid::Context::new(rand::random());
    let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    let uuid = uuid::Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);

    let mut row = conn
        .query(
            "SELECT * FROM users WHERE email = ?1",
            params![user.email.clone()],
        )
        .await
        .map_err(|e| {
            error!("Error checking if user exists: {:?}", e);
            error::ErrorInternalServerError("Failed to register user. Please try again.")
        })?;

    if let Ok(Some(_)) = row.next().await {
        return Ok(HttpResponse::BadRequest().body("User with this email already exists"));
    }

    user.insert_into_db(uuid, conn).await.map_err(|e| {
        error!("Error inserting user into database: {:?}", e);
        error::ErrorInternalServerError("Failed to register user. Please try again.")
    })?;

    Ok(HttpResponse::Created().json(json!(user)))
}

// =====================================  SENDS OTP FOR EMAIL VERIFICATION =================================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EmailVerificationForm {
    #[validate(email(message = "Invalid email address"))]
    email: String,
}

#[actix_web::post("/auth/send-otp")]
pub async fn send_otp(
    conn: Data<Connection>,
    email: Json<EmailVerificationForm>,
    mailer: Data<Email>,
) -> Result<HttpResponse, actix_web::Error> {
    email.validate().map_err(|validation_errors| {
        info!("Email validation failed: {:?}", validation_errors);
        error::ErrorBadRequest(serde_json::to_string(&validation_errors).unwrap_or_default())
    })?;

    let email = email.into_inner();

    if !email.email.ends_with("dcrustm.org") {
        return Ok(HttpResponse::BadRequest().body("Only DCRUSTM email addresses are allowed"));
    }

    info!("Email data is correct");

    let otp = Email::generate_otp();
    info!("OTP generated {otp}");

    // Clone necessary data for background task
    let mailer_clone = mailer.clone();

    conn.execute(
        r#"INSERT INTO otps (email, otp, created_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
            ON CONFLICT(email) DO UPDATE SET otp = ?2, created_at = CURRENT_TIMESTAMP"#,
        params![email.email.clone(), otp.clone()],
    )
    .await
    .map_err(|e| {
        error!("Error inserting OTP into database: {:?}", e);
        error::ErrorInternalServerError("Failed to send OTP. Please try again.")
    })?;

    let _ = actix_web::rt::spawn(async move {
        let _ = mailer_clone.send(email.email, otp.clone()).await;
    });

    Ok(HttpResponse::Ok().body("Otp sending initiated"))
}

// ======================================== VERIFY OTP FOR EMAIL VERIFICATION ==========================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OTPVerificationForm {
    #[validate(length(min = 6, max = 6, message = "OTP must be 6 characters long"))]
    otp: String,
    #[validate(email(message = "Invalid email address"))]
    email: String,
}

#[actix_web::post("/auth/verify-otp")]
pub async fn verify_otp(
    conn: Data<Connection>,
    form: Json<OTPVerificationForm>,
) -> Result<HttpResponse, actix_web::Error> {
    if let Err(e) = form.validate() {
        return Ok(HttpResponse::BadRequest().json(e));
    }

    let form = form.into_inner();

    let val = conn
        .query(
            "SELECT * FROM otps WHERE email = ?1",
            params![form.email.clone()],
        )
        .await
        .map_err(|_| error::ErrorBadGateway("Something went wrong"))?
        .next()
        .await
        .map_err(|_| error::ErrorInternalServerError("Something went wrong"))?
        .unwrap();

    log::info!("Row fetched successfully {:?}", val);
    let otp = Otp::try_from(val).map_err(|e| {
        error!("Error converting row to Otp: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;

    if otp.otp == form.otp
        && otp.email == form.email
        && otp.created_at.and_utc() + Duration::minutes(5) > chrono::Utc::now()
    {
        return Ok(HttpResponse::Ok().body("Your email has been verified"));
    }

    Ok(HttpResponse::BadRequest().body("Invalid OTP"))
}

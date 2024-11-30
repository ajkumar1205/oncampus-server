use actix_web::{
    error,
    web::{Data, Json},
    HttpResponse, Responder,
};
use chrono::{Duration, Timelike};
use libsql::{de, Row};
use libsql::{params, Connection};
use serde::{Deserialize, Serialize};
use log::info;

use validator::Validate;
use validator_derive::Validate;

use crate::email::Email;
use crate::models::otp::Otp;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EmailVerificationForm {
    #[validate(email(message = "Invalid email address"))]
    email: String,
}

#[actix_web::post("/auth/send-otp/")]
pub async fn send_otp(
    conn: Data<Connection>,
    email: Json<EmailVerificationForm>,
    mailer: Data<Email>,
) -> impl Responder {
    if let Err(e) = email.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    info!("Email data is correct");

    let otp = Email::generate_otp();
    info!("OTP generated");
    let email = email.into_inner();
    let _ = conn
        .execute(
            "INSERT INTO otps (email, otp) VALUES ($1, $2)",
            params![email.email.clone(), otp.clone()],
        )
        .await
        .map_err(|_| error::ErrorInternalServerError("Something Went Wrong"));

    let _ = mailer
        .send(email.email, otp)
        .map_err(|_| error::ErrorBadGateway("Unable to Send email"));

    HttpResponse::Ok().body("Otp sent successfully")
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OTPVerificationForm {
    #[validate(length(min = 6, max = 6, message = "OTP must be 6 characters long"))]
    otp: String,
    #[validate(email(message = "Invalid email address"))]
    email: String,
}

#[actix_web::post("/auth/verify-otp")]
pub async fn verify_otp(conn: Data<Connection>, form: Json<OTPVerificationForm>) -> impl Responder {
    if let Err(e) = form.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let form = form.into_inner();

    let val = conn
        .query(
            "SELECT * FROM otps WHERE email = $1 LIMIT 1",
            params![form.email.clone()],
        )
        .await
        .map_err(|_| error::ErrorBadGateway("Something went wrong"))
        .unwrap()
        .next()
        .await
        .map_err(|_| error::ErrorInternalServerError("Something went wrong"))
        .unwrap()
        .unwrap();

    let otp = de::from_row::<Otp>(&val).unwrap();

    if otp.otp == form.otp
        && otp.email == form.email
        && otp.created_at.and_utc() + Duration::seconds(5 * 60) < chrono::Local::now()
    {
        return HttpResponse::Ok().body("Your email has been verified");
    }

    HttpResponse::BadRequest().body("Invalid OTP")
}

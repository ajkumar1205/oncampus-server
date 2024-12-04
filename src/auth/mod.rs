use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse,
};
use chrono::Duration;
use libsql::{params, Connection};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use token::{Claims, JWT};
use uuid::Timestamp;

use validator::Validate;
use validator_derive::Validate;

use crate::models::{otp::Otp, user::User};
use crate::{email::Email, models::user::InsertUser};

pub mod token;

// ======================================== REGISTER USER FOR FURTHER VERIFICATION ==========================================

#[actix_web::post("/register")]
pub async fn register_user(
    user: Json<InsertUser>,
    conn: Data<Connection>,
) -> Result<HttpResponse, actix_web::error::Error> {
    user.validate().map_err(|validation_errors| {
        info!("User validation failed: {:?}", validation_errors);
        error::ErrorBadRequest(serde_json::to_string(&validation_errors).unwrap_or_default())
    })?;

    if user.username.contains(" ") || user.username.contains("@") || user.username.contains("/") {
        info!("Username cannot contain spaces or @");
        return Ok(HttpResponse::BadRequest().body("Username cannot contain spaces or @"));
    }

    if !user.email.ends_with("dcrustm.org") {
        info!("Only DCRUSTM email addresses are allowed");
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
        info!("User with this email already exists");
        return Ok(HttpResponse::BadRequest().body("User with this email already exists"));
    }

    user.insert_into_db(uuid, conn).await.map_err(|e| {
        error!("Error inserting user into database: {:?}", e);
        error::ErrorInternalServerError("Failed to register user. Please try again.")
    })?;

    Ok(HttpResponse::Created().json(json!({
        "user": {
            "id": uuid,
            "email": user.email,
            "username": user.username,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "roll": user.roll,
            "dob": user.dob
        }
    })))
}

// =====================================  SENDS OTP FOR EMAIL VERIFICATION =================================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EmailVerificationForm {
    #[validate(email(message = "Invalid email address"))]
    email: String,
}

#[actix_web::post("/send-otp")]
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

#[actix_web::post("/verify-otp")]
pub async fn verify_otp(
    conn: Data<Connection>,
    jwt: Data<token::JWT>,
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
        // WRITE UPDATE QUERY TO MARK USER is_active = true
        conn.execute(
            "UPDATE users SET is_active = true WHERE email = ?1",
            params![form.email.clone()],
        )
        .await
        .map_err(|e| {
            error!("Error updating user: {:?}", e);
            error::ErrorInternalServerError("Something went wrong")
        })?;

        let user_id = conn
            .query("SELECT id FROM users WHERE email = ?1", params!(form.email))
            .await
            .map_err(|e| {
                error!("Error fetching user: {:?}", e);
                error::ErrorInternalServerError("Something went wrong")
            })?
            .next()
            .await
            .map_err(|e| {
                error!("Error fetching user: {:?}", e);
                error::ErrorInternalServerError("Something went wrong")
            })?
            .unwrap()
            .get::<String>(0)
            .unwrap();

        let mut claim = Claims::new(user_id);

        let access_token = claim.get_access(&jwt).map_err(|e| {
            error!("Error generating access token: {:?}", e);
            error::ErrorInternalServerError("Something went wrong")
        })?;
        let refresh_token = claim.get_refresh(&jwt).map_err(|e| {
            error!("Error generating refresh token: {:?}", e);
            error::ErrorInternalServerError("Something went wrong")
        })?;

        return Ok(HttpResponse::Ok().json(json!({
            "tokens" : {
                "access_token": access_token,
                "refresh_token": refresh_token
            }
        })));
    }

    Ok(HttpResponse::BadRequest().body("Invalid OTP"))
}

// ================================================== REFRESH THE EXPIRED ACCESS TOKEN ========================================================

#[derive(Debug, Serialize, Deserialize)]
struct RefreshToken {
    token: String,
}

#[actix_web::post("/refresh")]
pub async fn refresh_tokens(
    token: Json<RefreshToken>,
    conn: Data<Connection>,
    jwt: Data<token::JWT>,
) -> Result<HttpResponse, actix_web::Error> {
    let refresh = token.into_inner().token;
    let jwt = jwt.into_inner();
    let conn = conn.into_inner();
    let rclaim: Claims;
    if let Ok(r) = Claims::decode(&refresh, &jwt) {
        rclaim = r;
    } else {
        return Ok(HttpResponse::Unauthorized().body("Invalid Token in body"));
    }

    if rclaim.token != "refresh" {
        return Ok(HttpResponse::Unauthorized().body("Use Refresh token"));
    }

    if let Ok(b) = Claims::is_valid(&refresh, &conn, &jwt).await {
        if b {
            return Ok(HttpResponse::Unauthorized().body("Token is blacklisted"));
        }
    } else {
        return Err(error::ErrorInternalServerError("Something went wrong"));
    }

    Claims::blacklist(&refresh, &conn).await.map_err(|e| {
        error!("Error blacklisting token: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;

    let mut tokens = Claims::new(rclaim.sub.clone());

    let access_token = tokens.get_access(&jwt).map_err(|e| {
        error!("Error generating access token: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;
    let refresh_token = tokens.get_refresh(&jwt).map_err(|e| {
        error!("Error generating refresh token: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "tokens" : {
            "access_token": access_token,
            "refresh_token": refresh_token
        }
    })))
}

// ======================================== LOGIN ENDPOINT ============================================
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Credentials {
    #[validate(length(min = 4, message = "Username must be at least 4 characters long"))]
    user: String,
    password: String,
}

#[actix_web::post("/login")]
pub async fn login(
    cred: Json<Credentials>,
    conn: Data<Connection>,
    jwt: Data<JWT>,
) -> Result<HttpResponse, actix_web::Error> {
    cred.validate().map_err(|validation_errors| {
        info!("User validation failed: {:?}", validation_errors);
        error::ErrorBadRequest(serde_json::to_string(&validation_errors).unwrap_or_default())
    })?;

    let cred = cred.into_inner();
    let conn = conn.into_inner();

    let mut row = conn
        .query(
            "SELECT id, password FROM users WHERE username = ?1",
            params![cred.user.clone()],
        )
        .await
        .map_err(|e| {
            error!("Error checking if user exists: {:?}", e);
            error::ErrorInternalServerError("Failed to login user. Please try again.")
        })?;

    let user = match row.next().await {
        Ok(Some(user)) => user,
        _ => return Ok(HttpResponse::BadRequest().body("User not found")),
    };

    let password = user.get::<String>(1).unwrap();
    if bcrypt::verify(cred.password, &password).unwrap() {
        let user_id = user.get::<String>(0).unwrap();
        let mut claim = Claims::new(user_id);

        let access_token = claim.get_access(&jwt).map_err(|e| {
            error!("Error generating access token: {:?}", e);
            error::ErrorInternalServerError("Something went wrong")
        })?;
        let refresh_token = claim.get_refresh(&jwt).map_err(|e| {
            error!("Error generating refresh token: {:?}", e);
            error::ErrorInternalServerError("Something went wrong")
        })?;

        return Ok(HttpResponse::Ok().json(json!({
            "tokens" : {
                "access_token": access_token,
                "refresh_token": refresh_token
            }
        })));
    }

    Ok(HttpResponse::BadRequest().body("Invalid credentials"))
}

// ======================================== LOGOUT ENDPOINT ============================================

#[actix_web::post("/logout")]
async fn logout(
    req: HttpRequest,
    refresh: Json<RefreshToken>,
    conn: Data<Connection>,
    jwt: Data<JWT>,
) -> Result<HttpResponse, actix_web::Error> {
    let refresh = refresh.into_inner().token;
    let jwt = jwt.into_inner();
    let conn = conn.into_inner();

    let access: Arc<String>;
    if let Some(t) = req.extensions().get::<Arc<String>>() {
        access = t.clone();
    } else {
        return Ok(HttpResponse::Unauthorized().body("Token not found"));
    }

    let rclaim: Claims;
    if let Ok(r) = Claims::decode(&refresh, &jwt) {
        rclaim = r;
    } else {
        return Ok(HttpResponse::Unauthorized().body("Invalid Token in body"));
    }

    if rclaim.token != "refresh" {
        return Ok(HttpResponse::Unauthorized().body("Use Refresh token"));
    }

    if let Ok(b) = Claims::is_valid(&refresh, &conn, &jwt).await {
        if !b {
            return Ok(HttpResponse::Unauthorized().body("Token is blacklisted"));
        }
    } else {
        return Err(error::ErrorInternalServerError("Something went wrong"));
    }

    Claims::blacklist(&refresh, &conn).await.map_err(|e| {
        error!("Error blacklisting token: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;

    Claims::blacklist(&access, &conn).await.map_err(|e| {
        error!("Error blacklisting token: {:?}", e);
        error::ErrorInternalServerError("Something went wrong")
    })?;

    Ok(HttpResponse::Ok().body("Logged out successfully"))
}

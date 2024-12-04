use actix_web::middleware::{from_fn, Logger};
use actix_web::web::{Data, Json};
use actix_web::{error, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use auth::token::{Claims, JWT};
use auth::{login, logout, refresh_tokens, register_user, send_otp, verify_otp};
use libsql::{params, Connection};
use simplelog::*;
use std::rc::Rc;
use std::{env, fs::File, sync::Arc};

mod auth;
mod db;
mod email;
mod middleware;
mod models;
mod posts;
mod aws;

use db::Db;
use email::Email;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    // std::env::set_var("RUST_LOG", "debug");
    let log_file = File::create("server.log")?;
    let trace_log_file = File::create("server.trace.log")?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info, // Log all levels
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
        WriteLogger::new(LevelFilter::Debug, Config::default(), trace_log_file),
    ])?;

    let url = env::var("DB_DCRUST_URL")?;
    let token = env::var("DB_DCRUST_TOKEN")?;

    let email = env::var("EMAIL")?;
    let email_pass = env::var("EMAIL_APP_PASSWORD")?;

    let db = Db::init(url, token).await?;
    // db.drop_db().await?;
    db.create_db().await?;
    let conn_data = web::Data::new(db.get_conn().clone());

    let mail_data = web::Data::new(Email::init(email, email_pass)?);

    let jwt = web::Data::new(JWT::init()?);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(conn_data.clone())
            .app_data(mail_data.clone())
            .app_data(jwt.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    err.to_string(),
                    HttpResponse::BadRequest().body(err.to_string()),
                )
                .into()
            }))
            .service(
                web::scope("/auth")
                    .service(register_user)
                    .service(verify_otp)
                    .service(send_otp)
                    .service(refresh_tokens)
                    .service(login)
            )
            .service(
                web::scope("/api")
                    .wrap(from_fn(middleware::jwt))
                    .service(home)
                    .service(logout)
            )
            .default_service(web::route().to(|| async { actix_web::HttpResponse::NotFound() }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

#[actix_web::get("/")]
async fn home(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let claims: Arc<Claims>;
    if let Some(r)  = req.extensions().get::<Arc<Claims>>() {
        claims = r.clone();
    } else {
        return Ok(HttpResponse::Unauthorized().body("Token not found"));
    }

    Ok(HttpResponse::Ok().body(format!("Welcome {}", claims.sub)))
}

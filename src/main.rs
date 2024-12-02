use actix_web::middleware::Logger;
use actix_web::{error, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use auth::{register_user, send_otp, verify_otp};
use libsql::{params, Connection};
use simplelog::*;
use std::{env, fs::File, sync::Arc};

mod auth;
mod db;
mod email;
mod models;

use db::Db;
use email::Email;

#[actix_web::get("/")]
async fn index(conn: web::Data<Arc<Connection>>) -> impl Responder {
    HttpResponse::Ok().body("This is home page")
}

#[actix_web::get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

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

    let db = Db::init_turso(url, token).await?;
    let conn_data = web::Data::new(db.get_conn().clone());

    let mail_data = web::Data::new(Email::init(email, email_pass)?);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(conn_data.clone())
            .app_data(mail_data.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    err.to_string(),
                    HttpResponse::BadRequest().body(err.to_string()),
                )
                .into()
            }))
            .service(register_user)
            .service(verify_otp)
            .service(send_otp)
            .default_service(web::route().to(|| async { actix_web::HttpResponse::NotFound() }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

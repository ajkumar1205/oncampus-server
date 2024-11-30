use actix_web::middleware::Logger;
use actix_web::{error, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use auth::{send_otp, verify_otp};
use libsql::{params, Connection};
use std::{env, sync::Arc};

mod auth;
mod db;
mod email;
mod models;

use db::Db;
use email::Email;

#[actix_web::get("/")]
async fn index(conn: web::Data<Connection>) -> impl Responder {
    HttpResponse::Ok().body("This is home page")
}

#[actix_web::get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let url = env::var("DB_DCRUST_URL")?;
    let token = env::var("DB_DCRUST_TOKEN")?;

    let email = env::var("EMAIL")?;
    let email_pass = env::var("EMAIL_APP_PASSWORD")?;

    let db = Db::init_turso(url, token).await?;
    let conn = db.get_conn();

    let mail = Email::init(email, email_pass)?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(conn.clone()))
            .app_data(web::Data::new(mail.clone()))
            .service(verify_otp)
            .service(send_otp)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

use actix_web::middleware::{from_fn, Logger};
use actix_web::web::{Data, Json, ServiceConfig};
use actix_web::{error, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use auth::token::{Claims, JWT};
use auth::{login, logout, refresh_tokens, register_user, send_otp, verify_otp};
use libsql::{params, Connection};
use posts::{create, upload_url};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use simplelog::*;
use std::rc::Rc;
use std::{env, fs::File, sync::Arc};

mod auth;
mod aws;
mod db;
mod email;
mod middleware;
mod models;
mod posts;

use db::Db;
use email::Email;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenvy::dotenv().ok();
    // std::env::set_var("RUST_LOG", "debug");
    let log_file = File::open("server.log")?;
    let trace_log_file = File::open("server.trace.log")?;

    // CombinedLogger::init(vec![
    //     TermLogger::new(
    //         LevelFilter::Info, // Log all levels
    //         Config::default(),
    //         TerminalMode::Mixed,
    //         ColorChoice::Auto,
    //     ),
    //     WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
    //     WriteLogger::new(LevelFilter::Debug, Config::default(), trace_log_file),
    // ])
    // .expect("Failed to initialize logger");

    let url = secrets
        .get("DB_DCRUST_URL")
        .expect("DB_DCRUST_URL is not set");
    let token = secrets
        .get("DB_DCRUST_TOKEN")
        .expect("DB_DCRUST_TOKEN is not set");
    let email = secrets.get("EMAIL").expect("EMAIL is not set");
    let email_pass = secrets
        .get("EMAIL_APP_PASSWORD")
        .expect("EMAIL_APP_PASSWORD is not set");
    let access_key_id = secrets
        .get("AWS_ACCESS_KEY_ID")
        .expect("AWS_ACCESS_KEY_ID is not set");
    let secret_access_key = secrets
        .get("AWS_SECRET_ACCESS_KEY")
        .expect("AWS_SECRET_ACCESS_KEY is not set");
    let region = secrets.get("AWS_REGION").expect("AWS_REGION is not set");
    let bucket = secrets.get("AWS_BUCKET").expect("AWS_BUCKET is not set");

    let db = Db::init(url, token)
        .await
        .expect("Failed to connect to database");
    // db.drop_db().await?;
    db.create_db().await.expect("Failed to create database");
    let conn_data = web::Data::new(db.get_conn().clone());

    let mail_data =
        web::Data::new(Email::init(email, email_pass).expect("Failed to connect to email"));

    let jwt = web::Data::new(JWT::init().expect("Failed to initialize JWT"));

    let s3 = web::Data::new(
        aws::S3::init(access_key_id, secret_access_key, region, bucket, "oncampus")
            .await
            .expect("Failed to connect to S3"),
    );

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("")
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
                .app_data(s3.clone())
                .service(
                    web::scope("/auth")
                        .service(register_user)
                        .service(verify_otp)
                        .service(send_otp)
                        .service(refresh_tokens)
                        .service(login),
                )
                .service(
                    web::scope("/api")
                        .wrap(from_fn(middleware::jwt))
                        .service(home)
                        .service(logout),
                )
                .service(
                    web::scope("/post")
                        // .wrap(from_fn(middleware::jwt))
                        .service(create)
                        .service(upload_url),
                )
                .default_service(web::route().to(|| async { actix_web::HttpResponse::NotFound() })),
        );
    };
    // .bind("127.0.0.1:8080")?
    // .run()
    // .await?;

    Ok(config.into())
}

#[actix_web::get("/")]
async fn home(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let claims: Arc<Claims>;
    if let Some(r) = req.extensions().get::<Arc<Claims>>() {
        claims = r.clone();
    } else {
        return Ok(HttpResponse::Unauthorized().body("Token not found"));
    }

    Ok(HttpResponse::Ok().body(format!("Welcome {}", claims.sub)))
}

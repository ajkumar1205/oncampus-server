use std::sync::Arc;

use actix_web::{web::Data, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Timestamp;

use crate::{auth::token::Claims, aws::S3};

#[actix_web::post("/create")]
pub async fn create() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::get("/upload-url")]
pub async fn upload_url(s3: Data<S3>, req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let context = uuid::Context::new(rand::random());
    let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    let uuid = uuid::Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);

    let key = format!("posts/{}/{}", user.sub, uuid.to_string());

    let url = s3.presigned_url(key).await?.uri().to_string();

    Ok(HttpResponse::Ok().json(json!(
        {
            "url": url,
        }
    )))
}

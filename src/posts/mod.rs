use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse,
};
use libsql::Connection;
use log::error;
use serde_json::json;
use uuid::{Timestamp, Uuid};
use validator::Validate;

use crate::{
    auth::token::Claims,
    aws::S3,
    models::post::{self, CreatePost, CreatePostImage},
};

#[actix_web::post("/create")]
pub async fn create(
    s3: Data<S3>,
    req: HttpRequest,
    post: Json<CreatePost>,
    conn: Data<Connection>,
) -> Result<HttpResponse, actix_web::Error> {
    post.validate().map_err(|e| {
        error!("Validation error: {}", post.validate().unwrap_err());
        error::ErrorBadRequest(serde_json::to_string(&e).unwrap_or_default())
    })?;

    let post = post.into_inner();

    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();
    let context = uuid::Context::new(rand::random());
    let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    let post_id = uuid::Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);

    let conn = conn.into_inner();
    let tran = conn.transaction().await.map_err(|e|{
        error!("Unable to create transaction for post");
        error::ErrorBadGateway("Some went wrong. Try again later")
    })?;

    post.insert_into_db(&user.sub, &post_id.to_string(), &tran)
        .await
        .map_err(|e| {
            error!("Error creating Post {}", e);
            error::ErrorBadGateway("Unable to upload post data")
        })?;

    let mut images_res: Vec<CreatePostImage> = vec![];
    for img in post.images {
        let mut image_type = "jpg".to_string();
        if let Some(image) = img.image.split(".").collect::<Vec<&str>>().last() {
            let image = image.to_string();
            if image == "jpg" || image == "png" || image == "jpeg" {
                image_type = image;
            } else {
                Err(error::ErrorBadRequest("Invalid file format"))?
            }
        } else {
            Err(error::ErrorBadRequest(
                "File name do not have . or any type",
            ))?
        }

        let image_id = Uuid::new_v4().to_string();
        let url = s3
            .presigned_url(format!("posts/{}/{}.{}", user.sub, post_id, image_type))
            .await
            .map_err(|e| {
                error!("Error Generating presigned url");
                error::ErrorBadGateway("Something went wrong while upload")
            })?;
        let post_image = CreatePostImage {
            image: url.uri().to_string(),
        };

        post_image
            .insert_into_db(&image_id, &post_id.to_string(), &tran)
            .await
            .map_err(|e| {
                error!("Error while inserting post image data {}", e);
                error::ErrorBadGateway("Something went wrong while upload")
            })?;

        images_res.push(post_image);
    }

    tran.commit().await.map_err(|e| {
        error!("Transaction of post failed at last point {}", e);
        error::ErrorBadGateway("Something unexpected happened")
    })?;

    Ok(HttpResponse::Ok().json(json!(images_res)))
}

#[actix_web::get("/list")]
pub async fn upload_url(s3: Data<S3>, req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    Ok(HttpResponse::Ok().finish())
}

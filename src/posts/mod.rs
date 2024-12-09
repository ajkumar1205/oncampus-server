use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, Json, Path, Query},
    HttpMessage, HttpRequest, HttpResponse,
};
use libsql::Connection;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::{Timestamp, Uuid};
use validator::Validate;

use crate::{
    auth::token::Claims,
    // aws::S3,
    models::comment::{CreateComment, RetrieveComment},
    models::post::{self, CreatePost, CreatePostImage, LikePost},
};

// #[actix_web::post("/create")]
// pub async fn create(
//     s3: Data<S3>,
//     req: HttpRequest,
//     post: Json<CreatePost>,
//     conn: Data<Connection>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     post.validate().map_err(|e| {
//         error!("Validation error: {}", post.validate().unwrap_err());
//         error::ErrorBadRequest(serde_json::to_string(&e).unwrap_or_default())
//     })?;

//     let post = post.into_inner();

//     let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();
//     let context = uuid::Context::new(rand::random());
//     let ts = Timestamp::from_unix(&context, 1497624119, 1234);
//     let post_id = uuid::Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);

//     let conn = conn.into_inner();
//     let tran = conn.transaction().await.map_err(|e| {
//         error!("Error: Unable to create transaction for post {}", e);
//         error::ErrorBadGateway("Some went wrong. Try again later")
//     })?;

//     post.insert_into_db(&user.sub, &post_id.to_string(), &tran)
//         .await
//         .map_err(|e| {
//             error!("Error creating Post {}", e);
//             error::ErrorBadGateway("Unable to upload post data")
//         })?;

//     let mut images_res: Vec<CreatePostImage> = vec![];
//     for img in post.images {
//         let mut image_type = "jpg".to_string();
//         if let Some(image) = img.image.split(".").collect::<Vec<&str>>().last() {
//             let image = image.to_string();
//             if image == "jpg" || image == "png" || image == "jpeg" {
//                 image_type = image;
//             } else {
//                 Err(error::ErrorBadRequest("Invalid file format"))?
//             }
//         } else {
//             Err(error::ErrorBadRequest(
//                 "File name do not have . or any type",
//             ))?
//         }

//         let image_id = Uuid::new_v4().to_string();
//         let image_key = format!("posts/{}/{}.{}", user.sub, post_id, image_type);
//         let url = s3.presigned_url(image_key.clone()).await.map_err(|e| {
//             error!("Error Generating presigned url {}", e);
//             error::ErrorBadGateway("Something went wrong while upload")
//         })?;
//         let post_image = CreatePostImage {
//             image: format!(
//                 "https://s3.ap-south-1.amazonaws.com/{}/{}",
//                 s3.bucket, image_key
//             ),
//         };

//         post_image
//             .insert_into_db(&image_id, &post_id.to_string(), &tran)
//             .await
//             .map_err(|e| {
//                 error!("Error while inserting post image data {}", e);
//                 error::ErrorBadGateway("Something went wrong while upload")
//             })?;

//         images_res.push(post_image);
//     }

//     tran.commit().await.map_err(|e| {
//         error!("Transaction of post failed at last point {}", e);
//         error::ErrorBadGateway("Something unexpected happened")
//     })?;

//     Ok(HttpResponse::Created().json(json!({ "images": images_res })))
// }

#[actix_web::post("/create")]
pub async fn create(
    // s3: Data<S3>,
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

    post.insert_into_db(&user.sub, &post_id.to_string(), &conn)
        .await
        .map_err(|e| {
            error!("Error creating Post {}", e);
            error::ErrorBadGateway("Unable to upload post data")
        })?;

    Ok(HttpResponse::Created().body("Post created"))
}

// ==================================================== LIST POSTS FOR THE MAIN PAGE ======================================================

// #[actix_web::get("/list")]
// pub async fn list_posts(
//     s3: Data<S3>,
//     req: HttpRequest,
//     conn: Data<Connection>,
//     query: Query<Option<u32>>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

//     let limit = query.unwrap_or(10);

//     let conn = conn.into_inner();
//     let posts = post::RetrieveOtherPost::retrieve_from_db(&user.sub, &conn, limit)
//         .await
//         .map_err(|e| {
//             error!("Error while retrieving posts {}", e);
//             error::ErrorBadGateway("Something went wrong while fetching posts")
//         })?;

//     Ok(HttpResponse::Ok().json(json!(posts)))
// }

// ==================================================== LIST OTHER POSTS ======================================================

#[derive(Debug, Serialize, Deserialize)]
struct CountQuery {
    count: Option<i32>,
}

#[actix_web::get("/list")]
pub async fn list_other_posts(
    // s3: Data<S3>,
    req: HttpRequest,
    conn: Data<Connection>,
    query: Query<CountQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();
    let query = query.into_inner();
    log::info!("Query: {:?}", query);

    let limit = query.count.unwrap_or(10);

    let conn = conn.into_inner();
    let posts = post::RetrieveOtherPost::retrieve_from_db(&user.sub, &conn, limit)
        .await
        .map_err(|e| {
            error!("Error while retrieving posts {}", e);
            error::ErrorBadGateway("Something went wrong while fetching posts")
        })?;

    Ok(HttpResponse::Ok().json(json!(posts)))
}

// ==================================================== LIST FRIENDS POSTS ======================================================

#[actix_web::get("/list/friends")]
pub async fn list_friends_posts(
    // s3: Data<S3>,
    req: HttpRequest,
    conn: Data<Connection>,
    query: Query<CountQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();
    let query = query.into_inner();
    log::info!("Query: {:?}", query);

    let limit = query.count.unwrap_or(10);

    let conn = conn.into_inner();
    let posts = post::RetrieveOtherPost::retrieve_from_db(&user.sub, &conn, limit)
        .await
        .map_err(|e| {
            error!("Error while retrieving posts {}", e);
            error::ErrorBadGateway("Something went wrong while fetching posts")
        })?;

    Ok(HttpResponse::Ok().json(json!(posts)))
}

// ==================================================== LIKE POST ======================================================

#[actix_web::post("/like/{post_id}")]
pub async fn like(
    req: HttpRequest,
    conn: Data<Connection>,
    post_id: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let conn = conn.into_inner();
    LikePost::insert_into_db(&user.sub, &post_id, &conn)
        .await
        .map_err(|e| {
            error!("Error while liking post {}", e);
            error::ErrorBadGateway("Something went wrong while liking post")
        })?;

    Ok(HttpResponse::Ok().body("Post liked"))
}

// ==================================================== COMMENT ON POST ======================================================

#[actix_web::post("/comment")]
pub async fn comment(
    req: HttpRequest,
    conn: Data<Connection>,
    comment: Json<CreateComment>,
) -> Result<HttpResponse, actix_web::Error> {
    comment.validate().map_err(|e| {
        error!("Validation error: {}", comment.validate().unwrap_err());
        error::ErrorBadRequest(serde_json::to_string(&e).unwrap_or_default())
    })?;

    let comment = comment.into_inner();
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let conn = conn.into_inner();

    comment
        .insert_into_db(&user.sub, &conn)
        .await
        .map_err(|e| {
            error!("Error while commenting on post {}", e);
            error::ErrorBadGateway("Something went wrong while commenting on post")
        })?;

    Ok(HttpResponse::Created().body("Commentd added"))
}

// ==================================================== RETRIEVE COMMENTS ======================================================

#[actix_web::get("/comments/{post_id}")]
pub async fn list_comments(
    // req: HttpRequest,
    conn: Data<Connection>,
    post_id: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let conn = conn.into_inner();
    let comments = RetrieveComment::retrieve_from_db(&post_id, &conn)
        .await
        .map_err(|e| {
            error!("Error while retrieving comments {}", e);
            error::ErrorBadGateway("Something went wrong while fetching comments")
        })?;

    Ok(HttpResponse::Ok().json(json!(comments)))
}

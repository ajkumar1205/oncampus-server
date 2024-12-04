use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::NaiveDateTime;
use validator_derive::Validate;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Post {
    #[validate(length(max = 1000))]
    pub text: String,
    pub user: String,
    pub likes: i32,
    pub comments: i32,
    pub images: Vec<PostImage>,
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostImage {
    pub post: String,
    #[validate(url(message = "Invalid URL"))]
    pub image_url: String,
    pub created_at: NaiveDateTime,
}


impl Post {
    
}
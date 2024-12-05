use std::sync::Arc;

use libsql::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePost {
    #[validate(length(max = 1000))]
    pub text: String,
    pub public: bool,
    pub images: Vec<CreatePostImage>
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostImage {
    pub image: String,
}

impl Serialize for CreatePostImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.image.serialize(serializer)
    }
}

impl CreatePost {
    pub async fn insert_into_db(&self, user: &String, uuid: &String, conn: &Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let text = self.text.clone();
        let public = self.public;

        conn.execute(
            r#"
            INSERT INTO posts (id, user, text, public)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![uuid.to_string(), user.clone(), text, public]
        ).await?;

        Ok(())

    }
}


impl CreatePostImage {
    pub async fn insert_into_db(&self, id: &String, post: &String, conn: &Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let image_url = self.image.clone();
        conn.execute(
            r#"
            INSERT INTO post_images (id, post, image_url)
            VALUES (?1, ?2, ?3)
            "#,
            params![id.clone(), post.clone(), image_url],
        )
        .await?;
        Ok(())
    }
}


pub struct RetrieveOtherPost {
    pub id: String,
    pub user: String,
    
}
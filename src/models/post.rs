use libsql::{params, Connection, Transaction};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePost {
    #[validate(length(max = 1000))]
    pub text: String,
    pub public: bool,
    // pub images: Vec<CreatePostImage>
}

#[derive(Debug, Validate)]
pub struct CreatePostImage {
    pub image: String,
}

impl Serialize for CreatePostImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.image)
    }
}

impl<'de> Deserialize<'de> for CreatePostImage {
    fn deserialize<D>(deserializer: D) -> Result<CreatePostImage, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(CreatePostImage { image: s })
    }
}

impl CreatePost {
    pub async fn insert_into_db(
        &self,
        user: &String,
        uuid: &String,
        conn: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = self.text.clone();
        let public = self.public;

        conn.execute(
            r#"
            INSERT INTO posts (id, user, text, public)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![uuid.to_string(), user.clone(), text, public],
        )
        .await?;

        Ok(())
    }
}

impl CreatePostImage {
    pub async fn insert_into_db(
        &self,
        id: &String,
        post: &String,
        conn: &Transaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveOtherPost {
    pub id: String,
    pub user: String,
    pub username: String,
    // pub user_profile: Option<String>,
    pub likes: u32,
    pub comments: u32,
    pub text: String,
    // pub images: Vec<String>,
    pub created_at: String,
}

impl RetrieveOtherPost {
    pub async fn retrieve_from_db(
        user: &String,
        conn: &Connection,
        limit: u32,
    ) -> Result<Vec<RetrieveOtherPost>, Box<dyn std::error::Error>> {
        let mut posts = vec![];
        // let mut stmt = conn.prepare(
        //     r#"
        //         SELECT posts.id, posts.user, users.username, users.profile_url, posts.likes, posts.comments, posts.text, posts.created_at
        //         FROM users
        //         INNER JOIN posts ON users.id = posts.user
        //         WHERE posts.public = true
        //         ORDER BY posts.created_at DESC
        //         LIMIT ?1
        //     "#
        // ).await?;

        let mut stmt = conn.prepare(
            r#"
                SELECT posts.id, posts.user, users.username, posts.likes, posts.comments, posts.text, posts.created_at
                FROM users 
                INNER JOIN posts ON users.id = posts.user
                WHERE posts.public = true
                ORDER BY posts.created_at DESC
                LIMIT ?1
            "#
        ).await?;

        let mut rows = stmt.query(params![limit]).await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let user: String = row.get(1)?;
            let username: String = row.get(2)?;
            // let user_profile = row.get(3)?;
            let likes: u32 = row.get(3)?;
            let comments: u32 = row.get(4)?;
            let text: String = row.get(5)?;
            let created_at: String = row.get(6)?;

            posts.push(RetrieveOtherPost {
                id,
                user,
                username,
                // user_profile,
                likes,
                comments,
                text,
                // images,
                created_at,
            });
        }

        Ok(posts)
    }
}

pub struct LikePost;

impl LikePost {
    pub async fn insert_into_db(
        user: &String,
        post: &String,
        conn: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tran = conn.transaction().await?;
        tran.execute(
            r#"
            INSERT INTO post_likes (id, post, user)
            VALUES (?1, ?2, ?3)
            "#,
            params![Uuid::new_v4().to_string(), post.clone(), user.clone()],
        )
        .await?;

        tran.execute(
            r#"
            UPDATE posts
            SET likes = likes + 1
            WHERE id = ?1
            "#,
            params![post.clone()],
        )
        .await?;

        tran.commit().await?;
        Ok(())
    }
}

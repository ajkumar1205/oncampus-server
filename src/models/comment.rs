use libsql::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use validator_derive::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateComment {
    pub post: String,
    #[validate(length(max = 500))]
    pub text: String,
}

impl CreateComment {
    pub async fn insert_into_db(
        &self,
        user: &String,
        conn: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = self.text.clone();
        let post = self.post.clone();

        let tran = conn.transaction().await?;
        tran.execute(
            r#"
            INSERT INTO post_comments (id, user, post, text)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![Uuid::new_v4().to_string(), user.clone(), post.clone(), text],
        )
        .await?;

        tran.execute(
            r#"
            UPDATE posts
            SET comments = comments + 1
            WHERE id = ?1
        "#,
            params![post],
        )
        .await?;

        tran.commit().await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveComment {
    pub id: String,
    pub user: String,
    pub username: String,
    pub text: String,
    pub created_at: String,
}

impl RetrieveComment {
    pub async fn retrieve_from_db(
        post: &String,
        conn: &Connection,
    ) -> Result<Vec<RetrieveComment>, Box<dyn std::error::Error>> {
        let mut comments = vec![];

        let mut rows = conn
            .query(
                r#"
            SELECT post_comments.id, post_comments.user, users.username, post_comments.text, post_comments.created_at
            FROM users
            INNER JOIN post_comments
            ON users.id = post_comments.user
            WHERE post_comments.post = ?1
            ORDER BY post_comments.created_at DESC
            "#,
                params![post.clone()],
            )
            .await?;

        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let user: String = row.get(1)?;
            let username: String = row.get(2)?;
            let text: String = row.get(3)?;
            let created_at: String = row.get(4)?;

            comments.push(RetrieveComment {
                id,
                user,
                username,
                text,
                created_at,
            });
        }

        Ok(comments)
    }
}

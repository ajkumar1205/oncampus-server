use libsql::{params, Builder, Connection, Database};
use std::{sync::Arc, time::Duration};

pub struct Db {
    conn: Connection,
    database: Database,
}

impl Db {
    pub async fn init(url: String, auth_token: String) -> Result<Self, libsql::Error> {
        let db = Builder::new_remote_replica("local.db", url, auth_token)
            .sync_interval(Duration::from_secs(30))
            .build()
            .await?;

        let conn: Connection = db.connect()?;

        Ok(Self { conn, database: db })
    }

    pub async fn create_db(&self) -> Result<(), libsql::Error> {
        let create_users_table = r#"
                CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    roll TEXT NOT NULL UNIQUE,
                    username TEXT NOT NULL UNIQUE,
                    password TEXT NOT NULL,
                    first_name TEXT NOT NULL,
                    last_name TEXT NOT NULL,
                    posts INTEGER DEFAULT 0,
                    followers INTEGER DEFAULT 0,
                    following INTEGER DEFAULT 0,
                    email TEXT NOT NULL UNIQUE,
                    dob DATE NOT NULL,
                    is_active BOOLEAN DEFAULT FALSE,
                    is_superuser BOOLEAN DEFAULT FALSE,
                    profile_url TEXT,
                    bio TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#;

        let create_followers_table = r#"
                CREATE TABLE IF NOT EXISTS followers (
                    follower_id TEXT NOT NULL,
                    followed_id TEXT NOT NULL,
                    PRIMARY KEY (follower_id, followed_id),
                    FOREIGN KEY (follower_id) REFERENCES users (id) ON DELETE CASCADE,
                    FOREIGN KEY (followed_id) REFERENCES users (id) ON DELETE CASCADE
                );
            "#;

        let create_posts_table = r#"
                CREATE TABLE IF NOT EXISTS posts (
                    id TEXT PRIMARY KEY,
                    user TEXT NOT NULL,
                    text TEXT,
                    pubic BOOLEAN DEFAULT TRUE,
                    likes INTEGER DEFAULT 0,
                    comments INTEGER DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE
                )
            "#;

        let create_post_image_table = r#"
                CREATE TABLE IF NOT EXISTS post_images (
                    id TEXT PRIMARY KEY,
                    post TEXT NOT NULL,
                    image_url TEXT NOT NULL,
                    FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE
                )
            "#;

        let create_post_likes_table = r#"
                CREATE TABLE IF NOT EXISTS post_likes (
                    id TEXT PRIMARY KEY,
                    post TEXT NOT NULL,
                    user TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
                    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE
                )
            "#;

        let create_post_comments_table = r#"
                CREATE TABLE IF NOT EXISTS post_comments (
                    id TEXT PRIMARY KEY,
                    post TEXT NOT NULL,
                    user TEXT NOT NULL,
                    text TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
                    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE
                )
            "#;

        let create_otp_table = r#"
                CREATE TABLE IF NOT EXISTS otps (
                    email TEXT PRIMARY KEY,
                    otp TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#;

        let create_tokens_table = r#"
                CREATE TABLE IF NOT EXISTS tokens (
                    token TEXT PRIMARY KEY
                );
                "#;

        let create_user_id_index = r#"
                CREATE INDEX IF NOT EXISTS idx_id ON users (id);
                "#;

        let create_follower_id_index = r#"
                CREATE INDEX IF NOT EXISTS idx_follower_id ON followers (follower_id);
                "#;

        let create_followed_id_index = r#"
                CREATE INDEX IF NOT EXISTS idx_followed_id ON followers (followed_id);
                "#;

        let create_otp_email_index = r#"
                CREATE INDEX IF NOT EXISTS idx_email ON otps (email);
                "#;

        self.conn.execute(create_users_table, params!()).await?;
        self.conn.execute(create_followers_table, params!()).await?;
        self.conn.execute(create_posts_table, params!()).await?;
        self.conn
            .execute(create_post_image_table, params!())
            .await?;
        self.conn
            .execute(create_post_likes_table, params!())
            .await?;
        self.conn
            .execute(create_post_comments_table, params!())
            .await?;
        self.conn
            .execute(create_follower_id_index, params!())
            .await?;
        self.conn
            .execute(create_followed_id_index, params!())
            .await?;
        self.conn.execute(create_otp_table, params!()).await?;
        self.conn.execute(create_otp_email_index, params!()).await?;
        self.conn.execute(create_user_id_index, params!()).await?;
        self.conn.execute(create_tokens_table, params!()).await?;

        Ok(())
    }

    pub fn get_db(&self) -> &Database {
        &self.database
    }

    pub fn get_conn(&self) -> &Connection {
        &self.conn
    }

    pub async fn drop_db(&self) -> Result<(), libsql::Error> {
        let drop_users_table = r#"
            DROP TABLE IF EXISTS users;
            "#;

        let drop_followers_table = r#"
            DROP TABLE IF EXISTS followers;
            "#;

        let drop_posts_table = r#"
            DROP TABLE IF EXISTS posts;
            "#;

        let drop_post_images_table = r#"
            DROP TABLE IF EXISTS post_images;
            "#;

        let drop_post_likes_table = r#"
            DROP TABLE IF EXISTS post_likes;
            "#;

        let drop_post_comments_table = r#"
            DROP TABLE IF EXISTS post_comments;
            "#;

        let drop_otp_table = r#"
            DROP TABLE IF EXISTS otps;
            "#;

        let drop_tokens_table = r#"
            DROP TABLE IF EXISTS tokens;
            "#;

        let drop_users_id_index = r#"
            DROP INDEX IF EXISTS idx_id;
            "#;

        let drop_email_id_index = r#"
            DROP INDEX IF EXISTS idx_email;
            "#;

        let drop_follower_id_index = r#"
            DROP INDEX IF EXISTS idx_follower_id;
            "#;

        let drop_followed_id_index = r#"
            DROP INDEX IF EXISTS idx_followed_id;
            "#;

        self.conn.execute(drop_users_table, params!()).await?;
        self.conn.execute(drop_followers_table, params!()).await?;
        self.conn.execute(drop_otp_table, params!()).await?;
        self.conn.execute(drop_posts_table, params!()).await?;
        self.conn
            .execute(drop_post_comments_table, params!())
            .await?;
        self.conn.execute(drop_post_images_table, params!()).await?;
        self.conn.execute(drop_post_likes_table, params!()).await?;
        self.conn.execute(drop_tokens_table, params!()).await?;
        self.conn.execute(drop_email_id_index, params!()).await?;
        self.conn.execute(drop_follower_id_index, params!()).await?;
        self.conn.execute(drop_followed_id_index, params!()).await?;
        self.conn.execute(drop_users_id_index, params!()).await?;

        Ok(())
    }
}

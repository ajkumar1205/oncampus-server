use libsql::{params, Builder, Connection, Database};
use std::{sync::Arc, time::Duration};

pub struct Db {
    conn: Arc<Connection>,
    database: Database,
}

impl Db {
    pub async fn init_turso(url: String, auth_token: String) -> Result<Self, libsql::Error> {
        let db = Builder::new_remote_replica("local.db", url, auth_token)
            .sync_interval(Duration::from_secs(60))
            .build()
            .await?;

        let conn: Connection = db.connect()?;

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
                    profile_url TEXT
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

        let create_otp_table = r#"
                CREATE TABLE IF NOT EXISTS otps (
                    email TEXT PRIMARY KEY,
                    otp TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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

        conn.execute(create_users_table, params!()).await?;
        conn.execute(create_followers_table, params!()).await?;
        conn.execute(create_follower_id_index, params!()).await?;
        conn.execute(create_followed_id_index, params!()).await?;
        conn.execute(create_otp_table, params!()).await?;
        conn.execute(create_otp_email_index, params!()).await?;
        conn.execute(create_user_id_index, params!()).await?;

        Ok(Self {
            conn: Arc::new(conn),
            database: db,
        })
    }

    pub async fn init_mongo() {}

    pub fn get_db(&self) -> &Database {
        &self.database
    }

    pub fn get_conn(&self) -> Arc<Connection> {
        self.conn.clone()
    }

    pub async fn drop_db(&self) -> Result<(), libsql::Error> {
        let drop_users_table = r#"
            DROP TABLE IF EXISTS users;
            "#;

        let drop_followers_table = r#"
            DROP TABLE IF EXISTS followers;
            "#;

        let drop_otp_table = r#"
            DROP TABLE IF EXISTS otps;
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
        self.conn.execute(drop_email_id_index, params!()).await?;
        self.conn.execute(drop_follower_id_index, params!()).await?;
        self.conn.execute(drop_followed_id_index, params!()).await?;
        self.conn.execute(drop_users_id_index, params!()).await?;

        Ok(())
    }
}

use libsql::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProfile {
    pub bio: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    // pub image: Option<String>,
}

impl UpdateProfile {
    pub async fn update_into_db(
        &self,
        conn: &Connection,
        user: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.bio.is_none()
            // && self.image.is_none()
            && self.first_name.is_none()
            && self.last_name.is_none()
        {
            return Ok(());
        }

        let mut query = String::from("UPDATE users SET ");
        let mut params_vec = Vec::new();

        if let Some(bio) = &self.bio {
            query.push_str("bio = ?1, ");
            params_vec.push(bio.clone());
        }

        // if let Some(image) = &self.image {
        //     query.push_str("profile_url = ?2, ");
        //     params_vec.push(image.clone());
        // }

        if let Some(first_name) = &self.first_name {
            query.push_str("first_name = ?2, ");
            params_vec.push(first_name.clone());
        }

        if let Some(last_name) = &self.last_name {
            query.push_str("last_name = ?3, ");
            params_vec.push(last_name.clone());
        }

        query = query.trim_end_matches(", ").to_string(); // Remove trailing comma
        query.push_str(" WHERE id = ?4");
        params_vec.push(user.to_string());

        // Execute the query
        conn.execute(&query, params_vec).await?;

        Ok(())
    }
}


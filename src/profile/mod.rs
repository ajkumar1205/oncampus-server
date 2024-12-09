use std::sync::Arc;

use actix_multipart::form::json;
use actix_web::{
    error,
    web::{Data, Form, Query},
    HttpMessage, HttpRequest, HttpResponse,
};
use libsql::Connection;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    auth::token::Claims,
    models::profile::{RetrieveProfile, UpdateProfile},
};

#[actix_web::post("/update")]
pub async fn update(
    req: HttpRequest,
    conn: Data<Connection>,
    form: Form<UpdateProfile>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let conn = conn.into_inner();
    form.update_into_db(&conn, &user.sub).await.map_err(|e| {
        error!("Error while updating profile {}", e);
        error::ErrorBadRequest("Something went wrong while updating profile")
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchProfile {
    string: String,
}

#[actix_web::get("/search")]
pub async fn search(
    req: HttpRequest,
    conn: Data<Connection>,
    query: Query<SearchProfile>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();
    let query = query.into_inner();

    log::info!("Query {:?}", query);

    let conn = conn.into_inner();
    let profiles = RetrieveProfile::get_from_db(&query.string.to_lowercase(), &conn)
        .await
        .map_err(|e| {
            error!("Error while searching profiles {}", e);
            error::ErrorBadGateway("Something went wrong while searching profiles")
        })?;

    Ok(HttpResponse::Ok().json(json!(profiles)))
}

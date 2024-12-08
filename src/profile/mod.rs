use std::sync::Arc;

use actix_web::{web::{Data, Form}, error, HttpMessage, HttpRequest, HttpResponse};
use libsql::Connection;
use log::error;

use crate::{auth::token::Claims, models::profile::UpdateProfile};


#[actix_web::post("/update")]
pub async fn update(
    req: HttpRequest,
    conn: Data<Connection>,
    form: Form<UpdateProfile>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req.extensions().get::<Arc<Claims>>().unwrap().clone();

    let conn = conn.into_inner();
    form.update_into_db(&conn, &user.sub)
        .await
        .map_err(|e| {
            error!("Error while updating profile {}", e);
            error::ErrorBadRequest("Something went wrong while updating profile")
        })?;

    Ok(HttpResponse::Ok().finish())
}

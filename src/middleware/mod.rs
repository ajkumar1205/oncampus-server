use std::{rc::Rc, sync::Arc};

use crate::auth::token::{Claims, JWT};
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Extensions, ServiceRequest, ServiceResponse},
    error,
    middleware::Next,
    web::Data,
    Error, HttpMessage,
};
use libsql::Connection;

pub async fn jwt<B>(req: ServiceRequest, next: Next<B>) -> Result<ServiceResponse<BoxBody>, Error>
where
    B: MessageBody + 'static,
{
    let jwt = req.app_data::<Data<JWT>>().unwrap();

    let conn = req.app_data::<Data<Connection>>().unwrap();
    let token = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| error::ErrorUnauthorized("Token not found"))?
        .to_str()
        .unwrap()
        .to_string()
        .replace("Bearer ", "")
        .replace(" ", "");

    let mut claims: Claims;
    if let Ok(r) = Claims::decode(&token, jwt) {
        claims = r;
    } else {
        return Ok(req.error_response(error::ErrorUnauthorized("Invalid Token")));
    }

    if claims.token != "access" {
        return Ok(req.error_response(error::ErrorUnauthorized("Use Access token")));
    }

    if !Claims::is_valid(&token, conn, jwt)
        .await
        .map_err(|e| error::ErrorInternalServerError("Something went wrong"))?
    {
        return Ok(req.error_response(error::ErrorUnauthorized("Token is blacklisted")));
    }

    req.extensions_mut().insert(Arc::new(claims));
    req.extensions_mut().insert(Arc::new(token));

    let res = next.call(req).await?;
    Ok(res.map_into_boxed_body())
}

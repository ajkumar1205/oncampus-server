use actix_web::HttpResponse;





#[actix_web::post("/create")]
pub async fn create() -> Result<HttpResponse, actix_web::Error>{

    Ok(HttpResponse::Ok().finish())
}


#[actix_web::get("/upload-url")]
pub async fn upload_url() -> Result<HttpResponse, actix_web::Error>{

    Ok(HttpResponse::Ok().finish())
}
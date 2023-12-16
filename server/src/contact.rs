use actix_web::{HttpRequest, HttpResponse, Result};
use std::path::PathBuf;

pub async fn get_handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/contact.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// pub async fn post_handler(form: web::Form<MyForm>) -> Result<HttpResponse> {
//     // Process the form...
//     Ok(HttpResponse::Ok().into())
// }
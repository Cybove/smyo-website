use crate::src::db;
use actix_web::{web::Json, HttpRequest, HttpResponse, Responder, Result};
use serde::Serialize;
use serde_derive::Deserialize;
use std::path::PathBuf;
use actix_web::web::Query;

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/main_content.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
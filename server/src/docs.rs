use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/dokuman.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn doc_handler(filename: web::Path<String>) -> Result<HttpResponse> {
    let filename_str = filename.into_inner();
    let mut path: PathBuf = "../public/assets/docs".parse().unwrap();
    path.push(&filename_str);

    let content = fs::read(path).await?;

    Ok(HttpResponse::Ok()
        .header(
            "Content-Disposition",
            format!("attachment; filename={}", filename_str),
        )
        .body(content))
}

use crate::src::db;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::Deserialize;
use std::path::PathBuf;

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/admin.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn dashboard_handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/dashboard.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn login_handler(form: web::Form<LoginForm>) -> impl Responder {
    let login_form = form.into_inner();

    println!("Username: {}", login_form.username);
    println!("Password: {}", login_form.password);

    if db::authenticate_user(&login_form.username, &login_form.password).unwrap() {
        let path: PathBuf = "../public/pages/dashboard.html".parse().unwrap();
        let content = tokio::fs::read_to_string(path).await.unwrap();
        HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("<div id=\"main-container\">{}</div>", content))
    } else {
        HttpResponse::Found()
            .header("location", "/admin?error=Invalid username or password")
            .finish()
    }
}

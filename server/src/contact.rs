use crate::src::db::contact_message;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_derive::Deserialize;
use std::path::PathBuf;

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/contact.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
    message: String,
}

pub async fn post_handler(req: HttpRequest, form: web::Form<FormData>) -> Result<HttpResponse> {
    let form_data = form.into_inner();

    let ip_address = match req.peer_addr() {
        Some(addr) => addr.ip().to_string(),
        None => String::from("Unknown"),
    };

    match contact_message(&form_data.name, &form_data.email, &form_data.message, &ip_address) {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(format!(
                "<div class='flex flex-col items-center justify-center h-screen'>
                    <h1 class='text-4xl font-bold text-gray-900'>Mesajınız Alınmıştır</h1>
                    <a href='#' hx-get='/main' hx-target='#main-container' hx-push-url='#' hx-trigger='click'
                    class='py-4 px-5 text-gray-900 rounded max-w-xs md:bg-transparent flex items-center' aria-current='page'>
                        <svg class='w-6 h-6 mr-2' fill='none' stroke='currentColor' viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
                            <path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M11 19l-7-7 7-7m8 14l-7-7 7-7'></path>
                        </svg>
                    </a>
                </div>"
            ))),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(format!(
                "<div class='flex flex-col items-center justify-center h-screen'>
                    <h1 class='text-4xl font-bold text-gray-900'>Mesajınız Gönderilemedi Lütfen Daha Sonra Tekrar Deneyiniz</h1>
                    <a href='#' hx-get='/main' hx-target='#main-container' hx-push-url='#' hx-trigger='click'
                    class='py-4 px-5 text-gray-900 rounded max-w-xs md:bg-transparent flex items-center' aria-current='page'>
                        <svg class='w-6 h-6 mr-2' fill='none' stroke='currentColor' viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
                            <path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M11 19l-7-7 7-7m8 14l-7-7 7-7'></path>
                        </svg>
                    </a>
                </div>"
            ))),
    }
}

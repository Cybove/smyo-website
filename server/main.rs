mod src;

use crate::src::db;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(src::index::handler))
            .route("/main", web::get().to(src::main_content::handler))
            .route(
                "/announcements",
                web::get().to(src::main_content::announcements_handler),
            )
            .route(
                "/announcement/{id}",
                web::get().to(src::main_content::announcement_detail_handler),
            )
            .route("/contact", web::get().to(src::contact::get_handler))
            .route("/programlar", web::get().to(src::programlar::handler))
            .service(Files::new("/node_modules", "../node_modules"))
            .service(Files::new("/pages", "../public/pages").index_file("index.html")) // Specify index file
            .service(Files::new("/", "../public").index_file("index.html")) // Specify index file
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

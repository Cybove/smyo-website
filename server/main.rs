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
            .route("/admin", web::get().to(src::admin::handler))
            .route("/login", web::post().to(src::admin::login_handler))
            .route("/admin/dashboard", web::get().to(src::admin::dashboard_handler))
            .route("/main", web::get().to(src::main_content::handler))
            .route(
                "/announcements",
                web::get().to(src::main_content::announcements_handler),
            )
            .route(
                "/announcement/{id}",
                web::get().to(src::main_content::announcement_detail_handler),
            )
            .route("/contact", web::get().to(src::contact::handler))
            .route("/contact", web::post().to(src::contact::post_handler))
            .route("/duyurular", web::get().to(src::duyurular::handler))
            .service(Files::new("/node_modules", "../node_modules"))
            .service(Files::new("/pages", "../public/pages").index_file("index.html"))
            .service(Files::new("/", "../public").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

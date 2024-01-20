mod src;

use crate::src::db;
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    let secret_key = Key::generate();
    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(10_262_144)) // Set max JSON payload size to 10MB
            .data(web::FormConfig::default().limit(10_485_760)) // Set max Form payload size to 10MB
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .route("/", web::get().to(src::index::handler))
            .route("/admin", web::get().to(src::admin::handler))
            .route("/login", web::post().to(src::admin::login_handler))
            .route(
                "/admin/dashboard",
                web::get().to(src::admin::admin_dashboard_handler),
            )
            .route("/main", web::get().to(src::main_content::handler))
            .route(
                "/announcements",
                web::get().to(src::main_content::announcements_handler),
            )
            .route(
                "/announcement/{id}",
                web::get().to(src::main_content::announcement_detail_handler),
            )
            .route("/admin/user", web::get().to(src::admin::admin_user_handler))
            .route(
                "/admin/announcements",
                web::get().to(src::admin::admin_announcements_handler),
            )
            .route("/contact", web::get().to(src::contact::handler))
            .route("/contact", web::post().to(src::contact::post_handler))
            .route("/duyurular", web::get().to(src::duyurular::handler))
            .service(
                web::resource("/admin/announcements/add")
                    .route(web::post().to(src::admin::add_announcement_handler)),
            )
            .service(
                web::resource("/admin/announcements/add/form")
                    .route(web::get().to(src::admin::add_announcement_form_handler)),
            )
            .service(
                web::resource("/admin/announcement/edit/form/{id}")
                    .route(web::get().to(src::admin::edit_announcement_form_handler)),
            )
            .service(
                web::resource("/admin/announcement/edit")
                    .route(web::post().to(src::admin::edit_announcement_handler)),
            )
            .service(
                web::resource("/admin/announcements/delete/{id}")
                    .route(web::post().to(src::admin::delete_announcement_handler)),
            )
            .service(Files::new("/node_modules", "../node_modules"))
            .service(Files::new("/pages", "../public/pages").index_file("index.html"))
            .service(Files::new("/", "../public").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

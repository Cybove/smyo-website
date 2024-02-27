mod src;

use crate::src::db;
use actix_cors::Cors;
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::{Cookie, Key, SameSite};
use actix_web::{middleware, web, App, HttpServer};
// use env_logger::Env;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let secret_key = Key::generate();
    let ip_address = "192.168.1.6";
    let port = "1907";

    HttpServer::new(move || {
        App::new()
            // .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(10_262_144)) // Set max JSON payload size to 10MB
            .data(web::FormConfig::default().limit(10_485_760)) // Set max Form payload size to 10MB
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    // .cookie_domain(Some(ip_address.to_string()))
                    .build(),
            )
            .route("/", web::get().to(src::index::handler))
            .route("/admin", web::get().to(src::admin::handler))
            .route("/login", web::post().to(src::admin::login_handler))
            .route("/logout", web::get().to(src::admin::logout_handler))
            .route(
                "/admin/dashboard",
                web::get().to(src::admin::admin_dashboard_handler),
            )
            .route("/main", web::get().to(src::main_content::handler))
            .route("/slider", web::get().to(src::slider::handler))
            .route(
                "/announcements/{page}",
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
            .route(
                "/admin/inbox",
                web::get().to(src::admin::admin_inbox_handler),
            )
            .route(
                "/admin/gallery",
                web::get().to(src::admin::admin_gallery_handler),
            )
            .route(
                "/admin/messages",
                web::get().to(src::admin::get_messages_handler),
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
            .service(
                web::resource("/admin/user/list")
                    .route(web::get().to(src::admin::get_user_list_handler)),
            )
            .service(
                web::resource("/admin/user/add/form")
                    .route(web::get().to(src::admin::add_user_form_handler)),
            )
            .service(
                web::resource("/admin/user/add")
                    .route(web::post().to(src::admin::add_user_handler)),
            )
            .service(
                web::resource("/admin/user/edit/form/{username}")
                    .route(web::get().to(src::admin::edit_user_form_handler)),
            )
            .service(
                web::resource("/admin/user/edit/{username}")
                    .route(web::post().to(src::admin::edit_user_handler)),
            )
            .service(
                web::resource("/admin/user/delete/{username}")
                    .route(web::delete().to(src::admin::delete_user_handler)),
            )
            .service(Files::new("/node_modules", "../node_modules"))
            .service(Files::new("/pages", "../public/pages").index_file("index.html"))
            .service(Files::new("/public/assets/slider", "../public/assets/slider"))
            .service(Files::new("/", "../public").index_file("index.html"))
    })
    .bind(format!("{}:{}", ip_address, port))?
    .run()
    .await
}

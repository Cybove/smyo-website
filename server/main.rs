use actix_files::Files;
use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
use std::path::PathBuf;

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "../public/pages/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .service(Files::new("/node_modules", "../node_modules"))
            .service(Files::new("/", "../public"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

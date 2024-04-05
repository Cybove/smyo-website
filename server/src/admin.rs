use crate::src::db;
use actix_multipart::{Field, Multipart};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::header::ContentDisposition;
use actix_web::web::BytesMut;
use actix_web::web::Query;
use actix_web::web::{self, Bytes};
use actix_web::{Error, HttpRequest, HttpResponse, Responder, Result};
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use image::imageops::resize;
use image::imageops::FilterType;
use image::GenericImageView;
use serde_derive::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct User {
    name: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct AnnouncementForm {
    pub id: i32,
    pub image: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct AddAnnouncementForm {
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
    pub image_path: String,
}

#[derive(Debug, Deserialize)]
pub struct EditAnnouncementForm {
    pub id: i32,
    pub image_path: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct DeleteAnnouncementForm {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct ArticleForm {
    pub id: i32,
    pub image: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct AddArticleForm {
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
    pub image_path: String,
}

#[derive(Debug, Deserialize)]
pub struct EditArticleForm {
    pub id: i32,
    pub image_path: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct DeleteArticleForm {
    pub id: i32,
}

#[derive(Deserialize, Clone)]
pub struct Pagination {
    page: Option<usize>,
    page_size: Option<usize>,
}

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/admin.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn login_handler(form: web::Form<LoginForm>, mut session: Session) -> impl Responder {
    let login_form = form.into_inner();

    match db::authenticate_user(&login_form.username, &login_form.password) {
        Ok((is_authenticated, name)) => {
            if is_authenticated {
                match session.insert("user_id", login_form.username) {
                    Ok(_) => (),
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .body(format!("Failed to set session: {}", e))
                    }
                }
                if let Some(name) = name {
                    session.insert("user_name", name).unwrap();
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().body("Authentication failed");
        }
    }

    match session.get::<String>("user_id") {
        Ok(user_id_option) => {
            if user_id_option.is_some() {
                let path: PathBuf = "../public/pages/dashboard.html".parse().unwrap();
                let content = tokio::fs::read_to_string(path).await.unwrap();
                HttpResponse::Ok().content_type("text/html").body(content)
            } else {
                HttpResponse::Unauthorized().body("Login failed")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to get session"),
    }
}

pub async fn logout_handler(mut session: Session) -> Result<HttpResponse> {
    session.remove("user_id");
    Ok(HttpResponse::Ok().finish())
}

pub async fn admin_dashboard_handler(session: Session) -> Result<HttpResponse> {
    match session.get::<String>("user_id") {
        Ok(user_id_option) => {
            if let Some(_) = user_id_option {
                let path: PathBuf = "../public/pages/dashboard.html".parse().unwrap();
                let content = tokio::fs::read_to_string(path).await?;
                Ok(HttpResponse::Ok().content_type("text/html").body(content))
            } else {
                Ok(HttpResponse::Found().header("Location", "/admin").finish())
            }
        }
        Err(_) => Err(actix_web::error::ErrorInternalServerError(
            "Failed to get session",
        )),
    }
}

pub async fn admin_announcements_handler(
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse> {
    let page: i32 = pagination.page.unwrap_or(1).try_into().unwrap();
    let page_size: i32 = pagination.page_size.unwrap_or(3).try_into().unwrap();

    let (announcements, total_announcements) = db::get_announcements(page, page_size)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let total_pages = (total_announcements as f32 / page_size as f32).ceil() as i32;

    let mut content = String::from("
    <div class='flex justify-center'>
        <button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 mt-4 rounded' hx-get='/admin/announcements/add/form' hx-swap='innerHTML' hx-target='#dashboard-container'>           Yeni Duyuru Ekle           </button>
    </div>
    ");
    for announcement in &announcements {
        let mut announcement_content =
            tokio::fs::read_to_string("../public/pages/announcements.html").await?;
        announcement_content = announcement_content.replace("{image}", &announcement.image);
        announcement_content = announcement_content.replace("{title}", &announcement.title);
        announcement_content = announcement_content.replace("{date}", &announcement.date);
        announcement_content = announcement_content.replace("{author}", &announcement.author);
        announcement_content = announcement_content.replace("{id}", &announcement.id.to_string());
        content.push_str(&announcement_content);
    }

    content.push_str("<div class='flex justify-center items-center mt-4 mb-4 space-x-2'>");
    if page > 1 {
        content.push_str(&format!(
            "<button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded' hx-get='/admin/announcements?page={}&page_size={}' hx-swap='innerHTML' hx-target='#dashboard-container'>Önceki Sayfa</button>",
            page - 1,
            page_size
        ));
    }
    if page < total_pages {
        content.push_str(&format!(
            "<button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded' hx-get='/admin/announcements?page={}&page_size={}' hx-swap='innerHTML' hx-target='#dashboard-container'>Sonraki Sayfa</button>",
            page + 1,
            page_size
        ));
    }
    content.push_str("</div>");

    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn add_announcement_handler(
    mut payload: Multipart,
    session: Session,
) -> Result<HttpResponse, Error> {
    let mut image: Option<Bytes> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut date: Option<String> = None;
    let mut author: Option<String> = None;
    let mut image_path: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().unwrap();
        let filename = content_disposition.get_filename().unwrap_or("unnamed");
        let extension = Path::new(filename)
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap();

        match name {
            "image" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                image = Some(bytes.freeze());
                image_path = Some(format!(
                    "../public/assets/image/upload/{}.{}",
                    Uuid::new_v4().to_string(),
                    extension
                ));
                let mut file = fs::File::create(&image_path.as_ref().unwrap()).unwrap();
                file.write_all(&image.as_ref().unwrap()).unwrap(); // Use as_ref to avoid moving image
            }
            "title" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                title = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "content" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                content = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "author" => {
                match session.get::<String>("user_id") {
                    Ok(user_id_option) => {
                        if let Some(username) = user_id_option {
                            author = Some(username);
                        }
                    }
                    Err(_) => {
                        // Handle case where getting user from session failed
                    }
                }
            }
            "date" => {
                date = Some(chrono::Local::now().format("%d-%m-%Y").to_string());
            }
            _ => (),
        }
    }

    let image = image.unwrap();
    let title = title.unwrap();
    let content = content.unwrap();
    let date = date.unwrap();
    let author = author.unwrap();

    let image_path = image_path.unwrap();

    let db_image_path = image_path.replace("../public", "");

    let mut file = fs::File::create(&image_path).unwrap();
    file.write_all(&image).unwrap();

    let form = AddAnnouncementForm {
        title,
        content,
        date,
        author,
        image_path: db_image_path,
    };

    match db::add_announcement(
        &form.image_path,
        &form.title,
        &form.content,
        &form.date,
        &form.author,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn add_announcement_form_handler() -> Result<HttpResponse, actix_web::Error> {
    let path: PathBuf = "../public/pages/add_announcement.html".parse().unwrap();
    let form = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(form))
}

pub async fn edit_announcement_form_handler(
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    match db::get_announcement(id.into_inner()) {
        Ok(announcement) => {
            let path: PathBuf = "../public/pages/edit_announcement.html".parse().unwrap();
            let mut form = tokio::fs::read_to_string(path).await?;
            form = form.replace("{{announcement.id}}", &announcement.id.to_string());
            form = form.replace("{{announcement.image}}", &announcement.image);
            form = form.replace("{{announcement.title}}", &announcement.title);
            form = form.replace("{{announcement.date}}", &announcement.date);
            form = form.replace("{{announcement.content}}", &announcement.content);
            form = form.replace("{{announcement.author}}", &announcement.author);
            Ok(HttpResponse::Ok().content_type("text/html").body(form))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn edit_announcement_handler(
    mut payload: Multipart,
    session: Session,
) -> Result<HttpResponse, Error> {
    let mut id: Option<i32> = None;
    let mut image: Option<Bytes> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut date: Option<String> = None;
    let mut author: Option<String> = None;
    let mut image_path: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().unwrap();
        let filename = content_disposition.get_filename().unwrap_or("unnamed");

        let extension = Path::new(filename)
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap();

        match name {
            "id" => {
                let data = field.next().await.unwrap().unwrap();
                match std::str::from_utf8(&data).unwrap().parse::<i32>() {
                    Ok(parsed_id) => id = Some(parsed_id),
                    Err(_) => {}
                }
            }
            "image" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                let image_data = bytes.freeze();

                image = Some(image_data);
                image_path = Some(format!(
                    "../public/assets/image/upload/{}.{}",
                    Uuid::new_v4().to_string(),
                    extension
                ));
                let mut file = fs::File::create(&image_path.as_ref().unwrap()).unwrap();
                file.write_all(&image.as_ref().unwrap()).unwrap(); // Use as_ref to avoid moving image
            }
            "title" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                title = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "content" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                content = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "author" => {
                match session.get::<String>("user_id") {
                    Ok(user_id_option) => {
                        if let Some(username) = user_id_option {
                            author = Some(username);
                        }
                    }
                    Err(_) => {
                        // Handle case where getting user from session failed
                    }
                }
            }
            "date" => {
                date = Some(chrono::Local::now().format("%d-%m-%Y").to_string());
            }
            _ => (),
        }
    }

    let id = id.unwrap();
    let title = title.unwrap();
    let content = content.unwrap();
    let date = date.unwrap();
    let author = author.unwrap();

    let image_path = image_path.unwrap_or_else(|| {
        if image.is_none() {
            db::get_announcement(id).unwrap().image
        } else {
            String::new()
        }
    });

    let db_image_path = image_path.replace("../public", "");

    if let Some(image) = image {
        let mut file = fs::File::create(&image_path).unwrap();
        file.write_all(&image).unwrap();
    }

    let form = EditAnnouncementForm {
        id,
        image_path: db_image_path,
        title,
        content,
        date,
        author,
    };

    match db::edit_announcement(
        id,
        &form.image_path,
        &form.title,
        &form.content,
        &form.date,
        &form.author,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn delete_announcement_handler(req: HttpRequest) -> Result<HttpResponse> {
    let id: i32 = req
        .match_info()
        .get("id")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    db::delete_announcement(id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(""))
}

pub async fn admin_articles_handler(Query(pagination): Query<Pagination>) -> Result<HttpResponse> {
    let page: i32 = pagination.page.unwrap_or(1).try_into().unwrap();
    let page_size: i32 = pagination.page_size.unwrap_or(3).try_into().unwrap();

    let (articles, total_articles) = db::get_articles(page, page_size)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let total_pages = (total_articles as f32 / page_size as f32).ceil() as i32;

    let mut content = String::from("
    <div class='flex justify-center'>
        <button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 mt-4 rounded' hx-get='/admin/articles/add/form' hx-swap='innerHTML' hx-target='#dashboard-container'>           Yeni Duyuru Ekle           </button>
    </div>
    ");
    for article in &articles {
        let mut article_content =
            tokio::fs::read_to_string("../public/pages/articles.html").await?;
        article_content = article_content.replace("{image}", &article.image);
        article_content = article_content.replace("{title}", &article.title);
        article_content = article_content.replace("{date}", &article.date);
        article_content = article_content.replace("{author}", &article.author);
        article_content = article_content.replace("{id}", &article.id.to_string());
        content.push_str(&article_content);
    }

    content.push_str("<div class='flex justify-center items-center mt-4 mb-4 space-x-2'>");
    if page > 1 {
        content.push_str(&format!(
            "<button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded' hx-get='/admin/articles?page={}&page_size={}' hx-swap='innerHTML' hx-target='#dashboard-container'>Önceki Sayfa</button>",
            page - 1,
            page_size
        ));
    }
    if page < total_pages {
        content.push_str(&format!(
            "<button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded' hx-get='/admin/articles?page={}&page_size={}' hx-swap='innerHTML' hx-target='#dashboard-container'>Sonraki Sayfa</button>",
            page + 1,
            page_size
        ));
    }
    content.push_str("</div>");

    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn add_article_handler(
    mut payload: Multipart,
    session: Session,
) -> Result<HttpResponse, Error> {
    let mut image: Option<Bytes> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut date: Option<String> = None;
    let mut author: Option<String> = None;
    let mut image_path: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().unwrap();
        let filename = content_disposition.get_filename().unwrap_or("unnamed");
        let extension = Path::new(filename)
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap();

        match name {
            "image" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                image = Some(bytes.freeze());
                image_path = Some(format!(
                    "../public/assets/image/upload/{}.{}",
                    Uuid::new_v4().to_string(),
                    extension
                ));
                let mut file = fs::File::create(&image_path.as_ref().unwrap()).unwrap();
                file.write_all(&image.as_ref().unwrap()).unwrap(); // Use as_ref to avoid moving image
            }
            "title" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                title = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "content" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                content = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "author" => {
                match session.get::<String>("user_id") {
                    Ok(user_id_option) => {
                        if let Some(username) = user_id_option {
                            author = Some(username);
                        }
                    }
                    Err(_) => {
                        // Handle case where getting user from session failed
                    }
                }
            }
            "date" => {
                date = Some(chrono::Local::now().format("%d-%m-%Y").to_string());
            }
            _ => (),
        }
    }

    let image = image.unwrap();
    let title = title.unwrap();
    let content = content.unwrap();
    let date = date.unwrap();
    let author = author.unwrap();

    let image_path = image_path.unwrap();

    let db_image_path = image_path.replace("../public", "");

    let mut file = fs::File::create(&image_path).unwrap();
    file.write_all(&image).unwrap();

    let form = AddArticleForm {
        title,
        content,
        date,
        author,
        image_path: db_image_path,
    };

    match db::add_article(
        &form.image_path,
        &form.title,
        &form.content,
        &form.date,
        &form.author,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn add_article_form_handler() -> Result<HttpResponse, actix_web::Error> {
    let path: PathBuf = "../public/pages/add_article.html".parse().unwrap();
    let form = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(form))
}

pub async fn edit_article_form_handler(
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    match db::get_article(id.into_inner()) {
        Ok(article) => {
            let path: PathBuf = "../public/pages/edit_article.html".parse().unwrap();
            let mut form = tokio::fs::read_to_string(path).await?;
            form = form.replace("{{article.id}}", &article.id.to_string());
            form = form.replace("{{article.image}}", &article.image);
            form = form.replace("{{article.title}}", &article.title);
            form = form.replace("{{article.date}}", &article.date);
            form = form.replace("{{article.content}}", &article.content);
            form = form.replace("{{article.author}}", &article.author);
            Ok(HttpResponse::Ok().content_type("text/html").body(form))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn edit_article_handler(
    mut payload: Multipart,
    session: Session,
) -> Result<HttpResponse, Error> {
    let mut id: Option<i32> = None;
    let mut image: Option<Bytes> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut date: Option<String> = None;
    let mut author: Option<String> = None;
    let mut image_path: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().unwrap();
        let filename = content_disposition.get_filename().unwrap_or("unnamed");

        let extension = Path::new(filename)
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap();

        match name {
            "id" => {
                let data = field.next().await.unwrap().unwrap();
                match std::str::from_utf8(&data).unwrap().parse::<i32>() {
                    Ok(parsed_id) => id = Some(parsed_id),
                    Err(_) => {}
                }
            }
            "image" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                let image_data = bytes.freeze();

                image = Some(image_data);
                image_path = Some(format!(
                    "../public/assets/image/upload/{}.{}",
                    Uuid::new_v4().to_string(),
                    extension
                ));
                let mut file = fs::File::create(&image_path.as_ref().unwrap()).unwrap();
                file.write_all(&image.as_ref().unwrap()).unwrap(); // Use as_ref to avoid moving image
            }
            "title" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                title = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "content" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                content = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "author" => {
                match session.get::<String>("user_id") {
                    Ok(user_id_option) => {
                        if let Some(username) = user_id_option {
                            author = Some(username);
                        }
                    }
                    Err(_) => {
                        // Handle case where getting user from session failed
                    }
                }
            }
            "date" => {
                date = Some(chrono::Local::now().format("%d-%m-%Y").to_string());
            }
            _ => (),
        }
    }

    let id = id.unwrap();
    let title = title.unwrap();
    let content = content.unwrap();
    let date = date.unwrap();
    let author = author.unwrap();

    let image_path = image_path.unwrap_or_else(|| {
        if image.is_none() {
            db::get_article(id).unwrap().image
        } else {
            String::new()
        }
    });

    let db_image_path = image_path.replace("../public", "");

    if let Some(image) = image {
        let mut file = fs::File::create(&image_path).unwrap();
        file.write_all(&image).unwrap();
    }

    let form = EditArticleForm {
        id,
        image_path: db_image_path,
        title,
        content,
        date,
        author,
    };

    match db::edit_article(
        id,
        &form.image_path,
        &form.title,
        &form.content,
        &form.date,
        &form.author,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn delete_article_handler(req: HttpRequest) -> Result<HttpResponse> {
    let id: i32 = req
        .match_info()
        .get("id")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    db::delete_article(id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(""))
}

pub async fn admin_user_handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/users.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn add_user_handler(form: web::Form<User>) -> Result<HttpResponse, actix_web::Error> {
    let user = form.into_inner();

    match db::add_user(&user.name, &user.username, &user.password) {
        Ok(_) => {
            let users = db::get_users().unwrap();
            let user_list_html = render_user_list(&users).await.unwrap();

            let mut response = HttpResponse::Ok();
            response.header("HX-Trigger", "refreshUserList");
            Ok(response.body(user_list_html))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn add_user_form_handler() -> Result<HttpResponse, actix_web::Error> {
    let path: PathBuf = "../public/pages/add_user.html".parse().unwrap();
    let form = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(form))
}

pub async fn edit_user_form_handler(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let username: String = req.match_info().query("username").parse().unwrap();
    let user = db::get_user(&username).unwrap();

    let path: PathBuf = "../public/pages/edit_user.html".parse().unwrap();
    let mut form = tokio::fs::read_to_string(path).await?;

    form = form.replace("{{name}}", &user.0);
    form = form.replace("{{username}}", &user.1);

    Ok(HttpResponse::Ok().content_type("text/html").body(form))
}

pub async fn edit_user_handler(
    req: HttpRequest,
    form: web::Form<User>,
) -> Result<HttpResponse, actix_web::Error> {
    let username: String = req.match_info().query("username").parse().unwrap();
    let user = form.into_inner();

    match db::edit_user(&username, &user.name, &user.username, &user.password) {
        Ok(_) => {
            let users = db::get_users().unwrap();
            let user_list_html = render_user_list(&users).await.unwrap();

            let mut response = HttpResponse::Ok();
            response.header("HX-Trigger", "refreshUserList");
            Ok(response.body(user_list_html))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn delete_user_handler(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let username: String = req.match_info().query("username").parse().unwrap();

    match db::delete_user(&username) {
        Ok(_) => {
            let users = db::get_users().unwrap();

            let user_list_html = render_user_list(&users).await.unwrap();

            let mut response = HttpResponse::Ok();
            response.header("HX-Trigger", "refreshUserList");
            Ok(response.body(user_list_html))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn render_user_list(users: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let path: PathBuf = "../public/pages/user_list.html".parse().unwrap();
    let mut template = tokio::fs::read_to_string(path).await?;

    let user_rows = users
        .iter()
        .map(|user| {
            let parts: Vec<&str> = user.split(" (").collect();
            let name = parts[0];
            let username = parts[1].trim_end_matches(')');
            format!(
                "<tr class=\"border\">\n
                <td class=\"border px-6 py-4 whitespace-nowrap\">{}</td>\n
                <td class=\"border px-6 py-4 whitespace-nowrap\">{}</td>\n
                <td class=\"border px-6 py-4 whitespace-nowrap\">
                <button hx-get=\"/admin/user/edit/form/{}\" hx-target=\"#modal-content .space-y-4\" hx-trigger=\"click\" class=\"px-4 py-2 text-white bg-blue-500 rounded\">Edit</button>
                <button class=\"px-4 py-2 text-white bg-red-500 rounded\"
                hx-delete=\"/admin/user/delete/{}\" hx-swap=\"innerHTML\" hx-target=\"#user-list\"
                hx-confirm=\"Are you sure you want to delete this user?\">Delete</button>
                </td>\n
                </tr>\n",
                name, username, username, username
            )
        })
        .collect::<Vec<String>>()
        .join("");

    template = template.replace("{{users}}", &user_rows);

    Ok(template)
}

pub async fn get_user_list_handler() -> Result<HttpResponse, actix_web::Error> {
    let users =
        db::get_users().map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let rendered = render_user_list(&users)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub async fn admin_inbox_handler() -> Result<HttpResponse, actix_web::Error> {
    let path: PathBuf = "../public/pages/messages.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn get_messages_handler() -> Result<HttpResponse, actix_web::Error> {
    let messages = db::get_messages()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let message_rows = messages
        .iter()
        .map(|message| {
            format!(
                "<tr class=\"bg-white border-b dark:bg-gray-800 dark:border-gray-700\">
                    <th scope=\"row\" class=\"px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white\">{}</th>
                    <td class=\"px-6 py-4\">{}</td>
                    <td class=\"px-6 py-4\">{}</td>
                    <td class=\"px-6 py-4\">{}</td>
                </tr>",
                message.0, message.1, message.2, message.3
            )
        })
        .collect::<Vec<String>>()
        .join("");

    let table = format!(
        "
        <div class=\"w-1/2 mx-auto mt-10 justify-center items-center text-center\">
            <div class=\"relative overflow-x-auto\">
                <table class=\"w-full text-sm text-center rtl:text-right text-gray-500 dark:text-gray-400\">
                    <thead class=\"text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400\">
                        <tr>
                            <th scope=\"col\" class=\"px-6 py-3\">İsim</th>
                            <th scope=\"col\" class=\"px-6 py-3\">Email</th>
                            <th scope=\"col\" class=\"px-6 py-3\">Mesaj</th>
                            <th scope=\"col\" class=\"px-6 py-3\">Ip</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
        </div>",
        message_rows
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(table))
}

pub async fn admin_gallery_handler() -> Result<HttpResponse, actix_web::Error> {
    let path: PathBuf = "../public/pages/gallery.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

async fn get_image_files() -> Result<Vec<String>, std::io::Error> {
    let image_folder = "../public/assets/slider";
    let mut entries = fs::read_dir(image_folder)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    entries.sort_by_key(|path| {
        fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let image_files: Vec<String> = entries
        .iter()
        .filter(|path| path.is_file())
        .filter_map(|path| path.file_name())
        .filter_map(|name| name.to_str().map(String::from))
        .collect();

    Ok(image_files)
}

fn paginate(image_files: Vec<String>, pagination: Pagination) -> Vec<String> {
    let page: usize = pagination.page.unwrap_or(1);
    let page_size: usize = pagination.page_size.unwrap_or(6);

    let start = (page - 1) * page_size;
    let end = std::cmp::min(start + page_size, image_files.len());

    image_files.get(start..end).unwrap_or(&[]).to_vec()
}

pub async fn admin_image_handler(
    web::Query(pagination): web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let image_files = get_image_files().await?;
    let paginated_images = paginate(image_files, pagination);

    Ok(HttpResponse::Ok().json(paginated_images))
}

pub async fn delete_image_handler(
    info: web::Path<(String,)>,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let image_name = &info.0;
    let image_path = format!("../public/assets/slider/{}", image_name);

    match fs::remove_file(&image_path) {
        Ok(_) => {
            let image_files = get_image_files().await?;
            let paginated_images = paginate(image_files, pagination.clone());

            Ok(HttpResponse::Ok().json(paginated_images))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn count_images_handler() -> Result<HttpResponse, Error> {
    let image_files = get_image_files().await?;

    let total_images = image_files.len();

    Ok(HttpResponse::Ok().json(total_images))
}

pub async fn admin_upload_handler(
    mut payload: Multipart,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut bytes = BytesMut::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            bytes.extend_from_slice(&data);
        }

        let image_data = bytes.freeze();
        let img = image::load_from_memory(&image_data).unwrap();
        let resized = img.resize_exact(1280, 720, image::imageops::FilterType::Lanczos3);
        let image_path = format!("../public/assets/slider/{}.webp", Uuid::new_v4().to_string());
        resized.save_with_format(&image_path, image::ImageFormat::WebP).unwrap();
    }

    let image_files = get_image_files().await?;
    let paginated_images = paginate(image_files, pagination.clone());

    Ok(HttpResponse::Ok().json(paginated_images))
}

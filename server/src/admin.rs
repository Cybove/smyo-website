use crate::src::db;
use actix_multipart::{Field, Multipart};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::header::ContentDisposition;
use actix_web::web::BytesMut;
use actix_web::web::{self, Bytes};
use actix_web::{Error, HttpRequest, HttpResponse, Responder, Result};
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use serde_derive::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginForm {
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

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/admin.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn login_handler(form: web::Form<LoginForm>, mut session: Session) -> impl Responder {
    let login_form = form.into_inner();

    if db::authenticate_user(&login_form.username, &login_form.password).unwrap() {
        match session.insert("user_id", login_form.username) {
            Ok(_) => (),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to set session: {}", e))
            }
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

pub async fn admin_user_handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/user.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn admin_announcements_handler(req: HttpRequest) -> Result<HttpResponse> {
    let page: i32 = req
        .match_info()
        .get("page")
        .unwrap_or("1")
        .parse()
        .unwrap_or(1);
    let page_size: i32 = 3;
    let announcements = db::get_announcements(page, page_size)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let mut content = String::from("<button class='bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 mx-auto mt-4 rounded items-center justify-center' hx-get='/admin/announcements/add/form' hx-swap='innerHTML' hx-target='#dashboard-container'>           Yeni Duyuru Ekle           </button>");

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

    let previous_page = if page > 1 { page - 1 } else { 1 };
    let next_page = page + 1;

    let pagination = format!("<div class='flex'>
    <a hx-get='/admin/announcements?page={previous_page}' hx-trigger='click' hx-target='#dashboard-container' class='flex items-center justify-center px-4 h-10 text-base font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'>Previous</a>
    <a hx-get='/admin/announcements?page={next_page}' hx-trigger='click' hx-target='#dashboard-container' class='flex items-center justify-center px-4 h-10 ms-3 text-base font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'>Next</a>
    </div>");

    content.push_str(&pagination);

    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

pub async fn add_announcement_handler(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut image: Option<Bytes> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut date: Option<String> = None;
    let mut author: Option<String> = None;
    let mut image_path: Option<String> = None;

    // iterate over multipart stream
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
            "date" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                date = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "author" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                author = Some(String::from_utf8(bytes.to_vec()).unwrap());
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

pub async fn edit_announcement_handler(mut payload: Multipart) -> Result<HttpResponse, Error> {
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
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                author = Some(String::from_utf8(bytes.to_vec()).unwrap());
            }
            "date" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                date = Some(String::from_utf8(bytes.to_vec()).unwrap());
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

pub async fn delete_announcement_handler(
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    match db::delete_announcement(id.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

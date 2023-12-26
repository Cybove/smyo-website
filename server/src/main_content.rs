use crate::src::db;
use actix_web::{web::Json, HttpRequest, HttpResponse, Responder, Result};
use serde::Serialize;
use serde_derive::Serialize;
use std::path::PathBuf;

#[derive(Clone, Serialize)]
pub struct Announcement {
    pub id: i32,
    pub image: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

pub async fn announcements_handler(_req: HttpRequest) -> Result<HttpResponse> {
    let announcements = db::get_announcements()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let mut response = String::new();
    for announcement in announcements {
        response.push_str(&format!(
            "<div class='mb-6 lg:mb-0'>
                <div class='relative block rounded-lg bg-white shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] dark:bg-neutral-700'>
                    <div class='flex justify-center'>
                        <div class='relative mx-4 -mt-4 overflow-hidden rounded-lg bg-cover bg-no-repeat shadow-lg dark:shadow-black/20' data-te-ripple-init data-te-ripple-color='light'>
                            <img src='{}' class='object-contain h-64 w-full max-w-md mx-auto' />
                            <div class='absolute top-0 right-0 bottom-0 left-0 h-full w-full overflow-hidden bg-fixed opacity-0 transition duration-300 ease-in-out hover:opacity-100 bg-[hsla(0,0%,98.4%,.15)]'></div>
                        </div>
                    </div>
                    <div class='p-6'>
                        <h5 class='mb-3 text-lg font-bold'>{}</h5>
                        <p class='mb-4 text-neutral-500 dark:text-neutral-300'>
                            <small>Published <u>{}</u> by
                                <a>{}</a></small>
                        </p>
                        <button hx-get='/announcement/{}' hx-target='#main-container' class='inline-flex items-center justify-center px-4 py-2 text-base font-medium text-white bg-blue-600 border border-transparent rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500'>
                            Oku
                        </button>
                        </div>
                </div>
            </div>",
            announcement.image, announcement.title, announcement.date, announcement.author, announcement.id
        ));
    }

    Ok(HttpResponse::Ok().content_type("text/html").body(response))
}

pub async fn announcement_detail_handler(req: HttpRequest) -> Result<HttpResponse> {
    let id: i32 = req.match_info().get("id").unwrap().parse().unwrap();
    let announcement = db::get_announcement(id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let html = format!(
        
        "
        <a href=\"#duyurular\" hx-get=\"/duyurular\" hx-target=\"#main-container\" hx-push-url=\"#duyurular\" hx-trigger=\"click\"
        class=\"py-4 px-5 text-gray-900 rounded max-w-xs md:bg-transparent flex items-center\" aria-current=\"page\">
        <svg class=\"w-6 h-6 text-gray-800 dark:text-white\" aria-hidden=\"true\" xmlns=\"http://www.w3.org/2000/svg\"
            fill=\"none\" viewBox=\"0 0 8 14\">
            <path stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\"
                d=\"M7 1 1.3 6.326a.91.91 0 0 0 0 1.348L7 13\" />
        </svg>
        </a>
        <div class=\"max-w-screen-lg mx-auto p-5 sm:p-10 md:p-16 items-center text-center justify-center\">

            <div class=\"mb-10 rounded overflow-hidden flex flex-col mx-auto\">
                <a
                    class=\"text-3xl sm:text-4xl font-semibold inline-block hover:text-indigo-600 transition duration-500 ease-in-out mb-2\">{}
                </a>

                <div class=\"relative\">
                    <img class=\"w-auto h-auto mx-auto rounded-lg shadow-lg object-cover object-center max-w-4xl\"
                        src=\"{}\">
                </div>

                <p class=\"text-gray-900 py-5 text-2xl leading-8 flex-col\">
                    {}
                </p>

                <div class=\"py-5 text-sm font-regular text-gray-900 flex\">
                    <span class=\"mr-3 flex flex-row items-center\">
                        <svg class=\"text-indigo-600\" fill=\"currentColor\" height=\"13px\" width=\"13px\" version=\"1.1\" id=\"Layer_1\"
                            xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" x=\"0px\" y=\"0px\"
                            viewBox=\"0 0 512 512\" style=\"enable-background:new 0 0 512 512;\" xml:space=\"preserve\">
                            <g>
                                <g>
                                    <path d=\"M256,0C114.837,0,0,114.837,0,256s114.837,256,256,256s256-114.837,256-256S397.163,0,256,0z M277.333,256
                        c0,11.797-9.536,21.333-21.333,21.333h-85.333c-11.797,0-21.333-9.536-21.333-21.333s9.536-21.333,21.333-21.333h64v-128
                        c0-11.797,9.536-21.333,21.333-21.333s21.333,9.536,21.333,21.333V256z\"></path>
                                </g>
                            </g>
                        </svg>
                        <span class=\"ml-1\">{}</span></span>
                    <a class=\"flex flex-row items-center hover:text-indigo-600\">
                        <svg class=\"text-indigo-600\" fill=\"currentColor\" height=\"16px\" aria-hidden=\"true\" role=\"img\"
                            focusable=\"false\" viewBox=\"0 0 24 24\" xmlns=\"http://www.w3.org/2000/svg\">
                            <path fill=\"currentColor\"
                                d=\"M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z\">
                            </path>
                            <path d=\"M0 0h24v24H0z\" fill=\"none\"></path>
                        </svg>
                        <span class=\"ml-1\">{}</span></a>
                </div>
                <hr>

            </div>

        </div>",
        
        announcement.title, announcement.image, announcement.content, announcement.date, announcement.author
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/main.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

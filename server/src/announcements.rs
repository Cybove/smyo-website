use crate::src::db;
use std::path::PathBuf;
use actix_web::{web::Json, web::Query ,HttpRequest, HttpResponse, Responder, Result};
use serde::Serialize;
use serde_derive::Deserialize;

pub async fn handler(_req: HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = "../public/pages/duyurular.html".parse().unwrap();
    let content = tokio::fs::read_to_string(path).await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[derive(Clone, Serialize)]
pub struct Announcement {
    pub id: i32,
    pub image: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    page: Option<i32>,
    page_size: Option<i32>,
}

pub async fn announcements_handler(
    req: HttpRequest,
) -> Result<HttpResponse> {
    let page: i32 = req.match_info().get("page").unwrap_or("1").parse().unwrap_or(1);
    let from_main_page: bool = req.query_string().contains("main_page=true");
    let announcements_per_page = if from_main_page { 3 } else { 6 };
    let (announcements, total_announcements) = db::get_announcements(page, announcements_per_page)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let mut response = String::new();
    response.push_str("<div class='grid gap-6 lg:grid-cols-3 xl:gap-x-12'>");
    for announcement in announcements {
        response.push_str(&format!(
            "<div class='mb-6 lg:mb-0'>
            <div
                class='relative block rounded-lg bg-white shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] dark:bg-neutral-700'>
                <div class='flex justify-center'>
                    <div class='relative mx-4 -mt-4 overflow-hidden rounded-lg bg-cover bg-no-repeat shadow-lg dark:shadow-black/20'
                        data-te-ripple-init data-te-ripple-color='light'>
                        <img src='{}' class='rounded-xl object-fit h-80 w-[450px] mx-auto xs:max-w-xs' />                        <div
                            class='absolute top-0 right-0 bottom-0 left-0 h-full w-full overflow-hidden bg-fixed opacity-0 transition duration-300 ease-in-out hover:opacity-100 bg-[hsla(0,0%,98.4%,.15)]'>
                        </div>
                    </div>
                </div>
                <div class='p-6'>
                    <h5 class='mb-3 text-lg font-bold'>{}</h5>
                    <p class='mb-4 text-neutral-500 dark:text-neutral-300'>
                        <small><u>{}</u><br /><a>{}</a></small>
                    </p>
                    <button hx-get='/announcement/{}' hx-target='#main-container' hx-push-url='#duyuru'
                        class='inline-flex items-center justify-center px-4 py-2 text-base font-medium text-white bg-blue-600 border border-transparent rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500'>Oku</button>
                </div>
            </div>
        </div>",
            announcement.image, announcement.title, announcement.date, announcement.author, announcement.id
        ));
    }

    response.push_str("</div>");

    if from_main_page {
        response.push_str(&format!(
            "<a id='link-duyurular' href='#duyurular' hx-get='/duyurular'
            hx-target='#main-container' hx-trigger='click'
            class='inline-flex items-center justify-center px-4 mt-6 py-2 text-base font-medium text-white bg-green-600 border border-transparent rounded-md shadow-sm hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500'>Tüm Duyurular</a>"
        ));
    } 
    
    else {
        response.push_str("<div class='flex justify-center mt-8'>");

        if page > 1 {
            let prev_page = page - 1;
            response.push_str(&format!(
                "<button class='flex items-center justify-center px-3 h-8 ms-3 text-lg font-bold text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-300 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white' hx-get='/announcements/{}' hx-boost='true' hx-target='#announcement-container'>
                <svg class='w-5 h-5 me-2 rtl:rotate-180' aria-hidden='true' xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 14 10'>
                <path stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M13 5H1m0 0 4 4M1 5l4-4'/>
                </svg>
                Önceki Sayfa
                </button>",
                prev_page
            ));
        }

        if page * 6 < total_announcements {
            let next_page = page + 1;
            response.push_str(&format!(
                "<button class='flex items-center justify-center px-3 h-8 ms-3 text-lg font-bold text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-300 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white' hx-get='/announcements/{}' hx-boost='true' hx-target='#announcement-container'>
                Sonraki Sayfa
                <svg class='w-5 h-5 ms-2 rtl:rotate-180' aria-hidden='true' xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 14 10'>
                <path stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M1 5h12m0 0L9 1m4 4L9 9'/>
                </svg>
                </button>",
                next_page
            ));
        }

        response.push_str("</div>");
    }
    
    response.push_str("</div>");
    Ok(HttpResponse::Ok().content_type("text/html").body(response))
}

pub async fn announcement_detail_handler(req: HttpRequest) -> Result<HttpResponse> {
    let id: i32 = req.match_info().get("id").unwrap().parse().unwrap();
    let announcement = db::get_announcement(id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let html = format!(
        
        "
        <a href='#duyurular' hx-get='/duyurular' hx-target='#main-container' hx-trigger='click'
        class='py-4 px-5 text-gray-900 rounded max-w-xs md:bg-transparent flex items-center' aria-current='page'>
        <svg class='w-6 h-6 text-gray-800 dark:text-white' aria-hidden='true' xmlns='http://www.w3.org/2000/svg'
            fill='none' viewBox='0 0 8 14'>
            <path stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' stroke-width='2'
                d='M7 1 1.3 6.326a.91.91 0 0 0 0 1.348L7 13' />
        </svg>
        </a>
        <div class='max-w-screen-lg mx-auto p-5 sm:p-10 md:p-16 items-center text-center justify-center'>

            <div class='mb-10 rounded overflow-hidden flex flex-col mx-auto'>
                <a
                    class='text-3xl sm:text-4xl font-semibold inline-block hover:text-indigo-600 transition duration-500 ease-in-out mb-2'>{}
                </a>

                <div class='relative mb-10'>
                    <img class='w-auto h-[450px] mx-auto rounded-lg shadow-lg object-fit object-center max-w-4xl'
                        src='{}'>
                </div>

                <div class='mb-10 w-full md:w-1/2 overflow-auto p-4 mx-auto bg-gray-300 border-2 border-gray-300 rounded-lg shadow-lg jodit-wysiwyg'>
                    <div class='text-black pb-8 text-2xl leading-8'>
                        {}
                    </div>
                </div>

                <div class='py-5 text-sm font-regular text-gray-900 flex'>
                    <span class='mr-3 flex flex-row items-center'>
                        <svg class='text-indigo-600' fill='currentColor' height='13px' width='13px' version='1.1' id='Layer_1'
                            xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' x='0px' y='0px'
                            viewBox='0 0 512 512' style='enable-background:new 0 0 512 512;' xml:space='preserve'>
                            <g>
                                <g>
                                    <path d='M256,0C114.837,0,0,114.837,0,256s114.837,256,256,256s256-114.837,256-256S397.163,0,256,0z M277.333,256
                        c0,11.797-9.536,21.333-21.333,21.333h-85.333c-11.797,0-21.333-9.536-21.333-21.333s9.536-21.333,21.333-21.333h64v-128
                        c0-11.797,9.536-21.333,21.333-21.333s21.333,9.536,21.333,21.333V256z'></path>
                                </g>
                            </g>
                        </svg>
                        <span class='ml-1'>{}</span></span>
                    <a class='flex flex-row items-center hover:text-indigo-600'>
                        <svg class='text-indigo-600' fill='currentColor' height='16px' aria-hidden='true' role='img'
                            focusable='false' viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
                            <path fill='currentColor'
                                d='M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z'>
                            </path>
                            <path d='M0 0h24v24H0z' fill='none'></path>
                        </svg>
                        <span class='ml-1'>{}</span></a>
                </div>
                <hr>
            </div>
        </div>",
        
        announcement.title, announcement.image, announcement.content, announcement.date, announcement.author
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
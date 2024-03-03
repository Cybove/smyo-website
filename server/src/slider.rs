use actix_web::web::Query;
use actix_web::{web::Json, HttpRequest, HttpResponse, Responder, Result};
use rand::prelude::SliceRandom;
use serde::Serialize;
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct SliderResponse {
    paths: Vec<String>,
}

pub async fn handler(req: HttpRequest) -> impl Responder {
    let path: PathBuf = "../public/assets/slider".parse().unwrap();
    let mut paths: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path().display().to_string())
        .collect();
    paths.sort();
    paths.shuffle(&mut rand::thread_rng());
    let paths: Vec<String> = paths.into_iter().take(10).collect();

    let slides: String = paths
        .into_iter()
        .map(|path| {
            format!(
                r#"
                <div class="swiper-slide">
                    <div class="flex items-center justify-center h-full w-full">
                        <img class="w-full h-full rounded-xl" src="{}" />
                    </div>
                </div>
                "#,
                path
            )
        })
        .collect();

    HttpResponse::Ok().body(slides)
}

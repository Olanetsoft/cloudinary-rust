use actix_web::{App, HttpServer};
use handler::upload_video;
mod handler;
mod models;
mod video_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(upload_video))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
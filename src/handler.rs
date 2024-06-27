use actix_multipart::Multipart;
use actix_web::{post, Error, HttpResponse};
use reqwest::StatusCode;
use crate::{
    models::{APIErrorResponse, APIResponse, CloudinaryResponse},
    video_service::VideoService,
};

#[post("/upload")]
pub async fn upload_video(multipart: Multipart) -> Result<HttpResponse, Error> {
    let file_path = VideoService::save_file(multipart).await?;
    let upload_details = VideoService::upload_to_cloudinary(&file_path).await;

    match upload_details {
        Ok(data) => Ok(HttpResponse::Created().json(APIResponse::<CloudinaryResponse> {
            status: StatusCode::CREATED.as_u16(),
            message: "success".to_string(),
            data: Some(data),
        })),
        Err(error) => Ok(HttpResponse::InternalServerError().json(APIErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "failure".to_string(),
            data: Some(error.to_string()),
        })),
    }
}
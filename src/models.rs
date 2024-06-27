// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudinaryResponse {
    pub public_id: String,
    pub secure_url: String,
}

#[derive(Serialize)]
pub struct APIResponse<T> {
    pub status: u16,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct APIErrorResponse {
    pub status: u16,
    pub message: String,
    pub data: Option<String>,
}

use crate::models::CloudinaryResponse;
use actix_multipart::Multipart;
use actix_web::Error;
use dotenv::dotenv;
use futures_util::StreamExt;
use reqwest::{
    multipart::{self, Part},
    Client,
};
use sha1::{Digest, Sha1};
use std::{collections::HashMap, env, io::Write};
use tempfile::NamedTempFile;
use tokio::io::AsyncReadExt;

const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB

enum ParamValue {
    Str(String),
    Int(i64),
}

pub struct VideoService;

impl VideoService {
    fn env_loader(key: &str) -> String {
        dotenv().ok();
        match env::var(key) {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        }
    }

    fn generate_signature(params: HashMap<&str, ParamValue>, api_secret: &str) -> String {
        // Step 1: Sort the parameters by keys and concatenate them
        let mut sorted_keys: Vec<&&str> = params.keys().collect();
        sorted_keys.sort();
        let mut sorted_params = String::new();
        for key in sorted_keys {
            if !sorted_params.is_empty() {
                sorted_params.push('&');
            }
            let value = match &params[key] {
                ParamValue::Str(s) => s.clone(),
                ParamValue::Int(i) => i.to_string(),
            };
            sorted_params.push_str(&format!("{}={}", key, value));
        }

        // Step 2: Concatenate the sorted parameters and the API secret
        let string_to_sign = format!("{}{}", sorted_params, api_secret);

        // Step 3: Generate an SHA-1 hash of the concatenated string
        let mut hasher = Sha1::new();
        hasher.update(string_to_sign.as_bytes());

        // Step 4: Return the hex-encoded result
        hex::encode(hasher.finalize())
    }


    pub async fn save_file(mut payload: Multipart) -> Result<NamedTempFile, Error> {
        let mut total_size = 0;

        // Create a temporary file
        let mut temp_file = NamedTempFile::new()?;

        // Iterate over multipart stream
        while let Some(field) = payload.next().await {
            let mut field = field?;

            // Get the MIME type of the file
            let content_type = field.content_type();

            // Ensure content_type is present and it is a video
            if let Some(content_type) = content_type {
                if content_type.type_() != mime::VIDEO {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Only video files are allowed",
                    ));
                }
            } else {
                return Err(actix_web::error::ErrorBadRequest("Missing content type"));
            }

            // Write the file content to the temporary file synchronously
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                total_size += data.len();
                if total_size > MAX_SIZE {
                    return Err(actix_web::error::ErrorBadRequest(
                        "File size limit exceeded",
                    ));
                }
                temp_file.write_all(&data)?;
            }
        }
        Ok(temp_file)
    }


    pub async fn upload_to_cloudinary(
        temp_file: &NamedTempFile,
    ) -> Result<CloudinaryResponse, Box<dyn std::error::Error>> {

        let client = Client::new();
        let cloud_name = VideoService::env_loader("CLOUDINARY_CLOUD_NAME");
        let api_secret = VideoService::env_loader("CLOUDINARY_API_SECRET");
        let api_key = VideoService::env_loader("CLOUDINARY_API_KEY");
        let timestamp = chrono::Utc::now().timestamp();

        let public_id = temp_file
            .path()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
            .to_string();
        
        // Include only public_id and timestamp in the signature
        let mut params = HashMap::new();
        params.insert("public_id", ParamValue::Str(public_id.to_string()));
        params.insert("timestamp", ParamValue::Int(timestamp));

        let signature = VideoService::generate_signature(params, &api_secret);

        let mut file = tokio::fs::File::open(temp_file.path()).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let part = Part::bytes(buffer).file_name(public_id.clone());

        let form = multipart::Form::new()
            .text("public_id", public_id.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .text("api_key", api_key)
            .part("file", part);

        let res = client
            .post(format!(
                "https://api.cloudinary.com/v1_1/{}/video/upload",
                cloud_name
            ))
            .multipart(form)
            .send()
            .await?;

        let result = res.text().await?;

        let cloudinary_response: CloudinaryResponse = serde_json::from_str(&result)?;

        Ok(cloudinary_response)
    }

}
use std::env;

use anyhow::Result;
use axum::{
    body::{Body, Bytes},
    extract::{Multipart, Path},
    response::Response,
};
use tower_sessions::Session;

use crate::{
    routes::{AppError, BodyBuilder, ResponseBody},
    services::{session_manager::SessionManagerService, SessionManager},
};

pub async fn write(
    Path(file_key): Path<String>,
    session: Session,
    mut multipart: Multipart,
) -> Result<Response<Body>, AppError> {
    // check file_key is available
    let is_available = SessionManager.is_available_key(&session, &file_key).await?;

    // check file_key is available
    if !is_available {
        let body = ResponseBody::new("file_key is not available".to_string()).build_body();

        let response = Response::builder().status(400).body(body).unwrap();

        return Ok(response);
    }

    // load chunkCount and fileData from multipart
    let mut chunk_count: u32 = 0;
    let mut file_data: Bytes = Bytes::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "chunkCount" => {
                chunk_count = field.text().await?.parse::<u32>()?;
            }
            "fileData" => {
                file_data = field.bytes().await?;
            }
            _ => {}
        }
    }

    // create folder if not exist
    let storage_path = env::var("BASE_STORAGE_PATH").unwrap_or("./storage".to_string());
    let dir_path = format!("{}/{}", &storage_path, &file_key);
    tokio::fs::create_dir_all(&dir_path).await?;
    // write fileData to file
    let file_path = format!("{}/{}", &dir_path, chunk_count);
    tokio::fs::write(file_path, file_data).await?;

    let body = ResponseBody::new("chunk upload success".to_string()).build_body();

    let response = Response::builder().status(200).body(body).unwrap();

    Ok(response)
}

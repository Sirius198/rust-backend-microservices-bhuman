use axum::{
    body::{Bytes, StreamBody},
    extract::{ContentLengthLimit, Extension, Json, Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    BoxError,
};
use axum_macros::debug_handler;
use futures::{Stream, TryStreamExt};
use sqlx::postgres::PgPool;
use std::io;
// use std::path::Path;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::{ReaderStream, StreamReader};

use microservice_utils::{
    jwt::extractor::AuthToken,
    server::response::{into_response, AxumRes, AxumResult},
};

use rusoto_core::Region;
use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
// use std::fs::File;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::get_file_info;
use crate::types::{AppState, FileUploadingState};

const UPLOADS_DIRECTORY: &str = "uploads";

#[derive(Serialize, Deserialize, Debug)]
// #[allow(dead_code)]
pub struct CreateFileReq {
    pub user_id: String,
    pub file_name: String,
    pub file_size: u128,
}

pub async fn create_new_file_on_db(
    json: Json<CreateFileReq>,
    Extension(pool): Extension<PgPool>,
    Path(pid): Path<String>,
    AuthToken(user_id): AuthToken,
) -> String {
    let file_size = json.file_size;
    let file_name = json.file_name.clone();
    // let user_id = json.user_id;
    let pid = pid.parse().unwrap();

    let res = create_new_file_record(&pool, user_id, file_name, file_size, pid).await;

    if let Ok((file_id, _)) = res {
        return format!("{}", file_id);
    } else if let Err(e) = res {
        println!("sqlx err! {}", e);
    }

    return String::from("0");
}

pub async fn create_new_file_record(
    pool: &PgPool,
    user_id: String,
    file_name: String,
    file_size: u128,
    pid: i32,
) -> Result<(i32, String), sqlx::Error> {
    let path = format!("{}/{}", UPLOADS_DIRECTORY, Uuid::new_v4()); // Generate path from uuid
    let query = format!(
        "INSERT INTO files(pid, user_id, name, path, size) VALUES({}, {}, \'{}\', \'{}\', {}) returning id, path;",
        pid, user_id, file_name, path, file_size
    );

    let row: (i32, String) = sqlx::query_as(&query).fetch_one(pool).await?;

    return Ok((row.0, row.1));
}

/**
 * Accept a file and save it in local storage
 * After the finishing the save, it uploads to S3 Bucket
 * Max file size: 5GB
 */

pub async fn accept_file(
    Path(file_id): Path<String>,
    AuthToken(user_id): AuthToken,
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { 5 * 1024 * 1024 * 1024 }>,
    Extension(state): Extension<Arc<AppState>>,
    Extension(pool): Extension<Arc<PgPool>>,
) {
    let file_path: String;
    if let Ok(file) = get_file_info(file_id, &pool).await {
        file_path = file.path.clone();

        let mut state_list = state.state_list.lock().unwrap();
        let state = FileUploadingState {
            file_id: file.id as u32,
            file_name: file.name,
            file_path: file.path.clone(),
            user_id: user_id,
            total_size: file.size as u32,
            uploaded_size: 0,
        };
        state_list.insert(file.id as u32, state);
    } else {
        return;
    }

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "attach" {
            let _ = stream_to_file(&file_path, field).await;
        }
    }

    // Upload to AWS S3 Bucket
    if let Err(_) = upload_to_bucket(file_path).await {
        println!("S3 upload failed!");
    }
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(File::create(path).await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;
    println!("Successfully saved here:");

    Ok(())
}

pub async fn upload_to_bucket(file_path: String) -> tokio::io::Result<()> {
    EnvironmentProvider::default().credentials().await.unwrap();
    let s3 = S3Client::new(Region::UsEast1);
    let bucket_name: String =
        std::env::var("S3_BUCKET").unwrap_or("henry-bhuman-bucket".to_string());

    // generate unique key for video resource
    let id = "test.file";

    // let path = std::path::Path::new(UPLOADS_DIRECTORY).join("path");

    println!("s3 file url: {}", file_path.clone());
    let mut tokio_file = std::fs::File::open(file_path.clone())?;
    let mut buffer: Vec<u8> = Vec::new();
    let _ = tokio_file.read_to_end(&mut buffer)?;

    println!("started pushing to s3");

    let result = s3
        .put_object(PutObjectRequest {
            key: file_path.clone(),
            content_type: Some("audio/wav".to_string()),
            content_disposition: Some(format!("inline; filename={}", id)),
            content_length: Some(buffer.len() as i64),
            body: Some(buffer.into()),
            bucket: bucket_name.clone(),
            acl: Some("public-read".to_string()),
            ..Default::default()
        })
        .await;

    println!("pushing s3 finished!");

    match result {
        Ok(success) => {
            println!("Success: {:?}", success);
        }
        Err(error) => {
            println!("Failure: {:?}", error);
        }
    }

    // Delete file
    let _ = std::fs::remove_file(file_path);

    Ok(())
}

#[debug_handler]
pub async fn pull_file(
    Path(file_id): Path<u32>,
    AuthToken(_user_id): AuthToken,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let path;

    {
        let state_list = state.state_list.lock().unwrap();
        if state_list.contains_key(&file_id) {
            if let Some(t) = state_list.get(&file_id) {
                path = t.file_path.clone();
            } else {
                return Err((StatusCode::NOT_FOUND, format!("File not found")));
            }
        } else {
            return Err((StatusCode::NOT_FOUND, format!("File not found")));
        }
    }

    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"file.bin\"",
        ),
    ];

    return Ok((headers, body));
}

pub async fn download_from_s3(
    Path(file_id): Path<u32>,
    AuthToken(_user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes<String>>> {
    if let Ok(fs) = get_file_info(file_id.to_string(), &pool).await {
        return Ok(axum::Json(AxumRes {
            code: 200,
            result: fs.path,
        }));
    }

    let ret = serde_json::json!({
        "error": "File not found",
    });
    return Err(into_response(400, ret));
}

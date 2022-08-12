use axum::{
    body::StreamBody,
    extract::{ContentLengthLimit, Extension, Multipart, Query},
    http::{self, header::HeaderMap},
    response::{IntoResponse, Response}, Json,
};
use axum_macros::debug_handler;
use csv::{StringRecord, Writer};
use sqlx::PgPool;
use std::{fs::File, io, sync::Arc};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use openapi_rs::openapi_proc_macro::handler;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;

use crate::handlers::video_instance_handler::{create_batch_data, db_get_generated_videos};
use crate::models::{
    csv::{AudioBatchId, CsvRequiredId},
    param::RequiredId,
};
use microservice_utils::{jwt::extractor::AuthToken, server::response::{AxumResult, AxumRes}};

// API
#[debug_handler]
pub async fn export_to_csv(
    params: Query<CsvRequiredId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let req = RequiredId {
        id: params.video_instance_id,
    };

    let videos = db_get_generated_videos(&user_id, &req, &pool).await;
    match videos {
        Ok(result) => {
            let csv_name = format!("csv/{}.csv", Uuid::new_v4().to_string());
            let mut csv_records = StringRecord::new();
            let mut new_csv = Writer::from_path(csv_name.clone()).unwrap();

            for item in result.iter() {
                let generated_video_id_str = item.id.to_string();
                let generated_video_id_str = &generated_video_id_str[..];
                csv_records.push_field(generated_video_id_str);
                for label in item.audio_lables.iter() {
                    csv_records.push_field(label);
                }
                if let Some(video_url) = item.video_url.clone() {
                    let video_url = &video_url[..];
                    csv_records.push_field(video_url);
                } else {
                    let item_status = &item.status[..];
                    csv_records.push_field(item_status);
                }
                let _ = new_csv.write_record(&csv_records);
                csv_records.clear();
            }
            new_csv.flush().unwrap();

            let mut headers = HeaderMap::new();
            headers.insert(
                http::header::CONTENT_DISPOSITION,
                format!("inline; filename={}", csv_name.clone())
                    .parse()
                    .unwrap(),
            );

            let file = match tokio::fs::File::open(csv_name.clone()).await {
                Ok(file) => file,
                Err(err) => {
                    return Err(Response::builder()
                        .status(404)
                        .body(format!("File not found: {}", err))
                        .unwrap());
                }
            };
            let stream = ReaderStream::new(file);

            let body = StreamBody::new(stream);
            Ok((headers, body))
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            Err(Response::builder()
                .status(500)
                .body(format!("{:?}", e))
                .unwrap())
        }
    }
}

const UPLOAD_LIMIT: u64 = 50 * 1000 * 1000;

#[debug_handler]
#[handler(method = "POST", tag = "csv")]
pub async fn import_from_csv(
    params: Query<AudioBatchId>,
    ContentLengthLimit(mut payload): ContentLengthLimit<Multipart, UPLOAD_LIMIT>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let field = payload.next_field().await.unwrap().unwrap();
    let mut bytes_data: &[u8] = &field.bytes().await.unwrap().to_vec();

    let csv_name = format!("csv/{}.csv", Uuid::new_v4().to_string());
    let mut out = File::create(&csv_name).expect("failed to create file");
    io::copy(&mut bytes_data, &mut out).expect("failed to copy content");

    let file = File::open(&csv_name);
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let mut record_vec = Vec::new();

        for item in record.iter() {
            record_vec.push(item.to_owned());
        }
        records.push(record_vec);
    }
    let _ = std::fs::remove_file(&csv_name);

    create_batch_data(records, &params.audio_batch_id, &user_id, &pool).await;

    let ret = serde_json::json!({
        "audio_batch_id": params.audio_batch_id,
    });
    Ok(axum::Json(AxumRes{code:200, result:ret}))
}

// Database

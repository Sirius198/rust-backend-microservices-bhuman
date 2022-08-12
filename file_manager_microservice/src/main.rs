mod ud;
use ud::{accept_file, create_new_file_on_db, download_from_s3, pull_file};
mod types;
use crate::types::AppState;
mod sock;
use sock::{media_recording_handler, websocket_handler};
mod db;
mod dir;
use dir::{create_new_folder, get_root_directory_id, get_sub_directory, move_folder_or_file, rename_folder};
mod model;

use axum::{
    extract::Extension,
    // http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    println!("File Manager Microservice is Starting...");

    // Load .env variables.
    dotenv().expect("Failed to read .env file");

    // save files to a separte directory to not override files in the current directory
    // tokio::fs::create_dir(UPLOADS_DIRECTORY)
    //     .await
    //     .expect("failed to create `uploads` directory");

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    // setup connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    println!("Database connected");

    // Application shared state
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {
        broadcaster: tx,
        state_list: Mutex::new(HashMap::new()),
        recording_list: Mutex::new(HashMap::new()),
        video_chunks: Mutex::new(Vec::new()),
    });

    // Cors
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_credentials(true)
        .allow_headers(Any);

    let pool_arc = Arc::new(pool.clone());

    // Route
    let app = Router::new()
        .route("/filemanager/pre_push/:pid", post(create_new_file_on_db))
        .route("/filemanager/push/:file_id", post(accept_file))
        .route("/filemanager/pull/:file_id", get(pull_file)) // concurrent download
        .route("/filemanager/download/:file_id", get(download_from_s3)) // returns S3 URLs
        .route("/ws/websocket/:token", get(websocket_handler))
        .route("/ws/record/:token", get(media_recording_handler))
        .nest("/filemanager/fs", folder_routes())
        .layer(cors)
        .layer(Extension(app_state))
        .layer(Extension(pool_arc))
        .layer(Extension(pool));

    axum::Server::bind(&"0.0.0.0:4007".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    // accept_file();
}

fn folder_routes() -> Router {
    Router::new()
        .route("/", get(get_root_directory_id))
        .route("/:folder_id", get(get_sub_directory))
        .route("/:folder_id/:folder_name", post(create_new_folder))
        .route(
            "/move/:src_folder_id/:dst_folder_id",
            get(move_folder_or_file),
        )
        .route("/ren/:file_id/:file_name", get(rename_folder))
}

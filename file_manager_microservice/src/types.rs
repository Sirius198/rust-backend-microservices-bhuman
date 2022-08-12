use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

// Our shared state
pub struct AppState {
    pub broadcaster: broadcast::Sender<String>,

    // file_id -> State
    pub state_list: Mutex<HashMap<u32, FileUploadingState>>,
    pub recording_list: Mutex<HashMap<i32, (i32, String)>>, // user_id->file_id

    // video chunks
    pub video_chunks: Mutex<Vec<VideoChunk>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileUploadingState {
    pub file_id: u32,
    pub file_name: String,
    pub file_path: String,
    // pub dealer_name: String,
    pub user_id: String,
    pub total_size: u32,
    pub uploaded_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadProgressChangeRequest {
    pub file_id: u32,
    pub total_size: u32,
    pub uploaded_size: u32,
}

#[derive(Debug, Clone)]
pub struct FileStruct {
    pub id: i32,
    pub pid: i32,
    pub user_id: String,
    pub name: String,
    pub path: String,
    pub size: usize,
    pub status: i8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecordingResponse {
    pub id: u32,
    pub name: String,
}

pub struct VideoChunk {
    pub file_path: String,
    pub data: Vec<u8>,
    pub finish: Option<bool>,
}
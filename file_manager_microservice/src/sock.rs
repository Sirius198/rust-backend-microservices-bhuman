// use crate::db::get_file_info;
use crate::types::VideoChunk;
use crate::types::{AppState, FileUploadingState, UploadProgressChangeRequest};
use crate::ud::create_new_file_record;
use crate::ud::upload_to_bucket;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use microservice_utils::jwt::auth::jwt_str_auth;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug)]
struct ProgressUpdateMsg {
    file_id: u32,
    total_size: u32,
    uploaded_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SocketMsg {
    header: String,
    body: String,
}

// Cannot write to file more than 15K at once
const MAX_WRITE_SIZE: usize = 15 * 1024;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(token): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    // return ws.on_upgrade(move |socket| websocket_callback(socket, state, pool, user_id));

    match jwt_str_auth(&token).await {
        Ok(user_id) => {
            ws.on_upgrade(move |socket| websocket_callback(socket, state, pool, user_id))
        }
        Err(e) => {
            println!("{:?}", e);
            ws.on_upgrade(|_| async {})
        }
    }
}

async fn websocket_callback(stream: WebSocket, state: Arc<AppState>, pool: PgPool, _user_id: String) {
    // By splitting we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Subscribe before sending joined message.
    let mut rx = state.broadcaster.subscribe();
    let _ = state.broadcaster.send(get_all_uploading_state_json(&state));

    // This task will receive broadcast messages and send text message to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // This task will receive messages from client and send them to broadcast subscribers.
    // This is main socket message handler
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(text) = message {
                let msg: SocketMsg = serde_json::from_str(&text).unwrap();
                println!("msg: {}", text);

                match msg.header.as_str() {
                    "UPDATE_PROGRESS" => {
                        // Broadcast updated progress to all sockets.

                        let body: UploadProgressChangeRequest =
                            serde_json::from_str(&msg.body).unwrap();
                        check_file_upload_state(&state, &body, &pool);
                        let _ = state.broadcaster.send(get_all_uploading_state_json(&state));
                    }
                    _ => {}
                }
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send user left message.
}

fn check_file_upload_state(state: &AppState, req: &UploadProgressChangeRequest, _pool: &PgPool) {
    let mut state_list = state.state_list.lock().unwrap();
    if state_list.contains_key(&req.file_id) {
        // println!("Total({}) / Done({})", req.total_size, req.uploaded_size);
        if req.total_size == req.uploaded_size {
            // If progress reaches 100%, removes from HashMap
            state_list.remove(&req.file_id);
        } else if let Some(t) = state_list.get(&req.file_id) {
            // Update uploaded size
            let state = FileUploadingState {
                file_id: req.file_id,
                file_name: t.file_name.clone(),
                file_path: t.file_path.clone(),
                // dealer_name: t.dealer_name.clone(),
                user_id: t.user_id.clone(),
                total_size: t.total_size,
                uploaded_size: req.uploaded_size,
            };
            state_list.insert(req.file_id, state);
        }
    } else {
        /*if let Ok(file_info) = get_file_info(req.file_id.to_string(), pool).await {
            let state = FileUploadingState {
                file_id: req.file_id,
                file_name: file_info.name,
                file_path: file_info.path,
                dealer_name: String::from("Later"),
                total_size: req.total_size,
                uploaded_size: req.uploaded_size,
            };
            state_list.insert(req.file_id, state);
        }*/
    }
}

fn get_all_uploading_state_json(state: &AppState) -> String {
    let state_list = state.state_list.lock().unwrap();
    let mut arr = Vec::new();
    for val in state_list.values() {
        arr.push(val);
    }
    if let Ok(s) = serde_json::to_string(&arr) {
        return s;
    }
    return String::from("");
}

pub async fn media_recording_handler(
    ws: WebSocketUpgrade,
    Path(token): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    // return ws.on_upgrade(move |socket| media_recording_callback(socket, state, pool, user_id));

    match jwt_str_auth(&token).await {
        Ok(user_id) => {
            ws.on_upgrade(move |socket| media_recording_callback(socket, state, pool, user_id))
        }
        Err(e) => {
            println!("{:?}", e);
            ws.on_upgrade(|_| async {})
        }
    }
}

async fn media_recording_callback(
    stream: WebSocket,
    _state: Arc<AppState>,
    pool: PgPool,
    user_id: String,
) {
    let (mut _sender, mut receiver) = stream.split();
    let mut file_name = String::from("");
    let mut pid: i32 = 0;
    let file_id: i32;
    let file_path: String;

    // Loop until a file name is received.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(txt) = message {
            let msg: serde_json::Value = serde_json::from_str(&txt).unwrap();
            file_name = msg["file_name"].to_string();
            pid = msg["pid"].to_string().parse().unwrap();
            break;
        }
    }

    if let Ok((id, path)) = create_new_file_record(&pool, user_id, file_name, 0, pid).await {
        file_id = id;
        file_path = path;
    } else {
        return;
    }

    let path1 = file_path.clone();
    let path2 = file_path.clone();
    let path3 = file_path.clone();
    println!("{}, {}", file_id, file_path);

    let arc_chunk: Arc<Mutex<Vec<VideoChunk>>> = Arc::new(Mutex::new(Vec::new()));
    let arc_chunk2 = Arc::clone(&arc_chunk);
    // let vc1 = Arc::clone(&vcs);
    let (chn_sender, chn_receiver) = mpsc::channel::<VideoChunk>();

    tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Close(_) => {
                    let mut list = arc_chunk.lock().unwrap();
                    list.push(VideoChunk {
                        data: Vec::new(),
                        file_path: "".to_owned(),
                        finish: Some(true),
                    });
                    break;
                }
                Message::Binary(bin) => {
                    let mut list = arc_chunk.lock().unwrap();
                    list.push(VideoChunk {
                        data: bin,
                        file_path: path2.clone(),
                        finish: None,
                    });
                    println!("received");
                    drop(list);
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let mut queue = arc_chunk2.lock().unwrap();
            let mut sleep_duration = 500;
            if queue.len() > 0 {
                let mut chunk = queue.remove(0);

                if chunk.data.len() > MAX_WRITE_SIZE {
                    let tail = chunk.data.drain(MAX_WRITE_SIZE..).collect();
                    let file_path = chunk.file_path.clone();
                    queue.insert(
                        0,
                        VideoChunk {
                            file_path,
                            data: tail,
                            finish: None,
                        },
                    );
                }

                // Terminate if finish is true
                let mut terminate = false;
                if let Some(_) = chunk.finish {
                    terminate = true;
                }
                let _ = chn_sender.send(chunk);
                if terminate {
                    break;
                }

                sleep_duration = 10;
            }

            drop(queue);
            thread::sleep(Duration::from_millis(sleep_duration));
        }
    });

    let writing_task = tokio::spawn(async move {
        loop {
            let chunk = chn_receiver.recv().unwrap();
            if let Some(_) = chunk.finish {
                break;
            }
            let _ = write_binary_file(chunk.data, path1.clone()).await;

            thread::sleep(Duration::from_millis(100));
        }
    });

    let _ = writing_task.await;

    // Upload to S3
    let t = upload_to_bucket(path3.clone()).await;
    if let Ok(_) = t {
        println!("Push to S3 success!");
    } else if let Err(e) = t {
        println!("Push to S3 failed: {}", e);
    }
}

async fn write_binary_file(data: Vec<u8>, filename: String) -> std::io::Result<()> {
    let path = filename;
    let mut file;
    if std::path::Path::new(&path).exists() {
        file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(&path)
            .await?;
    } else {
        file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await?;
    }

    let ret = file.write(&data).await;
    if let Ok(size) = ret {
        println!("Original: {}, Wrote {} bytes", data.len(), size);
    } else {
        println!("Something's wrong with writing");
    }

    Ok(())
}

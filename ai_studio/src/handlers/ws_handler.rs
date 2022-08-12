use axum::{
    extract::{Extension, Path, TypedHeader, WebSocketUpgrade},
    headers,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use dashmap::mapref::entry::Entry;
use std::time::Duration;
use std::{path::Path as FilePath, sync::Arc};
use tokio::time::{self, Instant};

use crate::models::ws_board::WsBoard;
use crate::models::ws_types::{Document, PersistedDocument, ServerState};

#[debug_handler]
pub async fn socket_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    Extension(state): Extension<ServerState>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected with {}", user_agent.as_str(), id);
    }
    let mut entry = match state.documents.entry(id.clone()) {
        Entry::Occupied(e) => e.into_ref(),
        Entry::Vacant(e) => {
            let wsboard = Arc::new(WsBoard::default());
            load(&id, &wsboard);
            tokio::spawn(save(id, Arc::clone(&wsboard)));

            e.insert(Document::new(wsboard))
        }
    };

    let value = entry.value_mut();
    value.last_accessed = Instant::now();
    let wsboard = Arc::clone(&value.wsboard);
    ws.on_upgrade(|socket| async move { wsboard.on_connection(socket).await })
}

const PERSIST_INTERVAL: Duration = Duration::from_secs(3);

async fn save(id: String, wsboard: Arc<WsBoard>) {
    while !wsboard.killed() {
        time::sleep(PERSIST_INTERVAL).await;

        let persist = wsboard.get_persist();
        let _ = std::fs::write(
            format!("spreadsheet/{}.json", id),
            serde_json::to_string_pretty(&persist).unwrap(),
        );
    }
}

fn load(id: &String, wsboard: &WsBoard) {
    let path = format!("spreadsheet/{}.json", id);
    if FilePath::new(&path).exists() {
        let persist = {
            let data = std::fs::read_to_string(&path);
            serde_json::from_str::<PersistedDocument>(&data.unwrap()).unwrap()
        };
        wsboard.set_persist(&persist);
    }
}

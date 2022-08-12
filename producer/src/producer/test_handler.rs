use std::sync::Arc;
use rdkafka::producer::FutureProducer;
use axum::{extract::Extension, extract::Query, body::Body, extract::rejection::JsonRejection, response::Response, Json};
use axum_macros::debug_handler;

use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestQuery {
    pub event: String
}

use crate::utils::{
    response::send_reponse,
};
use crate::producer::{
    producer::produce,
    ws_message::WsMessage,
};

#[debug_handler]
pub async fn test_handler(
    params: Query<TestQuery>,
    Extension(producer): Extension<Arc<FutureProducer>>
) -> Result<Response<Body>, Response<Body>> {

    let message = WsMessage {
        user_id: "1234567890".to_string(),
        message_type: "event".to_string(),
        message: params.event.clone(),
    };
    let msg_str = serde_json::to_string(&message).unwrap();  

    let produce = produce(&msg_str, &producer).await;
    println!("Message sent to kafka {:?}", message);

    send_reponse(200, Body::from("OK"))
}
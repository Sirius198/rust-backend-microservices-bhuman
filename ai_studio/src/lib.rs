use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use rdkafka::consumer::{CommitMode, Consumer};
use sqlx::PgPool;
use std::{env, ffi::OsStr, net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::{self};
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

pub mod handlers;
pub mod models;

use crate::handlers::actor_handler::{
    create_actor, create_actor_spec, delete_actor, delete_actor_spec, get_actor, get_actor_spec,
    update_actor, update_actor_spec,
};
use crate::handlers::csv_handler::{export_to_csv, import_from_csv, import_from_csv_spec};
use crate::handlers::folder_handler::{
    create_folder, create_folder_spec, delete_folder, delete_folder_spec, get_folder,
    get_folder_spec, update_folder, update_folder_spec,
};
use crate::handlers::segment_handler::{
    create_segment, create_segment_spec, delete_segment, delete_segment_spec, get_segment,
    get_segment_spec, update_segment, update_segment_spec,
};
use crate::handlers::video_instance_handler::{
    create_video_instance, create_video_instance_spec, delete_video_instance,
    delete_video_instance_spec, get_video_instance, get_video_instance_spec, update_video_instance,
    update_video_instance_spec,
};
use crate::handlers::ws_handler::socket_handler;
use crate::models::ws_types::ServerState;
use microservice_utils::{open_api::gen::generate_openapi_spec, server::{consumer::get_consumer, spa::SpaRouter}};
use microservice_utils::{
    open_api::gen::{GenSpec, Spec},
    server::error_404::error_404,
};

#[macro_use]
extern crate lazy_static;

fn ensure_var<K: AsRef<OsStr>>(key: K) -> anyhow::Result<String> {
    env::var(&key).map_err(|e| anyhow::anyhow!("{}: {:?}", e, key.as_ref()))
}

lazy_static! {
    static ref DATABASE_URL: String = ensure_var("DATABASE_URL").unwrap();
}

#[tokio::main]
pub async fn main() {
    dotenv().expect("Failed to read .env file");
    lazy_static::initialize(&DATABASE_URL);

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let axum_make_service = create_app(&pool);

    start_consumer();

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("Listening on http://{}", addr);

    axum_server::bind(addr)
        .serve(axum_make_service.into_make_service())
        .await
        .unwrap();
}

fn create_app(pool: &PgPool) -> Router {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "example_websockets=debug,tower_http=debug,librdkafka=trace,rdkafka::client=debug",
        )
    }

    let state = ServerState {
        documents: Default::default(),
    };
    tokio::spawn(cleaner(state.clone(), 1));

    let specs: Vec<Spec<GenSpec>> = vec![
        Spec {
            route: "/api/ai_studio/folder".into(),
            gen: Box::new(create_folder_spec),
        },
        Spec {
            route: "/api/ai_studio/folder".into(),
            gen: Box::new(update_folder_spec),
        },
        Spec {
            route: "/api/ai_studio/folder".into(),
            gen: Box::new(get_folder_spec),
        },
        Spec {
            route: "/api/ai_studio/folder".into(),
            gen: Box::new(delete_folder_spec),
        },
        Spec {
            route: "/api/ai_studio/actor".into(),
            gen: Box::new(create_actor_spec),
        },
        Spec {
            route: "/api/ai_studio/actor".into(),
            gen: Box::new(update_actor_spec),
        },
        Spec {
            route: "/api/ai_studio/actor".into(),
            gen: Box::new(get_actor_spec),
        },
        Spec {
            route: "/api/ai_studio/actor".into(),
            gen: Box::new(delete_actor_spec),
        },
        Spec {
            route: "/api/ai_studio/video_instance".into(),
            gen: Box::new(create_video_instance_spec),
        },
        Spec {
            route: "/api/ai_studio/video_instance".into(),
            gen: Box::new(update_video_instance_spec),
        },
        Spec {
            route: "/api/ai_studio/video_instance".into(),
            gen: Box::new(get_video_instance_spec),
        },
        Spec {
            route: "/api/ai_studio/video_instance".into(),
            gen: Box::new(delete_video_instance_spec),
        },
        Spec {
            route: "/api/ai_studio/segment".into(),
            gen: Box::new(create_segment_spec),
        },
        Spec {
            route: "/api/ai_studio/segment".into(),
            gen: Box::new(update_segment_spec),
        },
        Spec {
            route: "/api/ai_studio/segment".into(),
            gen: Box::new(get_segment_spec),
        },
        Spec {
            route: "/api/ai_studio/segment".into(),
            gen: Box::new(delete_segment_spec),
        },
        Spec {
            route: "/api/ai_studio/csv".into(),
            gen: Box::new(import_from_csv_spec),
        },
    ];

    generate_openapi_spec(specs).expect("failed to generate openapi spec");

    let pool_arc = Arc::new(pool.clone());

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_credentials(false)
        .allow_headers(Any);

    // Limit concurrency for all routes ,Trace layer for all routes
    let middleware_stack = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(ConcurrencyLimitLayer::new(64))
        .layer(cors)
        .into_inner();

    let app = Router::new()
        .merge(SpaRouter::new(vec!["/swagger-ui"], vec!["./swagger-ui"]))
        .route(
            "/api/ai_studio/folder",
            post(create_folder)
                .put(update_folder)
                .get(get_folder)
                .delete(delete_folder),
        )
        .route(
            "/api/ai_studio/actor",
            post(create_actor)
                .put(update_actor)
                .get(get_actor)
                .delete(delete_actor),
        )
        .route(
            "/api/ai_studio/video_instance",
            post(create_video_instance)
                .put(update_video_instance)
                .get(get_video_instance)
                .delete(delete_video_instance),
        )
        .route(
            "/api/ai_studio/segment",
            post(create_segment)
                .put(update_segment)
                .get(get_segment)
                .delete(delete_segment),
        )
        .route(
            "/api/ai_studio/csv",
            post(import_from_csv).get(export_to_csv),
        )
        .route("/socket/:id", get(socket_handler))
        .fallback(get(error_404))
        .layer(Extension(state))
        .layer(Extension(pool_arc))
        .layer(middleware_stack);

    return app;
}

fn start_consumer() {
    tokio::spawn(async move {
        let consumer = Arc::new(get_consumer("127.0.0.1:9092", "1234", &["bhuman_channel"]));
        loop {
            match consumer.recv().await {
                Err(e) => println!("Kafka error: {}", e),
                Ok(m) => {
                    let payload_s = match rdkafka::Message::payload_view::<str>(&m) {
                        None => "".to_string(),
                        Some(Ok(s)) => s.to_string(),
                        Some(Err(e)) => {
                            println!("Error while deserializing message payload: {:?}", e);
                            "".to_string()
                        }
                    };

                    println!("Received Message: {}", payload_s);

                    // let signal: WsMessage = WsMessage::from(payload_s.clone());

                    // let msg_str = serde_json::to_string(&signal).unwrap();
                    // let message = SocketMessage::Text(msg_str);

                    // let mut _clients = clients.lock().unwrap();
                    // match _clients.get(&signal.user_id) {
                    //     Some(v) => {
                    //         if let Some(sender) = &v.sender {
                    //             if sender.send(Ok(message)).is_err() {
                    //                 println!("Client disconnected {:?}", signal.user_id);
                    //                 _clients.remove(&signal.user_id);
                    //             }
                    //         }
                    //     }
                    //     None => {
                    //         println!("Comments send error");
                    //     }
                    // }
                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                }
            }
        }
    });
}

const HOUR: Duration = Duration::from_secs(3600);

/// Reclaims memory for documents.
async fn cleaner(state: ServerState, expiry_days: u32) {
    loop {
        time::sleep(HOUR).await;
        let mut keys = Vec::new();
        for entry in &*state.documents {
            if entry.last_accessed.elapsed() > HOUR * 24 * expiry_days {
                keys.push(entry.key().clone());
            }
        }
        println!("cleaner removing keys: {:?}", keys);
        for key in keys {
            state.documents.remove(&key);
        }
    }
}

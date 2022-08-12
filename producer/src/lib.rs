use std::{
    env, 
    sync::Arc,
    ffi::OsStr, 
    net::SocketAddr, 
};

use axum::{
    extract::Extension,
    response::Redirect,
    routing::{get, post, put, delete},
    Router,
};
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use dotenv::dotenv;
use sqlx::PgPool;

pub mod error_404;
pub mod utils;
pub mod producer;

pub mod invite;
pub mod contacts;
pub mod user;
pub mod workspace;

use crate::error_404::{
    error_404::error_404,
};
use crate::user::user_handler::{
    create_or_update,
    get_user,
    delete_user
};
use crate::invite::invite_handler::{
    generate_link,
    verify_link,
};
use crate::contacts::contacts_handler::{
    sync_contacts,
    get_contacts,
};
use crate::workspace::workspace_handler::{
    create_workspace,
    update_workspace,
    get_workspace,
    delete_workspace,
};
use crate::producer::producer::{
    get_producer,
};

#[macro_use]
extern crate lazy_static;

fn ensure_var<K: AsRef<OsStr>>(key: K) -> anyhow::Result<String> {
    env::var(&key).map_err(|e| anyhow::anyhow!("{}: {:?}", e, key.as_ref()))
}

lazy_static! {
    static ref ADDRESS: SocketAddr = format!("127.0.0.1:{}", ensure_var("PORT").unwrap())
        .parse()
        .unwrap();
    static ref URL: String = ensure_var("URL").unwrap();
    static ref DATABASE_URL: String = ensure_var("DATABASE_URL").unwrap();
    static ref AWS_ACCESS_KEY: String = ensure_var("AWS_ACCESS_KEY_ID").unwrap();
    static ref AWS_SECRET: String = ensure_var("AWS_SECRET_ACCESS_KEY").unwrap();
    static ref S3_BUCKET: String = ensure_var("S3_BUCKET").unwrap();
    static ref S3_REGION: String = ensure_var("S3_REGION").unwrap();
}

pub async fn init() -> Result<Router, anyhow::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "example_websockets=debug,tower_http=debug,librdkafka=trace,rdkafka::client=debug",
        )
    }

    dotenv().expect("Failed to read .env file");

    lazy_static::initialize(&ADDRESS);
    lazy_static::initialize(&URL);
    lazy_static::initialize(&DATABASE_URL);
    lazy_static::initialize(&AWS_ACCESS_KEY);
    lazy_static::initialize(&AWS_SECRET);
    lazy_static::initialize(&S3_BUCKET);
    lazy_static::initialize(&S3_REGION);

    let producer = Arc::new(get_producer("127.0.0.1:9092"));

    let pool = Arc::new(
        PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap(),
    );

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
        .route("/api/user", post(create_or_update)
                            .get(get_user)
                            .delete(delete_user))
        .route("/api/contacts", post(sync_contacts)
                            .get(get_contacts))
        .route("/api/invite", post(generate_link)
                            .put(verify_link))
        .route("/api/workspace", post(create_workspace)
                            .put(update_workspace)
                            .get(get_workspace)
                            .delete(delete_workspace))
        .route("/dl/:id", get(|| async { Redirect::permanent("https://platform-ui-ten.vercel.app/check-in") }))
        .layer(Extension(pool))
        .layer(Extension(producer))
        .layer(middleware_stack);

    println!("Listening on http://localhost:4000");    

    Ok(app.fallback(get(error_404)))    
}
use std::{
    env,
    sync::Arc,
    ffi::OsStr,
    net::SocketAddr,
};
use axum::{
    extract::{Extension, Path},
    routing::{get, post},
    Router,
    response::Redirect,
};
use invite::invite_handler::{generate_link_spec, verify_link_spec};
use microservice_utils::{open_api::gen::{GenSpec, Spec, generate_openapi_spec}, server::spa::SpaRouter};
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use dotenv::dotenv;
use sqlx::{Executor, PgPool};
use sync_wrapper::SyncWrapper;
use shuttle_service::error::CustomError;

pub mod invite;

use microservice_utils::server::error_404::{
    error_404,
};
use crate::invite::invite_handler::{
    generate_link,
    verify_link,
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

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
    
    let axum_make_service = create_app(&pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4002));
    println!("Listening on http://{}", addr);

    axum_server::bind(addr)
                .serve(axum_make_service.into_make_service())
                .await
                .unwrap();
}

#[shuttle_service::main]
async fn axum(pool: PgPool) -> shuttle_service::ShuttleAxum {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let app = create_app(&pool);
    let sync_wrapper = SyncWrapper::new(app);
    Ok(sync_wrapper)    
}

fn create_app(pool: &PgPool) -> Router {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "example_websockets=debug,tower_http=debug,librdkafka=trace,rdkafka::client=debug",
        )
    }
    
    let specs: Vec<Spec<GenSpec>> = vec![Spec {
        route: "/api/invite".into(),
        gen: Box::new(generate_link_spec)
    },Spec {
        route: "/api/invite".into(),
        gen: Box::new(verify_link_spec)
    }];

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
        .route("/api/invite", post(generate_link)
                            .put(verify_link))
        .route("/dl/:id", get(|Path(check_id): Path<String>| async move { Redirect::permanent(&format!("https://frontend_test.bhuman.ai/check-in/{}", check_id)) }))
        .fallback(get(error_404))
        .layer(Extension(pool_arc))
        .layer(middleware_stack);

    return app;
}
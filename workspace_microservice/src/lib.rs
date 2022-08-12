use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use shuttle_service::error::CustomError;
use sqlx::{Executor, PgPool};
use std::{env, ffi::OsStr, net::SocketAddr, sync::Arc};
use sync_wrapper::SyncWrapper;
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

pub mod producer;
pub mod workspace;

use microservice_utils::server::{error_404::error_404, spa::SpaRouter};

use microservice_utils::open_api::gen::{generate_openapi_spec, GenSpec, Spec};
use workspace::workspace_handler::{
    add_to_workspace_spec, create_workspace_spec, delete_workspace_spec, get_workspace_spec,
    remove_from_workspace_spec, update_workspace_spec,
};

use crate::producer::producer::get_producer;
use crate::workspace::workspace_handler::{
    add_to_workspace, create_workspace, delete_workspace, get_workspace, remove_from_workspace,
    update_workspace, MyWorkspaceService,
};
use microservice_utils::server::hybrid::hybrid;

pub mod workspace_service {
    tonic::include_proto!("workspace_service");
}

use workspace_service::workspace_service_server::WorkspaceServiceServer;

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
    let axum_make_service = create_app(pool.clone());

    let grpc_service = tonic::transport::Server::builder()
        .add_service(WorkspaceServiceServer::new(MyWorkspaceService::new(pool)))
        .into_service();

    let hybrid_make_service = hybrid(axum_make_service.into_make_service(), grpc_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4001));
    println!("Listening on http://{}", addr);

    axum_server::bind(addr)
        .serve(hybrid_make_service)
        .await
        .unwrap();
}

#[shuttle_service::main]
async fn axum(pool: PgPool) -> shuttle_service::ShuttleAxum {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let app = create_app(pool);
    let sync_wrapper = SyncWrapper::new(app);
    Ok(sync_wrapper)
}

fn create_app(pool: PgPool) -> Router {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "example_websockets=debug,tower_http=debug,librdkafka=trace,rdkafka::client=debug",
        )
    }

    let specs: Vec<Spec<GenSpec>> = vec![
        Spec {
            route: "/api/workspace".into(),
            gen: Box::new(create_workspace_spec),
        },
        Spec {
            route: "/api/workspace".into(),
            gen: Box::new(update_workspace_spec),
        },
        Spec {
            route: "/api/workspace".into(),
            gen: Box::new(get_workspace_spec),
        },
        Spec {
            route: "/api/workspace".into(),
            gen: Box::new(delete_workspace_spec),
        },
        Spec {
            route: "/api/workspace_util".into(),
            gen: Box::new(add_to_workspace_spec),
        },
        Spec {
            route: "/api/workspace_util".into(),
            gen: Box::new(remove_from_workspace_spec),
        },
    ];

    generate_openapi_spec(specs).expect("failed to generate openapi spec");

    let pool_arc = Arc::new(pool);

    let producer = Arc::new(get_producer("127.0.0.1:9092"));

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
            "/api/workspace",
            post(create_workspace)
                .put(update_workspace)
                .get(get_workspace)
                .delete(delete_workspace),
        )
        .route(
            "/api/workspace_util",
            post(add_to_workspace).delete(remove_from_workspace),
        )
        .fallback(get(error_404))
        .layer(Extension(pool_arc))
        .layer(Extension(producer))
        .layer(middleware_stack);

    return app;
}

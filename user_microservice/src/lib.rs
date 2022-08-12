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

pub mod user;

use crate::user::user_handler::{create_user, delete_user, get_user, update_user, MyUserService};
use microservice_utils::server::{hybrid::hybrid, spa::SpaRouter};
use microservice_utils::{
    open_api::gen::{generate_openapi_spec, GenSpec, Spec},
    server::error_404::error_404,
};
use user::user_handler::{create_user_spec, delete_user_spec, get_user_spec, update_user_spec};

pub mod user_service {
    tonic::include_proto!("user_service");
}

use user_service::user_service_server::UserServiceServer;

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

    let grpc_service = tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(MyUserService::new(pool)))
        .into_service();

    let hybrid_make_service = hybrid(axum_make_service.into_make_service(), grpc_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
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

    let specs: Vec<Spec<GenSpec>> = vec![
        Spec {
            route: "/api/user".into(),
            gen: Box::new(create_user_spec),
        },
        Spec {
            route: "/api/user".into(),
            gen: Box::new(update_user_spec),
        },
        Spec {
            route: "/api/user".into(),
            gen: Box::new(get_user_spec),
        },
        Spec {
            route: "/api/user".into(),
            gen: Box::new(delete_user_spec),
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
            "/api/user",
            post(create_user)
                .put(update_user)
                .get(get_user)
                .delete(delete_user),
        )
        .fallback(get(error_404))
        .layer(Extension(pool_arc))
        .layer(middleware_stack);

    return app;
}

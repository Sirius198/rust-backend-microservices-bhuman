use std::{env, ffi::OsStr, net::SocketAddr, sync::Arc};
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use shuttle_service::error::CustomError;
use sqlx::{Executor, PgPool};
use sync_wrapper::SyncWrapper;
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use microservice_utils::{open_api::gen::{generate_openapi_spec, Spec, GenSpec}, server::spa::SpaRouter};
use microservice_utils::server::hybrid::hybrid;
use microservice_utils::server::error_404::error_404;

pub mod tags;
pub mod groups;
pub mod contacts;

use crate::contacts::contacts_handler::{sync_contacts_spec, get_contacts_spec};
use crate::tags::tags_handler::{create_tag_spec, update_tag_spec, get_tag_spec, delete_tag_spec};
use crate::groups::groups_handler::{add_to_tag_spec, get_from_tag_spec, delete_from_tag_spec};

use crate::contacts::contacts_handler::{get_contacts, sync_contacts};
use crate::tags::tags_handler::{create_tag, update_tag, get_tag, delete_tag};
use crate::groups::groups_handler::{add_to_tag, get_from_tag, delete_from_tag};
use crate::{
    contacts::contacts_handler::{
        address_book_service::address_book_service_server::AddressBookServiceServer,
        MyAddressBookService,
    },
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
    // addres book service
    let grpc_service = tonic::transport::Server::builder()
        .add_service(AddressBookServiceServer::new(MyAddressBookService::new(
            pool,
        )))
        .into_service();

    // addres book service
    let hybrid_make_service = hybrid(axum_make_service.into_make_service(), grpc_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4003));
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

    let specs: Vec<Spec<GenSpec>> = vec![Spec {
        route: "/api/contacts".into(),
        gen: Box::new(get_contacts_spec)
    },Spec {
        route: "/api/contacts".into(),
        gen: Box::new(sync_contacts_spec)
    },Spec {
        route: "/api/contacts/tag".into(),
        gen: Box::new(create_tag_spec)
    },Spec {
        route: "/api/contacts/tag".into(),
        gen: Box::new(get_tag_spec)
    },Spec {
        route: "/api/contacts/tag".into(),
        gen: Box::new(update_tag_spec)
    },Spec {
        route: "/api/contacts/tag".into(),
        gen: Box::new(delete_tag_spec)
    },Spec {
        route: "/api/contacts/group".into(),
        gen: Box::new(add_to_tag_spec)
    },Spec {
        route: "/api/contacts/group".into(),
        gen: Box::new(get_from_tag_spec)
    },Spec {
        route: "/api/contacts/group".into(),
        gen: Box::new(delete_from_tag_spec)
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
        .route("/api/contacts", post(sync_contacts).get(get_contacts))
        .route("/api/contacts/tag", post(create_tag).get(get_tag).put(update_tag).delete(delete_tag))
        .route("/api/contacts/group", post(add_to_tag).get(get_from_tag).delete(delete_from_tag))
        .fallback(get(error_404))
        .layer(Extension(pool_arc))
        .layer(middleware_stack);

    return app;
}

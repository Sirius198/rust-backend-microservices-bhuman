#[tokio::main]
async fn main() {
    let app = producer::init().await.expect("Failed to create app");
    axum::Server::bind(&"127.0.0.1:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
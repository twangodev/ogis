mod routes;

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = routes::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("OGIS server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

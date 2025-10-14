use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ping", get(ping));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("OGIS server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn ping() -> &'static str {
    "pong"
}

mod generator;
mod routes;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub fontdb: Arc<usvg::fontdb::Database>,
}

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load fonts once at startup
    tracing::info!("Loading system fonts...");
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb.set_sans_serif_family("Arial");

    tracing::info!("Loaded {} font faces", fontdb.faces().count());
    for face in fontdb.faces() {
        let families: Vec<String> = face.families.iter().map(|f| f.0.clone()).collect();
        tracing::debug!("Font: {} ({})", families.join(", "), face.post_script_name);
    }

    let state = AppState {
        fontdb: Arc::new(fontdb),
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("OGIS server listening on http://0.0.0.0:3000");
    tracing::info!("Swagger UI available at http://0.0.0.0:3000/docs");
    axum::serve(listener, app).await.unwrap();
}

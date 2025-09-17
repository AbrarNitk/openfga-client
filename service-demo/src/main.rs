use service_demo::context::Ctx;
use service_demo::listener;
use service_demo::routes;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize the application context
    let ctx = match Ctx::new().await {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Failed to initialize application context: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize the application
    let app = routes::create_routes(ctx).layer(TraceLayer::new_for_http());

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 5001));
    tracing::info!("Server listening on {}", addr);

    listener::serve(app, addr).await.unwrap();
}

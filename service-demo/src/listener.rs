use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Starts the HTTP server with the given router
pub async fn serve(app: Router, addr: SocketAddr) -> Result<(), std::io::Error> {
    tracing::info!("Listener starting on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server started successfully");
    axum::serve(listener, app.into_make_service()).await
}

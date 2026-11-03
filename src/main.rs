use learn_rust::app;
use learn_rust::config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).init();

    let config = Config::from_env()?;
    let addr = config.socket_addr();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app::router())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("server stopped");
    Ok(())
}

/// Resolves on Ctrl+C, or on SIGTERM (what `docker stop` and Kubernetes send).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install the Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install the SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutdown signal received, draining connections");
}

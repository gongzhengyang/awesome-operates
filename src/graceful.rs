use tokio::signal;

/// used for axum
/// ```
/// use axum::Router;
///
/// async fn running() {
///  //   let app = Router::new();
///  // check details at https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
///  //  hyper::Server::bind(&"0.0.0.0:3000".parse().unwrap())
///  //     .serve(app.into_make_service())
///  //    .with_graceful_shutdown(awesome_operates::graceful::shutdown_signal())
///  //    .await
///  //     .unwrap();
/// }
/// ```
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}

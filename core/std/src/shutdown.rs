use tokio::signal;

/// Waits for either a SIGINT (Ctrl+C) or SIGTERM signal.
///
/// This function returns a future that resolves when one of the signals is received.
/// It handles the platform-specific details for listening to these signals.
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

    println!("\nSignal received, starting graceful shutdown.");
}

pub async fn hop_signal() {
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::hangup())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    
    // TODO implement #[cfg(not(unix))]

    tokio::select! {
        _ = terminate => {},
    }

    println!("\nSignal received, starting graceful restart.");
}

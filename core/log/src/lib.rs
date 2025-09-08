pub mod env;

use std::io::stdout;
use std::path::{Path, PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt,
    layer::SubscriberExt, // Important for .with() method
    util::SubscriberInitExt, // Important for .init() method
    EnvFilter, Layer,
};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};

// This struct will hold the guard, keeping it in scope for the
// lifetime of the application.
pub struct LogGuard {
    _guard: Option<WorkerGuard>,
}

pub fn init_tracer() -> LogGuard {
    // 1. Create a single filter that will be shared by all layers.
    let default_filter: LevelFilter = if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    // 2. Create the console layer conditionally.
    // It will be `Some(layer)` in debug builds, `None` in release.
    let console_layer = if cfg!(debug_assertions) {
        let env_filter = EnvFilter::builder()
            .with_default_directive(default_filter.into())
            .from_env_lossy();
        
        let layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_ansi(true)
            .with_writer(stdout)
            .with_filter(env_filter); // Apply the shared filter
        Some(layer)
    } else {
        None
    };

    // 3. Create the file layer conditionally.
    // This also captures the worker guard that needs to be returned.
    let mut file_guard = None; // The guard must be stored to be returned later.
    let file_layer = match env::log_path_env() {
        // If a log path is found, create the file layer.
        Ok(log_path) => {
            println!("Logging to file: {}", log_path);
            let env_filter = EnvFilter::builder()
                .with_default_directive(default_filter.into())
                .from_env_lossy();
            
            // TODO to env
            let logfile = FileRotate::new(
                log_path.clone(),
                AppendCount::new(10),
                ContentLimit::Bytes(10 * 1024 * 1024), // 25MB
                Compression::None,
                None,
            );

            let path = Path::new(&log_path);
            if !path.exists() {
                panic!("Wasn't able to create log file");
            }

            let (non_blocking_writer, guard) = tracing_appender::non_blocking(logfile);

            // IMPORTANT: Store the guard so we can return it.
            file_guard = Some(guard);

            let layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(false) // No colors in files
                .with_writer(non_blocking_writer)
                .with_filter(env_filter); // Apply the shared filter
            Some(layer)
        }
        
        // If no log path is found...
        Err(_) => {
            // ...panic in release builds.
            if !cfg!(debug_assertions) {
                panic!("LOG_PATH environment variable is not set in release build.");
            }
            
            // ...otherwise, do nothing in debug builds.
            None
        }
    };

    // 4. Combine the layers with the registry and initialize.
    // The `.with()` method accepts `Option<Layer>`, neatly handling our conditions.
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    // 5. Return the guard for the file writer.
    // This guard must be kept alive for the duration of the program.
    LogGuard { _guard: file_guard }
}
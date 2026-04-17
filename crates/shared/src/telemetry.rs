use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

pub fn init(log_level: &str, format: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    match format {
        "json" => {
            fmt()
                .with_env_filter(filter)
                .json()
                .init();
        }
        _ => {
            fmt()
                .with_env_filter(filter)
                .init();
        }
    }
}

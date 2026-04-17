use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

pub fn init(log_level: &str, format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let _ = match format {
        "json" => fmt().with_env_filter(filter).json().try_init(),
        _ => fmt().with_env_filter(filter).try_init(),
    };
}

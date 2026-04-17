use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Config error: {0}")]
    Config(Box<figment::Error>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Info source not found: {0}")]
    InfoSourceNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<figment::Error> for AppError {
    fn from(e: figment::Error) -> Self {
        AppError::Config(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

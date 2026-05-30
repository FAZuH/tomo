#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("failed to connect to database: {0}")]
    Connection(String),

    #[error("failed to run database migrations: {0}")]
    Migration(String),

    #[error("failed to run sql: {0}")]
    Sql(String),
}
impl From<r2d2::Error> for RepoError {
    fn from(value: r2d2::Error) -> Self {
        Self::Connection(value.to_string())
    }
}

impl From<diesel::result::Error> for RepoError {
    fn from(value: diesel::result::Error) -> Self {
        Self::Sql(value.to_string())
    }
}

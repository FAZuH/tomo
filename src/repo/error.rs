#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Error when trying to connect to database: {0}")]
    Connection(String),
}

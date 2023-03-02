
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("An error occurred in the database.")]
    DatabaseError(#[from] sqlx::Error)
}
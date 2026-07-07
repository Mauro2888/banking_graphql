use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found with id {0}")]
    UserNotFound(String),
    #[error("Non autorizzato")]
    Unauthorized,
    #[error("Internal error")]
    InternalError,
    #[error("Event publishing failed")]
    EventPublishError
}
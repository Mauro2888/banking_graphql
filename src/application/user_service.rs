// trait + impl dei casi d'uso

use crate::ports::UserRepository;
use std::sync::Arc;
use uuid::Uuid;
use crate::adapters::outbound::KafkaService;
use crate::domain::{User, DomainError};
use crate::application::command::CreateUserCommand;

pub struct UserHandler {
    repo: Arc<dyn UserRepository>,
    publisher: Arc<KafkaService>
}
impl UserHandler {
    pub fn new(repo: Arc<dyn UserRepository>, publisher: Arc<KafkaService>) -> Self {
        UserHandler { repo, publisher }
    }

    pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<User, DomainError> {
        let user = User::new(&cmd.name);
        self.repo.create_user(&user).await?;

        let event = serde_json::json!({
        "event_type": "USER_CREATED",
        "user_id": user.id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

        self.publisher
            .publish("user-events", &user.id.to_string(), &event)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Publish failed");
                DomainError::EventPublishError
            })?;
        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        self.repo.find_user_by_id(id).await
    }

    pub async fn find_all_users(&self) -> Result<Vec<User>, DomainError> {
        self.repo.find_all_users().await
    }

    pub async fn delete_user_by_id(&self, id: Uuid) -> Result<(), DomainError> {
        self.repo.delete_user(id).await
    }
    pub async fn find_user_by_name(&self, name: &str) -> Result<User, DomainError> {
        self.repo.find_by_name(name).await
    }
}

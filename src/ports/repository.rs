// trait Repository, trait EventPublisher

use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::{Order, User};
use crate::domain::DomainError;

#[async_trait]
pub trait UserRepository: Sync + Send {
    async fn create_user(&self, name:&User) -> Result<User, DomainError>;
    async fn find_user_by_id(&self, id: Uuid) -> Result<User, DomainError>;
    async fn find_all_users(&self) -> Result<Vec<User>,DomainError>;
    async fn delete_user(&self, user_id: Uuid) -> Result<(), DomainError>;
    async fn find_by_name(&self, name: &str) -> Result<User, DomainError>;
}

#[async_trait]
pub trait OrderRepository: Sync + Send {
    async fn create_order(&self, order: &Order) -> Result<Order, DomainError>;
    async fn delete_order(&self, id: Uuid) -> Result<(), DomainError>;
    async fn find_order_by_user(&self, user_id: Uuid) -> Result<Vec<Order>, DomainError>;
    async fn find_all_orders(&self) -> Result<Vec<Order>, DomainError>;
}
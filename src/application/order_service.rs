// trait + impl dei casi d'uso

use crate::application::command::CreateOrderCommand;
use crate::domain::{DomainError, Order};
use crate::ports::OrderRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct OrderHandler {
    repo: Arc<dyn OrderRepository>,
}


impl OrderHandler {
    pub fn new(repo: Arc<dyn OrderRepository>) -> Self {
        OrderHandler { repo }
    }

    pub async fn create_order(&self, cmd: CreateOrderCommand) -> Result<Order, DomainError> {
        let order = Order::new(&cmd.name,cmd.user_id,cmd.total);
        self.repo.create_order(&order).await
    }

    pub async fn find_order_by_user(&self, id: Uuid) -> Result<Vec<Order>, DomainError> {
        self.repo.find_order_by_user(id).await
    }

    pub async fn find_all_orders(&self) -> Result<Vec<Order>, DomainError> {
        self.repo.find_all_orders().await
    }

    pub async fn delete_order_by_id(&self, order_id: Uuid) -> Result<(), DomainError> {
        self.repo.delete_order(order_id).await
    }

}

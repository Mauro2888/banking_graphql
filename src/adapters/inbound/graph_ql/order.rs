use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use crate::domain::Order;

#[derive(InputObject)]
pub struct CreateOrderInput {
    pub name: String,
    pub user_id: Uuid,
    pub total: Decimal,
}

#[derive(SimpleObject)]
pub struct OrderView {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub total: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<Order> for OrderView {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            name: o.name,
            user_id: o.user_id,
            total: o.total,
            created_at: o.created_at,
        }
    }
}
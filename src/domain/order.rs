// User (dominio) + logica + errori di dominio

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub total: Decimal,
    pub created_at: DateTime<Utc>,
}

impl Order {
    pub fn new(name: &str, user_id: Uuid, total: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            user_id,
            total,
            created_at: Utc::now(),
        }
    }
}


// logica di business come deposit transfer
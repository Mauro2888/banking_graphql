// User (dominio) + logica + errori di dominio

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub name: String,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            balance: Decimal::ZERO,
            created_at: Utc::now(),
        }
    }
}


// logica di business come deposit transfer
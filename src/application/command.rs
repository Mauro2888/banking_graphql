// Command DTO del layer applicativo (nessuna dipendenza da adapter)

use rust_decimal::Decimal;
use uuid::Uuid;

pub struct CreateUserCommand {
    pub name: String,
}

pub struct CreateOrderCommand {
    pub name: String,
    pub user_id: Uuid,
    pub total: Decimal
}
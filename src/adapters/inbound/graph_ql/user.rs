use std::sync::Arc;
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use crate::application::OrderHandler;
use crate::adapters::inbound::graph_ql::order::OrderView;

#[derive(InputObject)]
pub struct CreateUserInput {
    pub name: String,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct UserView {
    pub id: Uuid,
    pub name: String,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl UserView {
    // Campo di relazione: scatta SOLO se il client chiede `orders`
    async fn orders(&self, ctx: &Context<'_>) -> Result<Vec<OrderView>> {
        let handler = ctx.data::<Arc<OrderHandler>>()?;
        let orders = handler.find_order_by_user(self.id).await?;
        Ok(orders.into_iter().map(OrderView::from).collect())
    }
}
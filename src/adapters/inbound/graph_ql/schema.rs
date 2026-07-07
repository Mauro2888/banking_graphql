//# QueryRoot, MutationRoot (i punti d'ingresso)

use crate::adapters::inbound::graph_ql::order::{CreateOrderInput, OrderView};
use crate::adapters::inbound::graph_ql::user::{CreateUserInput, UserView};
use crate::adapters::outbound::KafkaService;
use crate::application::{CreateOrderCommand, CreateUserCommand, OrderHandler, UserHandler};
use crate::domain::User;
use async_graphql::*;
use rdkafka::producer::FutureProducer;
use std::sync::Arc;
use uuid::Uuid;

pub struct MutationRoot;
pub struct QueryRoot;

impl From<CreateUserInput> for CreateUserCommand {
    fn from(value: CreateUserInput) -> Self {
        Self { name: value.name }
    }
}

impl From<CreateOrderInput> for CreateOrderCommand {
    fn from(order: CreateOrderInput) -> Self {
        Self {
            name: order.name,
            user_id: order.user_id,
            total: order.total,
        }
    }
}

impl From<User> for UserView {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            balance: user.balance,
            created_at: user.created_at,
        }
    }
}

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<UserView> {
        let handler = ctx.data::<Arc<UserHandler>>()?;
        let user = handler.create_user(input.into()).await?;
        Ok(user.into())
    }

    async fn delete_user(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let handler = ctx.data::<Arc<UserHandler>>()?;
        let uuid = Uuid::parse_str(&id)?;
        handler.delete_user_by_id(uuid).await?;
        Ok(true)
    }

    async fn create_order(&self, ctx: &Context<'_>, input: CreateOrderInput) -> Result<OrderView> {
        let handler = ctx.data::<Arc<OrderHandler>>()?;
        let order = handler.create_order(input.into()).await?;
        Ok(order.into())
    }

    async fn delete_order(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let handler = ctx.data::<Arc<OrderHandler>>()?;
        let uuid = Uuid::parse_str(&id)?;
        handler.delete_order_by_id(uuid).await?;
        Ok(true)
    }
}

#[Object]
impl QueryRoot {
    async fn find_user_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<UserView> {
        let handler = ctx.data::<Arc<UserHandler>>()?;
        let uuid = Uuid::parse_str(&id)?;
        let user = handler.find_user_by_id(uuid).await?;
        Ok(user.into())
    }

    async fn find_all_users(&self, ctx: &Context<'_>) -> Result<Vec<UserView>> {
        let handler = ctx.data::<Arc<UserHandler>>()?;
        let users = handler.find_all_users().await?;
        Ok(users.into_iter().map(UserView::from).collect())
    }
    async fn find_user_by_name(&self, ctx: &Context<'_>, name: String) -> Result<UserView> {
        let handler = ctx.data::<Arc<UserHandler>>()?;
        let user = handler.find_user_by_name(&name).await?;
        Ok(user.into())
    }

    async fn find_orders_by_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<OrderView>> {
        let handler = ctx.data::<Arc<OrderHandler>>()?;
        let orders = handler
            .find_order_by_user(Uuid::parse_str(&user_id)?)
            .await?;
        Ok(orders.into_iter().map(OrderView::from).collect())
    }
}

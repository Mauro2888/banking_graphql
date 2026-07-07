use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::domain::{DomainError, User,Order};
use crate::ports::{UserRepository,OrderRepository};

#[derive(FromRow, Clone, Debug)]
pub struct UserEntity {
    #[sqlx(flatten)]
    pub metadata: BaseEntity,
    pub name: String,
    pub balance: Decimal,
}

#[derive(FromRow, Clone, Debug)]
pub struct OrderEntity {
    #[sqlx(flatten)]
    pub metadata: BaseEntity,
    pub name: String,
    pub user_id: Uuid,
    pub total: Decimal,
}

#[derive(FromRow,Debug, Clone)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseEntity {
    pub fn new() -> Self {
        Self::with_id(Uuid::now_v7())
    }

    pub fn with_id(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_existing(id: Uuid, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            updated_at: Utc::now(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl From<OrderEntity> for Order {
    fn from(value: OrderEntity) -> Self {
        Self {
            id: value.metadata.id,
            user_id: value.user_id,
            name: value.name,
            total: value.total,
            created_at: value.metadata.created_at,
        }
    }
}

impl From<&Order> for OrderEntity {
    fn from(o: &Order) -> Self {
        Self {
            metadata: BaseEntity::from_existing(o.id, o.created_at),
            name: o.name.clone(),
            user_id: o.user_id,
            total: o.total,
        }
    }
}

impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        Self {
            id: entity.metadata.id,
            name: entity.name,
            balance: entity.balance,
            created_at: entity.metadata.created_at,
        }
    }
}
impl From<&User> for UserEntity {
    fn from(user: &User) -> Self {
        Self {
            metadata: BaseEntity::from_existing(user.id, user.created_at),
            name: user.name.clone(),
            balance: user.balance,
        }
    }
}

fn map_db(e: sqlx::Error) -> DomainError {
    tracing::error!(error = ?e, "db error");
    DomainError::InternalError
}

pub struct PostgresUserRepository { pool: PgPool }


impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}


#[async_trait]
impl OrderRepository for PostgresUserRepository {
    async fn create_order(&self, order: &Order) -> Result<Order, DomainError> {
        let e: OrderEntity = order.into();
        sqlx::query(
            "INSERT INTO orders (id, name, user_id, total, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)"
        )
            .bind(e.metadata.id)
            .bind(&e.name)
            .bind(e.user_id)
            .bind(e.total)
            .bind(e.metadata.created_at)
            .bind(e.metadata.updated_at)
            .execute(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(e.into())
    }

    async fn delete_order(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM orders WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(())
    }

    async fn find_order_by_user(&self, user_id: Uuid) -> Result<Vec<Order>, DomainError> {
        let rows = sqlx::query_as::<_, OrderEntity>(
            "SELECT id, name, user_id, total, created_at, updated_at
               FROM orders WHERE user_id = $1"
        )
            .bind(user_id)
            .fetch_all(&self.pool)      // fetch_all = molte righe
            .await
            .map_err(map_db)?;
        Ok(rows.into_iter().map(Order::from).collect())
    }

    async fn find_all_orders(&self) -> Result<Vec<Order>, DomainError> {
        let rows = sqlx::query_as::<_, OrderEntity>(
            "SELECT id, name, user_id, total, created_at, updated_at FROM orders"
        )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(rows.into_iter().map(Order::from).collect())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository{
    async fn create_user(&self, user: &User) -> Result<User, DomainError> {
        let e: UserEntity = user.into();
        sqlx::query(
            "INSERT INTO users (id, name, balance, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)"
        )
            .bind(e.metadata.id)
            .bind(&e.name)
            .bind(e.balance)
            .bind(e.metadata.created_at)
            .bind(e.metadata.updated_at)
            .execute(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(e.into())
    }

    async fn find_user_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        sqlx::query_as::<_, UserEntity>(
            "SELECT id, name,balance,created_at, updated_at FROM users WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db)?
            .map(User::from)
            .ok_or(DomainError::UserNotFound(format!("User with id {} not found", id)))
    }

    async fn find_all_users(&self) -> Result<Vec<User>, DomainError> {
        let users = sqlx::query_as::<_, UserEntity>(
            "SELECT id, name,balance, created_at, updated_at FROM users",
        )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(users.into_iter().map(User::from).collect())
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await.map_err(map_db)?;
        Ok(())
    }

    async fn find_by_name(&self, name: &str) -> Result<User, DomainError> {
        sqlx::query_as::<_, UserEntity>(
            "SELECT id, name, balance, created_at, updated_at FROM users WHERE name = $1",
        )
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db)?
            .map(User::from)
            .ok_or(DomainError::UserNotFound(format!("User with name {} not found", name)))
    }
}
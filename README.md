# Banking GraphQL — Architettura

Servizio Rust con **Actix-web + async-graphql + SQLx + Kafka**, strutturato in
**architettura esagonale** (Ports & Adapters) con vocabolario **CQRS**.

Il principio: il **dominio** è al centro e non dipende da nulla. Gli **adapter**
(GraphQL, Postgres, Kafka) stanno ai bordi e dipendono dal dominio, mai il contrario.

---

## Struttura delle cartelle

```
src/
├── main.rs                      # composition root: crea le dipendenze e le collega
│
├── domain/                      # ⬛ IL CUORE — zero dipendenze esterne
│   ├── mod.rs
│   ├── user.rs                  # entità User + logica di business (deposit, ecc.)
│   └── error.rs                 # DomainError (regole di business violate)
│
├── application/                 # 🟦 USE CASE — orchestrano il dominio
│   ├── mod.rs
│   ├── commands.rs              # CreateUserCommand, DepositCommand (intenzioni)
│   └── handlers.rs              # command/query handler (eseguono i use case)
│
├── ports/                       # 🟨 INTERFACCE — i trait (contratti)
│   ├── mod.rs
│   ├── repository.rs            # trait UserRepository
│   └── event_publisher.rs       # trait EventPublisher
│
└── adapters/                    # 🟩 IMPLEMENTAZIONI concrete dei port
    ├── mod.rs
    ├── inbound/                 # ciò che ENTRA (chi chiama la app)
    │   ├── mod.rs
    │   ├── graph_ql/
    │   │   ├── mod.rs
    │   │   ├── schema.rs        # QueryRoot, MutationRoot
    │   │   └── user.rs          # CreateUserInput, UserView, resolver, From<>
    │   └── rest/                # (opzionale) handler Actix REST
    │       ├── mod.rs
    │       └── error.rs         # impl ResponseError for DomainError
    └── outbound/                # ciò che ESCE (cosa la app chiama)
        ├── mod.rs
        ├── postgres.rs          # UserEntity + PostgresUserRepository + From<>
        └── kafka.rs             # KafkaEventPublisher
```

**Regola di dipendenza:** le frecce puntano sempre verso il centro.
`adapters → ports → application → domain`. Il dominio non importa mai nulla
da application/ports/adapters.

---

## I tipi: uno per layer, con suffisso che ne indica il ruolo

Lo stesso concetto ("user") esiste in forme diverse in ogni layer. Il **file** può
chiamarsi `user.rs` in più cartelle (il modulo lo distingue); i **tipi** hanno nomi
distinti che comunicano il ruolo.

| Layer | Tipo | File | Ruolo |
|---|---|---|---|
| Dominio | `User` | `domain/user.rs` | L'entità: cosa È un utente + regole |
| Command | `CreateUserCommand` | `application/commands.rs` | L'intenzione: cosa si VUOLE fare |
| Input GraphQL | `CreateUserInput` | `adapters/inbound/graph_ql/user.rs` | Il body della mutation (`InputObject`) |
| Output GraphQL | `UserView` | `adapters/inbound/graph_ql/user.rs` | La risposta (`SimpleObject`) |
| Persistenza | `UserEntity` | `adapters/outbound/postgres.rs` | La riga del DB (`FromRow`) |

Naming: **tipi in `PascalCase`** (`UserView`, mai `User_View`), **file in `snake_case`**
(`user.rs`). Vocabolario CQRS: comandi all'imperativo (`DepositCommand`),
eventi al passato (`AmountDeposited`).

---

## Flusso dei dati (WRITE) — creare un utente

```
   Client GraphQL
        │  mutation { createUser(input: { name: "Mauro" }) { id name balance } }
        ▼
┌─────────────────────────────────────────────────────────────┐
│ adapters/inbound/graph_ql/user.rs                            │
│                                                              │
│   CreateUserInput ──From──▶ CreateUserCommand                │
│   (InputObject)             (intenzione pura)                │
└───────────────────────────────┬──────────────────────────────┘
                                 ▼
┌─────────────────────────────────────────────────────────────┐
│ application/handlers.rs                                      │
│                                                              │
│   handle(CreateUserCommand):                                 │
│     1. User::new(cmd.name)          ← crea entità di dominio  │
│     2. repo.save(user)              ← via trait (port)        │
│     3. publisher.publish(event)     ← via trait (port)        │
└───────────────────────────────┬──────────────────────────────┘
                    ┌────────────┴────────────┐
                    ▼                          ▼
┌──────────────────────────────┐  ┌──────────────────────────────┐
│ adapters/outbound/postgres.rs│  │ adapters/outbound/kafka.rs   │
│                              │  │                              │
│   User ──From──▶ UserEntity  │  │   pubblica AccountCreated    │
│   INSERT INTO users ...      │  │   su Kafka                   │
└──────────────────────────────┘  └──────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────┐
│ ritorno: User ──From──▶ UserView ──▶ risposta GraphQL         │
└─────────────────────────────────────────────────────────────┘
```

## Flusso dei dati (READ) — leggere un utente

```
   Client GraphQL
        │  query { user(id: "...") { id name balance } }
        ▼
   graph_ql/user.rs  →  application/handlers.rs  →  postgres.rs
        │                       │                       │
        │                       │              SELECT ... FROM users
        │                       │                       │
        │              UserEntity ──From──▶ User         │
        │                       │                        ▼
        └──────  User ──From──▶ UserView ──▶ risposta GraphQL
```

---

## Come si implementa ogni pezzo

### 1. Il dominio (`domain/user.rs`)

```rust
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::domain::error::DomainError;

pub struct User {
    pub id: Uuid,
    pub name: String,
    pub balance: Decimal,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self { id: Uuid::new_v4(), name: name.trim().to_string(), balance: Decimal::ZERO }
    }

    // La logica di business vive nel dominio
    pub fn deposit(&mut self, amount: Decimal) -> Result<(), DomainError> {
        if amount <= Decimal::ZERO {
            return Err(DomainError::InvalidAmount(amount));
        }
        self.balance += amount;
        Ok(())
    }
}
```

### 2. Il port (`ports/repository.rs`) — il contratto

```rust
use uuid::Uuid;
use crate::domain::user::User;
use crate::domain::error::DomainError;

// Il trait: definisce COSA serve, non COME. Il dominio dipende da questo,
// non dall'implementazione concreta.
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: User) -> Result<(), DomainError>;
}
```

### 3. L'adapter outbound (`adapters/outbound/postgres.rs`) — implementa il port

```rust
use sqlx::PgPool;
use crate::domain::user::User;
use crate::ports::repository::UserRepository;

// UserEntity vive QUI: è un dettaglio di persistenza, non di dominio
#[derive(sqlx::FromRow)]
struct UserEntity {
    id: Uuid,
    name: String,
    balance: Decimal,
}

// I mapper stanno al confine: traducono tra DB e dominio
impl From<UserEntity> for User {
    fn from(e: UserEntity) -> Self {
        Self { id: e.id, name: e.name, balance: e.balance }
    }
}
impl From<User> for UserEntity {
    fn from(u: User) -> Self {
        Self { id: u.id, name: u.name, balance: u.balance }
    }
}

pub struct PostgresUserRepository { pool: PgPool }

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        let entity = sqlx::query_as::<_, UserEntity>(
            "SELECT id, name, balance FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::UserNotFound(id))?;
        Ok(entity.into())
    }

    async fn save(&self, user: User) -> Result<(), DomainError> {
        let e: UserEntity = user.into();
        sqlx::query("INSERT INTO users (id, name, balance) VALUES ($1, $2, $3)")
            .bind(e.id).bind(&e.name).bind(e.balance)
            .execute(&self.pool)
            .await
            .map_err(|_| DomainError::Internal)?;
        Ok(())
    }
}
```

### 4. L'application (`application/handlers.rs`) — orchestra

```rust
use std::sync::Arc;
use crate::domain::user::User;
use crate::ports::repository::UserRepository;
use crate::application::commands::CreateUserCommand;

pub struct UserHandler {
    repo: Arc<dyn UserRepository>,     // dipende dal PORT (trait), non dall'impl
}

impl UserHandler {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self { Self { repo } }

    pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<User, DomainError> {
        let user = User::new(&cmd.name);        // Command → entità di dominio
        self.repo.save(user.clone()).await?;    // salva via port
        Ok(user)
    }
}
```

### 5. L'adapter inbound (`adapters/inbound/graph_ql/user.rs`) — espone GraphQL

```rust
use async_graphql::*;
use crate::application::commands::CreateUserCommand;
use crate::domain::user::User;

// Input GraphQL — vestito del protocollo
#[derive(InputObject)]
pub struct CreateUserInput {
    pub name: String,
}

// Output GraphQL — la view di risposta
#[derive(SimpleObject)]
pub struct UserView {
    pub id: Uuid,
    pub name: String,
    pub balance: Decimal,
}

// Mapper al confine GraphQL ↔ dominio
impl From<CreateUserInput> for CreateUserCommand {
    fn from(i: CreateUserInput) -> Self { Self { name: i.name } }
}
impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self { id: u.id, name: u.name, balance: u.balance }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput)
        -> Result<UserView>
    {
        let handler = ctx.data::<UserHandler>()?;
        let command: CreateUserCommand = input.into();      // GraphQL → Command
        let user = handler.create_user(command).await?;     // application
        Ok(user.into())                                     // dominio → UserView
    }
}
```

### 6. Il wiring (`main.rs`) — collega tutto

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = /* crea PgPool */;

    // Costruisci il grafo delle dipendenze (composition root)
    let repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool));
    let handler = UserHandler::new(repo.clone());

    // Schema GraphQL con l'handler iniettato
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(handler)
        .finish();

    // Actix serve lo schema
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .route("/graphql", web::post().to(graphql_handler))
    })
    .bind("0.0.0.0:8080")?.run().await
}
```

---

## Quanti mapper? (livello pragmatico)

Non tutte le conversioni servono. Collassa quelle "identità", tieni quelle vere:

| Conversione | Serve? | Perché |
|---|---|---|
| `CreateUserInput → CreateUserCommand` | opzionale | se identici, si può collassare in un tipo solo |
| `CreateUserCommand → User` | sì | il dominio genera id, applica regole |
| `User ↔ UserEntity` | **sì** | l'entità ha la forma del DB, diversa dal dominio |
| `User → UserView` | **sì** | la view può nascondere campi / denormalizzare |

Domanda guida per ogni mapper: *"questi due tipi sono davvero diversi, o sto copiando
gli stessi campi per dogma?"* Se diversi → mapper. Se identici → collassa.

---

## Regola d'oro

- Il **dominio** non importa mai `sqlx`, `async_graphql`, `actix`, `rdkafka`.
- Gli **adapter** dipendono dal dominio e dai port, mai il contrario.
- I **mapper** stanno al confine (nell'adapter che possiede il tipo non-dominio).
- Se il dominio compila senza le dipendenze web/db nel suo `Cargo.toml`,
  l'architettura sta funzionando.
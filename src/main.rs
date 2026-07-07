use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use crate::ports::{UserRepository, OrderRepository};
use crate::adapters::outbound::{KafkaService, PostgresUserRepository};
use crate::adapters::outbound::kafka::{create_consumer,create_producer};
use crate::adapters::inbound::graph_ql::{build_schema, graphql_handler, graphql_playground};
use crate::application::{UserHandler, OrderHandler};
use crate::config::{DatabaseConfig, create_pg_pool};

mod domain;
mod config;
mod application;
mod ports;
mod adapters;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info,sqlx=debug")
        .pretty()
        .init();

    let config = DatabaseConfig::from_env().expect("invalid database config");
    let pool = create_pg_pool(&config).await.expect("failed to create db pool");

    sqlx::migrate!("./migrations").run(&pool).await.expect("migration failed");
    tracing::info!("Migrazioni applicate");

    // Costruisci il grafo delle dipendenze (composition root)
    // Stessa impl (PostgresUserRepository) soddisfa entrambi i trait; pool e' Clone (Arc dentro)

    // === KAFKA SERVICE ===
    let producer = Arc::new(create_producer("localhost:9092"));
    let consumer = Arc::new(create_consumer("localhost:9092", "my-service-group"));
    let kafka_service = Arc::new(KafkaService::new(producer, consumer));

    // Avvia il consumer in background
    kafka_service.clone().start_consuming();
    // ↑ Tutto incapsulato nel service, clean!

    // === REPOSITORIES E HANDLERS ===
    let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
    let order_repo: Arc<dyn OrderRepository> = Arc::new(PostgresUserRepository::new(pool));
    let user_handler = Arc::new(UserHandler::new(user_repo, kafka_service.clone()));
    let order_handler = Arc::new(OrderHandler::new(order_repo));

    let schema = build_schema(user_handler, order_handler, kafka_service);

    // Actix serve lo schema
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(graphql_handler)
            .service(graphql_playground)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
mod schema;
mod user;
mod order;

use crate::application::{OrderHandler, UserHandler};
use actix_web::{get, post, web, HttpResponse, Responder};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use std::sync::Arc;

use crate::adapters::outbound::KafkaService;
pub use schema::{MutationRoot, QueryRoot};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(user_handler: Arc<UserHandler>, order_handler: Arc<OrderHandler>, kafka_service: Arc<KafkaService>) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(user_handler)   // ctx.data::<Arc<UserHandler>>()
        .data(order_handler) // usato dal ComplexObject `orders`
        .data(kafka_service)
        .finish()
}

// 1. LA ROTTA GET: GraphiQL con Explorer point-and-click
#[get("/graphql")]
pub async fn graphql_playground() -> impl Responder {
    use async_graphql::http::GraphiQLSource;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
}

// 2. LA ROTTA POST: Elabora le query reali (corretto il tipo in AppSchema)
#[post("/graphql")]
pub async fn graphql_handler(schema: web::Data<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

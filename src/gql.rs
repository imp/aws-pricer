use async_graphql::Context;
use async_graphql::Object;
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
// use axum::{
//     extract::Extension,
//     response::{self, IntoResponse},
//     routing::get,
//     Router, Server,
// };

use super::*;

type PricerSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn service(&self, ctx: &Context<'_>) -> Vec<types::Service> {
        let state = ctx.data::<Arc<state::State>>().unwrap();
        state.get_all_services().await
    }
}

pub(crate) async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub(crate) async fn graphql(
    schema: Extension<PricerSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub(crate) fn schema(state: Arc<state::State>) -> PricerSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .finish()
}

use async_graphql::dataloader::DataLoader;
use async_graphql::Context;
use async_graphql::Object;
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use super::*;

mod loader;

type PricerSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn service(&self, ctx: &Context<'_>, code: Option<String>) -> Vec<types::Service> {
        let state = ctx.data::<Arc<state::State>>().unwrap();
        let loader = ctx.data::<DataLoader<loader::PricingLoader>>().unwrap();
        if let Some(code) = code {
            loader
                .load_one(code)
                .await
                .unwrap_or_default()
                .into_iter()
                .collect()
            // state.get_service(&code).await.into_iter().collect()
        } else {
            state.get_all_services().await
        }
    }

    async fn product(
        &self,
        ctx: &Context<'_>,
        code: String,
        attributes: HashMap<String, String>,
    ) -> Vec<json::Value> {
        let state = ctx.data::<Arc<state::State>>().unwrap();
        state.products(code, attributes).await
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
    let loader = loader::PricingLoader::new(&state);
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .data(DataLoader::new(loader, tokio::spawn))
        .finish()
}

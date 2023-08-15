use async_graphql::dataloader::Loader;

use super::*;

pub(super) struct PricingLoader {
    state: Arc<state::State>,
}

impl PricingLoader {
    pub(super) fn new(state: &Arc<state::State>) -> Self {
        let state = state.clone();
        Self { state }
    }
}

#[async_trait::async_trait]
impl Loader<String> for PricingLoader {
    type Value = types::Service;
    type Error = ();

    async fn load(&self, codes: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut ok = HashMap::new();
        for code in codes {
            if let Some(service) = self.state.load_service(code).await {
                ok.insert(code.clone(), service);
            }
        }

        Ok(ok)
    }
}

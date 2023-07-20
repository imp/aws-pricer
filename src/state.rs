use super::*;

#[derive(Debug)]
pub struct State {
    client: pricing::AwsPricingClient,
}

impl State {
    pub async fn new(client: pricing::AwsPricingClient) -> Self {
        Self { client }
    }

    pub fn pricing(&self) -> &pricing::AwsPricingClient {
        &self.client
    }
}

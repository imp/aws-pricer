use aws_sdk_pricing as pricing;
use aws_types::region::Region;
use tokio_stream::StreamExt;

use super::*;

pub type PricingResult<T> = Result<T, pricing::Error>;
const US_EAST_1: Region = Region::from_static("us-east-1");

#[derive(Debug)]
pub struct AwsPricingClient {
    client: pricing::Client,
}

impl AwsPricingClient {
    pub async fn new(secrets: shuttle_secrets::SecretStore) -> Self {
        let credentials = ShuttleSecretsAwsCredentials::new(secrets);
        let sdk_config = ::aws_config::from_env()
            .region(US_EAST_1)
            .credentials_provider(credentials)
            .load()
            .await;
        let client = pricing::Client::new(&sdk_config);
        Self { client }
    }

    pub async fn services(&self) -> Vec<String> {
        self.describe_services_impl(None)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|service| service.service_code)
            .collect()
    }

    async fn describe_services_impl(
        &self,
        service_code: Option<String>,
    ) -> PricingResult<Vec<pricing::types::Service>> {
        let services = self
            .client
            .describe_services()
            .set_service_code(service_code)
            .into_paginator()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?
            .into_iter()
            .filter_map(|output| output.services)
            .flatten()
            .collect();
        Ok(services)
    }
}

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

    pub async fn services(&self) -> HashMap<String, Vec<String>> {
        self.describe_services_impl(None)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|service| {
                service
                    .service_code
                    .map(|code| (code, service.attribute_names.unwrap_or_default()))
            })
            .collect()
    }

    pub async fn service(&self, code: String) -> Vec<String> {
        self.describe_services_impl(Some(code))
            .await
            .unwrap_or_default()
            .pop()
            .and_then(|service| service.attribute_names)
            .unwrap_or_default()
    }

    pub async fn attribute(&self, code: String, attribute: String) -> Vec<String> {
        self.attribute_values(code, attribute)
            .await
            .unwrap_or_default()
    }

    pub async fn products(
        &self,
        code: String,
        attributes: HashMap<String, String>,
    ) -> Vec<json::Value> {
        let filters = attributes.into_iter().map(filter).collect();
        self.get_products(code, filters)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|text| json::from_str(&text).ok())
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

    async fn attribute_values(
        &self,
        code: String,
        attribute: String,
    ) -> PricingResult<Vec<String>> {
        let values = self
            .client
            .get_attribute_values()
            .service_code(code)
            .attribute_name(attribute)
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?
            .into_iter()
            .filter_map(|value| value.value)
            .collect();
        Ok(values)
    }

    async fn get_products(
        &self,
        code: String,
        filters: Vec<pricing::types::Filter>,
    ) -> PricingResult<Vec<String>> {
        let prices = self
            .client
            .get_products()
            .service_code(code)
            .set_filters(Some(filters))
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;
        Ok(prices)
    }
}

fn filter((field, value): (String, String)) -> pricing::types::Filter {
    pricing::types::Filter::builder()
        .r#type(pricing::types::FilterType::TermMatch)
        .field(field)
        .value(value)
        .build()
}

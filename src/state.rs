use tokio::sync::Mutex;
use tokio::time;

use super::*;

#[derive(Debug)]
pub struct State {
    client: pricing::AwsPricingClient,
    services: Mutex<HashMap<String, types::Service>>,
    load_stats: Mutex<HashMap<String, time::Duration>>,
}

impl State {
    pub async fn new(client: pricing::AwsPricingClient) -> Self {
        let services = Mutex::new(HashMap::new());
        let load_stats = Mutex::new(HashMap::new());
        Self {
            client,
            services,
            load_stats,
        }
    }

    pub(crate) async fn services(&self) -> Vec<types::Service> {
        self.client
            .services()
            .await
            .into_iter()
            .map(types::Service::from)
            .collect()
    }

    pub(crate) async fn service(&self, code: String) -> Option<types::Service> {
        self.client.service(code).await.map(types::Service::from)
    }

    pub(crate) async fn attribute(&self, code: String, attribute: String) -> types::Attribute {
        let values = self
            .client
            .attribute(code, attribute.clone())
            .await
            .into_iter()
            .filter_map(|value| value.value)
            .collect();
        types::Attribute::new(attribute).with_values(values)
    }

    pub(crate) async fn products(
        &self,
        code: String,
        attributes: HashMap<String, String>,
    ) -> Vec<json::Value> {
        self.client.products(code, attributes).await
    }

    pub(crate) async fn get_all_services(&self) -> Vec<types::Service> {
        self.services.lock().await.values().cloned().collect()
    }

    pub(crate) async fn get_service(&self, code: &str) -> Option<types::Service> {
        self.services.lock().await.get(code).cloned()
    }

    pub(crate) async fn codes(&self) -> json::Value {
        let services = self.services.lock().await;
        json::to_value(&*services).unwrap_or_default()
    }

    pub(crate) async fn stats(&self) -> json::Value {
        self.load_stats
            .lock()
            .await
            .iter()
            .map(|(code, duration)| json::json!({ code: format!("{duration:?}") }))
            .collect::<Vec<_>>()
            .into()
    }

    async fn fill_attribute_values(&self, service: types::Service) -> types::Service {
        let mut attributes = Vec::with_capacity(service.attributes.len());
        for attribute in service.attributes {
            let attribute = self.attribute(service.code.clone(), attribute.name).await;
            attributes.push(attribute);
        }
        types::Service {
            attributes,
            ..service
        }
    }

    pub async fn load_services(&self) {
        for service in self.services().await {
            let now = time::Instant::now();
            let service = self.fill_attribute_values(service).await;
            self.load_stats
                .lock()
                .await
                .insert(service.code.clone(), now.elapsed());
            self.services
                .lock()
                .await
                .insert(service.code.clone(), service);
        }
    }
}

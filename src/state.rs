use tokio::sync::Mutex;
use tokio::time;

use super::*;

#[derive(Debug)]
pub struct State {
    client: pricing::AwsPricingClient,
    services: Mutex<HashMap<String, types::Service>>,
    load_duration: Mutex<HashMap<String, time::Duration>>,
}

impl State {
    pub async fn new(client: pricing::AwsPricingClient) -> Self {
        let services = Mutex::new(HashMap::new());
        let load_duration = Mutex::new(HashMap::new());
        Self {
            client,
            services,
            load_duration,
        }
    }

    fn pricing(&self) -> &pricing::AwsPricingClient {
        &self.client
    }

    pub(crate) async fn services(&self) -> Vec<types::Service> {
        self.pricing()
            .services()
            .await
            .into_iter()
            .map(types::Service::from)
            .collect()
    }

    pub(crate) async fn service(&self, code: String) -> Option<types::Service> {
        self.pricing().service(code).await.map(types::Service::from)
    }

    pub(crate) async fn attribute(&self, code: String, attribute: String) -> types::Attribute {
        let values = self
            .pricing()
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
        self.pricing().products(code, attributes).await
    }

    pub(crate) async fn codes(&self) -> json::Value {
        let services = self.services.lock().await;
        json::to_value(&*services).unwrap_or_default()
    }

    pub(crate) async fn load_duration(&self) -> json::Value {
        let duration = self.load_duration.lock().await;
        json::to_value(&*duration).unwrap_or_default()
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
            self.load_duration
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

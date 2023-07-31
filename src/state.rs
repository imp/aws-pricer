use tokio::sync::Mutex;
use tokio::time;

use super::*;

#[derive(Debug)]
pub struct State {
    client: pricing::AwsPricingClient,
    services: Mutex<HashMap<String, HashMap<String, Vec<String>>>>,
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

    pub fn pricing(&self) -> &pricing::AwsPricingClient {
        &self.client
    }

    pub async fn codes(&self) -> json::Value {
        let services = self.services.lock().await;
        json::to_value(&*services).unwrap_or_default()
    }

    pub async fn load_duration(&self) -> json::Value {
        let duration = self.load_duration.lock().await;
        json::to_value(&*duration).unwrap_or_default()
    }

    pub async fn load_services(&self) {
        for (code, attributes) in self.client.services().await {
            let now = time::Instant::now();
            let mut service = HashMap::new();
            for attribute in attributes {
                let values = self.client.attribute(code.clone(), attribute.clone()).await;
                service.insert(attribute, values);
            }
            self.load_duration
                .lock()
                .await
                .insert(code.clone(), now.elapsed());
            self.services.lock().await.insert(code, service);
        }
    }
}

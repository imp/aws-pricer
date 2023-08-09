use aws_sdk_pricing as pricing;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, async_graphql::SimpleObject)]
pub(crate) struct Service {
    pub(crate) code: String,
    pub(crate) attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, Serialize, Deserialize, async_graphql::SimpleObject)]
pub(crate) struct Attribute {
    pub(crate) name: String,
    pub(crate) values: Vec<String>,
}

impl Service {
    pub(crate) fn test() -> Self {
        Self {
            code: "code".to_string(),
            attributes: vec![],
        }
    }

    pub(crate) fn attrs(&self) -> Vec<String> {
        self.attributes
            .iter()
            .map(|attr| attr.name.clone())
            .collect()
    }
}

impl From<pricing::types::Service> for Service {
    fn from(service: pricing::types::Service) -> Self {
        let pricing::types::Service {
            service_code,
            attribute_names,
            ..
        } = service;

        let code = service_code.unwrap_or_default();
        let attributes = attribute_names
            .unwrap_or_default()
            .into_iter()
            .map(Attribute::new)
            .collect();

        Self { code, attributes }
    }
}

impl Attribute {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: vec![],
        }
    }

    pub fn with_values(self, values: Vec<String>) -> Self {
        Self { values, ..self }
    }
}

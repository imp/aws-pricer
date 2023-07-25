use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Params {
    #[serde(default)]
    content: ContentType,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ContentType {
    #[default]
    Html,
    Json,
}

impl Params {
    pub fn is_json(&self) -> bool {
        self.content == ContentType::Json
    }

    pub fn is_html(&self) -> bool {
        self.content == ContentType::Html
    }
}

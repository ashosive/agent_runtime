use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelTag {
    pub name: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    pub digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub name: String,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateStreamChunk {
    pub model: Option<String>,
    pub created_at: Option<String>,
    pub response: Option<String>,
    pub done: Option<bool>,
}

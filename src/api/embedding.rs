use crate::IntoRequest;
use derive_builder::Builder;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Builder)]
#[builder(pattern = "mutable")]
pub struct EmbeddingRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for text-embedding-ada-002), cannot be an empty string, and any array must be 2048 dimensions or less.
    input: EmbeddingInput,
    /// ID of the model to use. You can use the List models API to see all of your available models, or see our Model overview for descriptions of them.
    #[builder(default)]
    model: EmbeddingModel,
    /// The format to return the embeddings in. Can be either float or base64.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<EmbeddingEncodingFormat>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. Learn more.
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

// currently we don't support array of integers, or array of array of integers
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingModel {
    #[default]
    #[serde(rename = "text-embedding-ada-002")]
    TextEmbeddingAda002,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingEncodingFormat {
    #[default]
    Float,
    Base64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingData {
    /// The index of the embedding in the list of embeddings.
    pub index: usize,
    /// The embedding vector, which is a list of floats. The length of vector depends on the model as listed in the embedding guide.
    pub embedding: Vec<f32>,
    /// The object type, which is always "embedding".
    pub object: String,
}

impl IntoRequest for EmbeddingRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = format!("{}/embeddings", base_url);
        client.post(url).json(&self)
    }
}

impl EmbeddingRequest {
    pub fn new(input: impl Into<EmbeddingInput>) -> Self {
        EmbeddingRequestBuilder::default()
            .input(input.into())
            .build()
            .unwrap()
    }

    pub fn new_array(input: Vec<String>) -> Self {
        EmbeddingRequestBuilder::default()
            .input(input.into())
            .build()
            .unwrap()
    }
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(s: Vec<String>) -> Self {
        Self::StringArray(s)
    }
}

impl From<&[String]> for EmbeddingInput {
    fn from(s: &[String]) -> Self {
        Self::StringArray(s.to_vec())
    }
}

#[cfg(test)]
mod test {
    use crate::{EmbeddingRequest, SDK};
    use anyhow::Result;

    #[tokio::test]
    async fn test() -> Result<()> {
        let req = EmbeddingRequest::new("Hello, my dog is cute.");
        let _res = SDK.embedding(req).await?;
        Ok(())
    }
}

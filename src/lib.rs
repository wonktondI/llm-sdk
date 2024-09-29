mod api;
mod middleware;

use crate::middleware::RetryMiddleware;
use anyhow::Result;
pub use api::*;
use bytes::Bytes;
use derive_builder::Builder;
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use schemars::{schema_for, JsonSchema};
use std::time::Duration;
use tracing::{error, info};

const TIMEOUT: u64 = 30;
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone, Builder)]
pub struct LlmSDK {
    #[builder(setter(into), default = r#""https://api.openai.com/v1".into()"#)]
    pub(crate) base_url: String,
    #[builder(setter(into))]
    pub(crate) token: String,
    #[allow(dead_code)]
    #[builder(default = "3")]
    pub(crate) max_retries: u32,
    #[builder(setter(skip), default = "self.default_client()")]
    pub(crate) client: ClientWithMiddleware,
}

pub trait IntoRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder;
}

/// For tool function. If you have a function that you want ChatGPT to call, you shall put
/// all params into a struct and derive schemars::JsonSchema for it. Then you can use
/// `YourStruct::to_schema()` to generate json schema for tools.
pub trait ToSchema: JsonSchema {
    fn to_schema() -> serde_json::Value;
}

impl LlmSDKBuilder {
    fn default_client(&self) -> ClientWithMiddleware {
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(self.max_retries.unwrap_or(MAX_RETRIES));
        info!("init client");
        let m = RetryTransientMiddleware::new_with_policy(retry_policy);
        ClientBuilder::new(
            reqwest::Client::builder()
                .build()
                .unwrap(),
        )
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryMiddleware::from(m))
        .build()
    }
}

impl LlmSDK {
    pub fn new(token: impl Into<String>) -> Self {
        LlmSDKBuilder::default().token(token).build().unwrap()
    }

    // fixme Method new1 can run to retry, but new can't
    pub fn new1(base_url: impl Into<String>, token: impl Into<String>, max_retries: u32) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);
        let client = ClientBuilder::new(Client::new())
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self {
            base_url: base_url.into(),
            token: token.into(),
            max_retries: 3,
            client,
        }
    }

    pub fn new_with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        LlmSDKBuilder::default()
            .token(token)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    pub async fn chat_completion(
        &self,
        req: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<ChatCompletionResponse>().await?)
    }

    pub async fn create_image(&self, req: CreateImageRequest) -> Result<CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send().await?;
        Ok(res.json::<CreateImageResponse>().await?)
    }

    pub async fn speech(&self, req: SpeechRequest) -> Result<Bytes> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.bytes().await?)
    }

    pub async fn whisper(&self, req: WhisperRequest) -> Result<WhisperResponse> {
        let is_json = req.response_format == WhisperResponseFormat::Json;
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        let ret = if is_json {
            res.json::<WhisperResponse>().await?
        } else {
            let text = res.text().await?;
            WhisperResponse { text }
        };
        Ok(ret)
    }

    pub async fn embedding(&self, req: EmbeddingRequest) -> Result<Bytes> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.bytes().await?)
    }

    fn prepare_request(&self, req: impl IntoRequest) -> RequestBuilder {
        let req = req.into_request(&self.base_url, self.client.clone());
        let req = if self.token.is_empty() {
            req
        } else {
            req.bearer_auth(&self.token)
                .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
        };
        req.timeout(Duration::from_secs(TIMEOUT))
    }
}

trait SendAndLog {
    async fn send_and_log(self) -> Result<Response>;
}

impl SendAndLog for RequestBuilder {
    async fn send_and_log(self) -> Result<Response> {
        let res = self.send().await?;
        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            let text = res.text().await?;
            error!("API failed: {}", text);
            return Err(anyhow::anyhow!("API failed: {}", text));
        }
        Ok(res)
    }
}

impl<T: JsonSchema> ToSchema for T {
    fn to_schema() -> serde_json::Value {
        serde_json::to_value(schema_for!(Self)).unwrap()
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
}

use crate::IntoRequest;
use derive_builder::Builder;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateImageRequest {
    /// A text description of the desired image(s). The maximum length is 4000 characters for dall-e-3.
    #[builder(setter(into))]
    prompt: String,
    /// The model to use for image generation. Only support Dall-e-3
    #[builder(default)]
    model: ImageModel,
    /// The number of images to generate. Must be between 1 and 10. For dall-e-3, only n=1 is supported.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<usize>,
    /// The quality of the image that will be generated. hd creates images with finer details and greater consistency across the image. This param is only supported for dall-e-3.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<ImageQuality>,
    /// The format in which the generated images are returned. Must be one of url or b64_json.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ImageResponseFormat>,
    /// The size of the generated images. Must be one of 1024x1024, 1792x1024, or 1024x1792 for dall-e-3 models.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<ImageSize>,
    /// The style of the generated images. Must be one of vivid or natural. Vivid causes the model to lean towards generating hyper-real and dramatic images. Natural causes the model to produce more natural, less hyper-real looking images. This param is only supported for dall-e-3.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<ImageStyle>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum ImageModel {
    #[serde(rename = "dall-e-3")]
    #[default]
    DallE3,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageQuality {
    #[default]
    Standard,
    Hd,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageResponseFormat {
    #[default]
    Url,
    B64Json,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum ImageSize {
    #[serde(rename = "1024x1024")]
    #[default]
    Large,
    #[serde(rename = "1792x1024")]
    LargeWide,
    #[serde(rename = "1024x1792")]
    LargeTall,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageStyle {
    #[default]
    Vivid,
    Natural,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateImageResponse {
    pub created: u64,
    pub data: Vec<ImageObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageObject {
    /// The base64-encoded JSON of the generated image, if response_format is b64_json
    pub b64_json: Option<String>,
    /// The URL of the generated image, if response_format is url (default).
    pub url: Option<String>,
    /// The prompt that was used to generate the image, if there was any revision to the prompt.
    pub revised_prompt: String,
}

impl IntoRequest for CreateImageRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = format!("{}/images/generations", base_url);
        client.post(url).json(&self)
    }
}

impl CreateImageRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        CreateImageRequestBuilder::default()
            .prompt(prompt)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::SDK;
    use anyhow::Result;

    #[tokio::test]
    async fn test_create_image_request() -> Result<()> {
        let req = CreateImageRequestBuilder::default()
            .prompt("draw a cute caterpillar")
            .style(ImageStyle::Natural)
            .quality(ImageQuality::Hd)
            .build()?;
        // println!("{}", serde_json::to_value(req)?);
        let res = SDK.create_image(req).await?;
        println!("{:?}", res);
        // https://oaidalleapiprodscus.blob.core.windows.net/private/org-yOPEjRLqCfhQQpKeyHvlOyYx/user-0Uz39xQ39nFbH7nz4Lz9cjUW/img-V2sJdapoAtUTrdtj0mZAO9ac.png?st=2024-01-09T14%3A58%3A20Z&se=2024-01-09T16%3A58%3A20Z&sp=r&sv=2021-08-06&sr=b&rscd=inline&rsct=image/png&skoid=6aaadede-4fb3-4698-a8f6-684d7786b067&sktid=a48cca56-e6da-484e-a814-9c849652bcb3&skt=2024-01-09T14%3A27%3A15Z&ske=2024-01-10T14%3A27%3A15Z&sks=b&skv=2021-08-06&sig=SEk%2BPNurCKOwkYkivnzZkGfJM7BN2GaL8ZsLFEwNBx4%3D
        Ok(())
    }
}

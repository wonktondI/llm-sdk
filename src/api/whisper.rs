use crate::IntoRequest;
use derive_builder::Builder;
use reqwest::multipart::{Form, Part};
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Builder, Serialize)]
#[builder(pattern = "mutable")]
pub struct WhisperRequest {
    /// The audio file object (not file name) to transcribe/translate, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    file: Vec<u8>,
    /// ID of the model to use. Only whisper-1 is currently available.
    #[builder(default)]
    model: WhisperModel,
    /// The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency. Should not use this for translation
    #[builder(default, setter(strip_option, into))]
    language: Option<String>,
    /// An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language for transcription, and should be English only for translation.
    #[builder(default, setter(strip_option, into))]
    prompt: Option<String>,
    /// The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    #[builder(default)]
    pub(crate) response_format: WhisperResponseFormat,
    /// The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    #[builder(default, setter(strip_option))]
    temperature: Option<f32>,

    request_type: WhisperRequestType,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, Display, Serialize)]
pub enum WhisperModel {
    #[default]
    #[strum(serialize = "whisper-1")]
    Whisper1,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, Display, Serialize)]
#[strum(serialize_all = "snake_case")]
pub enum WhisperResponseFormat {
    #[default]
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, Display, Serialize)]
pub enum WhisperRequestType {
    #[default]
    Transcription,
    Translation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhisperResponse {
    pub text: String,
}

impl WhisperRequest {
    pub fn transcription(data: Vec<u8>) -> Self {
        WhisperRequestBuilder::default()
            .file(data)
            .request_type(WhisperRequestType::Transcription)
            .build()
            .unwrap()
    }

    pub fn translation(data: Vec<u8>) -> Self {
        WhisperRequestBuilder::default()
            .file(data)
            .request_type(WhisperRequestType::Translation)
            .build()
            .unwrap()
    }

    pub fn into_form(self) -> Form {
        let part = Part::bytes(self.file)
            .file_name("file.mp3")
            .mime_str("audio/mp3")
            .unwrap();
        let mut form = Form::new()
            .part("file", part)
            .text("model", self.model.to_string())
            .text("response_format", self.response_format.to_string());

        form = match (self.request_type, self.language) {
            (WhisperRequestType::Transcription, Some(language)) => form.text("language", language),
            _ => form,
        };
        form = if let Some(prompt) = self.prompt {
            form.text("prompt", prompt)
        } else {
            form
        };
        if let Some(temperature) = self.temperature {
            form.text("temperature", temperature.to_string())
        } else {
            form
        }
    }
}

impl IntoRequest for WhisperRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = match self.request_type {
            WhisperRequestType::Transcription => format!("{}/audio/transcriptions", base_url),
            WhisperRequestType::Translation => format!("{}/audio/translations", base_url),
        };
        client.post(url).multipart(self.into_form())
    }
}

#[cfg(test)]
mod test {
    use crate::{WhisperRequest, SDK};
    use anyhow::Result;
    use std::fs;

    #[tokio::test]
    async fn test() -> Result<()> {
        let data = fs::read("test.mp3")?;
        let req = WhisperRequest::transcription(data);
        println!("{}", serde_json::to_value(&req)?);
        let res = SDK.whisper(req).await?;
        println!("{:?}", res);
        Ok(())
    }
}

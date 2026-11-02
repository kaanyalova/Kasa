mod image_embedding;
mod ocr;
mod onnx_rt;
mod text_embedding;
mod wdv;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use image_embedding::{
    ImageEmbeddingModel, ImageEmbeddingModelPaths, ImageEmbeddingModelSource,
};
pub use ocr::{OcrModel, OcrModelPaths, OcrModelSource};
pub use onnx_rt::OnnxRTSource;
pub use text_embedding::{TextEmbeddingModel, TextEmbeddingModelPaths, TextEmbeddingModelSource};
pub(crate) use wdv::WdvModel;
pub use wdv::WdvModelSource;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSourcesConfig {
    #[serde(flatten)]
    #[serde(rename = "OnnxRTVersion")]
    onnx_rt_sources: Vec<OnnxRTSource>,
    #[serde(rename = "OcrModel")]
    #[serde(default)]
    ocr_models: Vec<OcrModelSource>,
    #[serde(rename = "WdvModel")]
    #[serde(default)]
    wdv_models: Vec<WdvModelSource>,
    #[serde(rename = "TextEmbeddingModel")]
    #[serde(default)]
    text_embedding_models: Vec<TextEmbeddingModelSource>,
    #[serde(rename = "ImageEmbeddingModel")]
    #[serde(default)]
    image_embedding_models: Vec<ImageEmbeddingModelSource>,
}

#[derive(Default)]
pub struct ModelSources {
    pub ocr_models: HashMap<String, OcrModelSource>,
    pub wdv_models: HashMap<String, WdvModelSource>,
    pub text_embedding_models: HashMap<String, TextEmbeddingModelSource>,
    pub image_embedding_models: HashMap<String, ImageEmbeddingModelSource>,
    pub onnx_rt: HashMap<String, OnnxRTSource>,
}

impl ModelSourcesConfig {
    pub fn map_to_names(self) -> ModelSources {
        ModelSources {
            ocr_models: self
                .ocr_models
                .into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            wdv_models: self
                .wdv_models
                .into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            text_embedding_models: self
                .text_embedding_models
                .into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            image_embedding_models: self
                .image_embedding_models
                .into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            onnx_rt: self
                .onnx_rt_sources
                .into_iter()
                .map(|m| (m.version_reference.clone(), m))
                .collect(),
        }
    }
}

use std::path::Path;

use anyhow::Result;
use futures_util::StreamExt;
use reqwest::header::CONTENT_LENGTH;
use thiserror::Error;
use tokio::{fs::File, io::BufWriter};

#[derive(Debug)]
pub enum ModelDownloadProgress {
    HashingStart,
    HashingEnd,
    UnzippingStart,
    UnzippingEnd,
    DownloadProgress(ModelDownloadProgressValues),
}

#[derive(Debug)]
pub struct ModelDownloadProgressValues {
    url: String,
    bytes_downloaded: u64,
    bytes_total: u64,
}

pub trait Downloadable {}

#[async_trait::async_trait]
pub trait Model {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()>;

    async fn streaming_download(
        client: &reqwest::Client,
        url: &str,
        output_path: &Path,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        let bytes_total = client
            .head(url)
            .send()
            .await?
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|h| h.to_str().unwrap_or("0").parse::<u64>().ok())
            .unwrap_or(0);

        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut output = BufWriter::new(File::create(output_path).await?);

        let mut stream = client.get(url).send().await?.bytes_stream();

        let mut bytes_downloaded: u64 = 0;

        while let Some(item) = stream.next().await {
            let item = &mut item?;

            bytes_downloaded += item.len() as u64;

            tokio::io::copy(&mut item.as_ref(), &mut output).await?;
            on_progress(ModelDownloadProgress::DownloadProgress(
                ModelDownloadProgressValues {
                    url: url.to_owned(),
                    bytes_downloaded,
                    bytes_total,
                },
            ))
        }

        Ok(())
    }

    fn check_hash(path: &Path, expected: &str) -> Result<()> {
        let hash = sha256::try_digest(path)?;

        if hash != expected {
            return Err(ModelDownloadError::HashDoesNotMatch(
                hash,
                expected.to_string(),
                path.to_string_lossy().to_string(),
            )
            .into());
        }

        Ok(())
    }

    fn check_and_update_status(&mut self);

    fn status(&self) -> ModelState;

    fn check_if_already_downloaded(&mut self) -> bool {
        self.check_and_update_status();
        matches!(self.status(), ModelState::Downloaded)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModelState {
    Downloaded,
    NotDownloaded,
    Errored(String),
}

#[derive(Debug, Error)]
pub(crate) enum ModelDownloadError {
    #[error("the hash {0} != {1} for file {2}")]
    HashDoesNotMatch(String, String, String),
}

#[test]
fn test_download() {}

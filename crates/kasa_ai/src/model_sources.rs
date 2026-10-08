use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use fastembed::TokenizerFiles;
use futures_util::StreamExt;
use image::DynamicImage;
use reqwest::header::CONTENT_LENGTH;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[derive(Debug)]
pub enum ModelDownloadProgress {
    HashingStart,
    HashingEnd,
    DownloadProgress(ModelDownloadProgressValues),
}
#[derive(Debug)]
pub struct ModelDownloadProgressValues {
    url: String,
    bytes_downloaded: u64,
    bytes_total: u64,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnnxRTSource {
    version_reference: String,

    linux_x64_url: String,
    linux_x64_sha256: String,
    linux_x64_cuda_13_url: String,
    linux_x64_cuda_13_sha256: String,
    linux_x64_cuda_12_url: String,
    linux_x64_cuda_12_sha256: String,
    linux_arm64_url: String,
    linux_arm64_sha256: String,

    windows_x64_url: String,
    windows_x64_sha256: String,
    windows_x64_cuda_12_url: String,
    windows_x64_cuda_12_sha256: String,
    windows_x64_cuda_13_url: String,
    windows_x64_cuda_13_sha256: String,
    windows_arm64_url: String,
    windows_arm64_sha256: String,
    windows_arm64_x_url: String, // what is this?
    windows_arm64_x_sha256: String,

    osx_arm64_url: String,
    osx_arm64_sha256: String,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WdvModelSource {
    pub name: String,
    pub reference_url: String,

    pub model_url: String,
    pub model_sha256: String,
    pub tag_labels_url: String,
    pub tag_labels_sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WdvModelPaths {
    pub model_path: PathBuf,
    pub tag_labels_path: PathBuf,
}

#[derive(Clone)]
struct WdvModel {
    state: ModelState,
    paths: WdvModelPaths,
    source: WdvModelSource,
}

impl WdvModelSource {
    fn into_model(self) -> WdvModel {
        let paths = WdvModelPaths {
            model_path: PathBuf::new()
                .join("wdv")
                .join(&self.name)
                .join("model.onnx"),
            tag_labels_path: PathBuf::new()
                .join("wdv")
                .join(&self.name)
                .join("labels.csv"),
        };

        WdvModel {
            state: ModelState::NotDownloaded,
            paths,
            source: self,
        }
    }
}

#[derive(Debug, Error)]
enum ModelDownloadError {
    #[error("the hash {0} != {1} for file {2}")]
    HashDoesNotMatch(String, String, String),
}

#[async_trait::async_trait]
impl Model for WdvModel {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        Self::streaming_download(
            client,
            &self.source.model_url,
            &self.paths.model_path,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.tag_labels_url,
            &self.paths.tag_labels_path,
            &on_progress,
        )
        .await?;

        on_progress(ModelDownloadProgress::HashingStart);
        Self::check_hash(&self.paths.tag_labels_path, &self.source.tag_labels_sha256)?;
        Self::check_hash(&self.paths.model_path, &self.source.model_sha256)?;
        on_progress(ModelDownloadProgress::HashingEnd);

        self.state = ModelState::Downloaded;
        Ok(())
    }

    fn check_and_update_status(&mut self) {
        let does_the_files_exist =
            self.paths.model_path.exists() && self.paths.tag_labels_path.exists();

        if does_the_files_exist {
            self.state = ModelState::Downloaded
        }
    }

    fn status(&self) -> ModelState {
        self.state.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrModelSource {
    pub name: String,
    pub reference_url: String,

    pub rec_model_url: String,
    pub rec_model_sha256: String,
    pub det_model_url: String,
    pub det_model_sha256: String,
    pub dict_url: String,
    pub dict_sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrModelPaths {
    pub rec_model_path: PathBuf,
    pub det_model_path: PathBuf,
    pub dict_path: PathBuf,
}

pub struct OcrModel {
    state: ModelState,
    pub paths: OcrModelPaths,
    pub source: OcrModelSource,
}

impl OcrModelSource {
    pub fn into_model(self, base_path: &Path) -> OcrModel {
        let paths = OcrModelPaths {
            det_model_path: PathBuf::new()
                .join(base_path)
                .join("ocr")
                .join(&self.name)
                .join("det_model.onnx"),
            rec_model_path: PathBuf::new()
                .join(base_path)
                .join("ocr")
                .join(&self.name)
                .join("rec_model.onnx"),
            dict_path: PathBuf::new()
                .join(base_path)
                .join("ocr")
                .join(&self.name)
                .join("dictionary.txt"),
        };

        OcrModel {
            state: ModelState::NotDownloaded,
            paths,
            source: self,
        }
    }
}

#[async_trait::async_trait]
impl Model for OcrModel {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        Self::streaming_download(
            client,
            &self.source.det_model_url,
            &self.paths.det_model_path,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.rec_model_url,
            &self.paths.rec_model_path,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.dict_url,
            &self.paths.dict_path,
            &on_progress,
        )
        .await?;

        on_progress(ModelDownloadProgress::HashingStart);
        Self::check_hash(&self.paths.det_model_path, &self.source.det_model_sha256)?;
        Self::check_hash(&self.paths.rec_model_path, &self.source.rec_model_sha256)?;
        Self::check_hash(&self.paths.dict_path, &self.source.dict_sha256)?;
        on_progress(ModelDownloadProgress::HashingEnd);

        self.state = ModelState::Downloaded;
        Ok(())
    }

    fn check_and_update_status(&mut self) {
        let does_the_files_exist = self.paths.det_model_path.exists()
            && self.paths.rec_model_path.exists()
            && self.paths.dict_path.exists();

        if does_the_files_exist {
            self.state = ModelState::Downloaded
        }
    }

    fn status(&self) -> ModelState {
        self.state.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextEmbeddingModelSource {
    name: String,
    reference_url: String,

    model_url: String,
    model_sha256: String,

    tokenizer_file_url: String,
    tokenizer_file_sha256: String,
    config_file_url: String,
    config_file_sha256: String,
    special_tokens_map_file_url: String,
    special_tokens_map_file_sha256: String,
    tokenizer_config_file_url: String,
    tokenizer_config_file_sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextEmbeddingModelPaths {
    pub model: PathBuf,
    pub tokenizer_file: PathBuf,
    pub config_file: PathBuf,
    pub special_tokens_map_file: PathBuf,
    pub tokenizer_config_file: PathBuf,
}

impl TextEmbeddingModelPaths {
    pub fn read_tokenizer_files(&self) -> Result<TokenizerFiles> {
        Ok(TokenizerFiles {
            tokenizer_file: fs::read(&self.tokenizer_file)?,
            config_file: fs::read(&self.config_file)?,
            special_tokens_map_file: fs::read(&self.special_tokens_map_file)?,
            tokenizer_config_file: fs::read(&self.tokenizer_config_file)?,
        })
    }
}

#[derive(Clone)]
pub struct TextEmbeddingModel {
    state: ModelState,
    pub paths: TextEmbeddingModelPaths,
    pub source: TextEmbeddingModelSource,
}

impl TextEmbeddingModelSource {
    pub fn into_model(self, base_path: &Path) -> TextEmbeddingModel {
        let base_dir = PathBuf::new()
            .join(base_path)
            .join("text_embedding")
            .join(&self.name);

        let paths = TextEmbeddingModelPaths {
            model: base_dir.join("model.onnx"),
            tokenizer_file: base_dir.join("tokenizer.json"),
            config_file: base_dir.join("config.json"),
            special_tokens_map_file: base_dir.join("special_tokens_map.json"),
            tokenizer_config_file: base_dir.join("tokenizer_config.json"),
        };

        TextEmbeddingModel {
            state: ModelState::NotDownloaded,
            paths,
            source: self,
        }
    }
}

#[async_trait::async_trait]
impl Model for TextEmbeddingModel {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        let paths = &self.paths;

        Self::streaming_download(client, &self.source.model_url, &paths.model, &on_progress)
            .await?;
        Self::streaming_download(
            client,
            &self.source.tokenizer_file_url,
            &paths.tokenizer_file,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.config_file_url,
            &paths.config_file,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.special_tokens_map_file_url,
            &paths.special_tokens_map_file,
            &on_progress,
        )
        .await?;
        Self::streaming_download(
            client,
            &self.source.tokenizer_config_file_url,
            &paths.tokenizer_config_file,
            &on_progress,
        )
        .await?;

        on_progress(ModelDownloadProgress::HashingStart);
        Self::check_hash(&paths.model, &self.source.model_sha256)?;
        Self::check_hash(&paths.tokenizer_file, &self.source.tokenizer_file_sha256)?;
        Self::check_hash(&paths.config_file, &self.source.config_file_sha256)?;
        Self::check_hash(
            &paths.special_tokens_map_file,
            &self.source.special_tokens_map_file_sha256,
        )?;
        Self::check_hash(
            &paths.tokenizer_config_file,
            &self.source.tokenizer_config_file_sha256,
        )?;
        on_progress(ModelDownloadProgress::HashingEnd);

        self.state = ModelState::Downloaded;
        Ok(())
    }

    fn check_and_update_status(&mut self) {
        let does_the_files_exist = self.paths.model.exists()
            && self.paths.tokenizer_file.exists()
            && self.paths.config_file.exists()
            && self.paths.special_tokens_map_file.exists()
            && self.paths.tokenizer_config_file.exists();

        if does_the_files_exist {
            self.state = ModelState::Downloaded
        }
    }

    fn status(&self) -> ModelState {
        self.state.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageEmbeddingModelSource {
    name: String,
    reference_url: String,

    model_url: String,
    model_sha256: String,
    preprocessor_config_url: String,
    preprocessor_config_sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageEmbeddingModelPaths {
    pub model: PathBuf,
    pub preprocessor_config: PathBuf,
}

#[derive(Clone)]
pub struct ImageEmbeddingModel {
    state: ModelState,
    pub paths: ImageEmbeddingModelPaths,
    pub source: ImageEmbeddingModelSource,
}

impl ImageEmbeddingModelSource {
    pub fn into_model(self, base_path: &Path) -> ImageEmbeddingModel {
        let paths = ImageEmbeddingModelPaths {
            model: PathBuf::new()
                .join(base_path)
                .join("image_embedding")
                .join(&self.name)
                .join("model.onnx"),
            preprocessor_config: PathBuf::new()
                .join(base_path)
                .join("image_embedding")
                .join(&self.name)
                .join("preprocessor_config.json"),
        };

        ImageEmbeddingModel {
            state: ModelState::NotDownloaded,
            paths,
            source: self,
        }
    }
}

#[async_trait::async_trait]
impl Model for ImageEmbeddingModel {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        Self::streaming_download(
            client,
            &self.source.model_url,
            &self.paths.model,
            &on_progress,
        )
        .await?;

        Self::streaming_download(
            client,
            &self.source.preprocessor_config_url,
            &self.paths.preprocessor_config,
            &on_progress,
        )
        .await?;

        on_progress(ModelDownloadProgress::HashingStart);
        Self::check_hash(&self.paths.model, &self.source.model_sha256)?;
        Self::check_hash(
            &self.paths.preprocessor_config,
            &self.source.preprocessor_config_sha256,
        )?;
        on_progress(ModelDownloadProgress::HashingEnd);

        self.state = ModelState::Downloaded;
        Ok(())
    }

    fn check_and_update_status(&mut self) {
        let does_the_files_exist = self.paths.model.exists();

        if does_the_files_exist {
            self.state = ModelState::Downloaded
        }
    }

    fn status(&self) -> ModelState {
        self.state.clone()
    }
}

#[test]
fn test_download() {}

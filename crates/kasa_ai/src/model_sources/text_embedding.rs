use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use fastembed::TokenizerFiles;
use serde::{Deserialize, Serialize};

use crate::{
    model_manager::ModelManager,
    model_sources::{Model, ModelDownloadProgress, ModelState},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextEmbeddingModelSource {
    pub(crate) name: String,
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

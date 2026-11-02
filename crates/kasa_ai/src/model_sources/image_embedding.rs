use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model_sources::{Model, ModelDownloadProgress, ModelState, hash_source};

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct ImageEmbeddingModelSource {
    pub(crate) name: String,
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
    pub model_id: u64,
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

        let model_id = hash_source(&self);

        ImageEmbeddingModel {
            state: ModelState::NotDownloaded,
            model_id,
            paths,
            source: self,
        }
    }
}

#[async_trait::async_trait]
impl Model for ImageEmbeddingModel {
    fn model_id(&self) -> u64 {
        self.model_id
    }

    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        let model_id = self.model_id();

        Self::streaming_download(
            client,
            model_id,
            &self.source.model_url,
            &self.paths.model,
            &on_progress,
        )
        .await?;

        Self::streaming_download(
            client,
            model_id,
            &self.source.preprocessor_config_url,
            &self.paths.preprocessor_config,
            &on_progress,
        )
        .await?;

        on_progress(ModelDownloadProgress::HashingStart(model_id));
        Self::check_hash(&self.paths.model, &self.source.model_sha256)?;
        Self::check_hash(
            &self.paths.preprocessor_config,
            &self.source.preprocessor_config_sha256,
        )?;
        on_progress(ModelDownloadProgress::HashingEnd(model_id));

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

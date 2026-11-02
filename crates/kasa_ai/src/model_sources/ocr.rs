use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model_sources::{Model, ModelDownloadProgress, ModelState};

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

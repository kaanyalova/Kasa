use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model_sources::{Model, ModelDownloadProgress, ModelState};

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
pub(crate) struct WdvModelPaths {
    pub(crate) model_path: PathBuf,
    pub(crate) tag_labels_path: PathBuf,
}

#[derive(Clone)]
pub(crate) struct WdvModel {
    pub(crate) state: ModelState,
    pub(crate) paths: WdvModelPaths,
    pub(crate) source: WdvModelSource,
}

impl WdvModelSource {
    pub(crate) fn into_model(self, base_path: &Path) -> WdvModel {
        let paths = WdvModelPaths {
            model_path: PathBuf::new()
                .join(base_path)
                .join("wdv")
                .join(&self.name)
                .join("model.onnx"),
            tag_labels_path: PathBuf::new()
                .join(base_path)
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
